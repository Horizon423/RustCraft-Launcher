//! RustCraft Launcher — native engine.
//!
//! All launcher logic lives in this crate. The Java side (`MainActivity`) is a thin
//! Android shell that loads this library and calls the `Java_*` functions in
//! [`jni_bridge`].
//!
//! Module overview:
//!
//! * [`error`]   – the single error type used across the engine
//! * [`version`] – Minecraft version-id validation and classification
//! * [`profile`] – launch profiles and their on-disk text format
//! * [`launch`]  – JVM command-line construction
//! * [`engine`]  – engine lifecycle, persistence, and the process-wide instance
//! * [`jni_bridge`] – JNI entry points (the only code that touches the JVM)

pub const ENGINE_NAME: &str = "RustCraft Engine";
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------
pub mod error {
    use std::fmt;
    use std::io;

    /// Every failure the engine can report. The JNI bridge turns these into Java
    /// `IllegalStateException`s carrying the `Display` text.
    #[derive(Debug)]
    pub enum LauncherError {
        InvalidArgument(String),
        InvalidState(&'static str),
        NotFound(String),
        AlreadyExists(String),
        Io(io::Error),
        Parse(String),
        Jni(String),
        Panic(String),
    }

    impl fmt::Display for LauncherError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::InvalidArgument(m) => write!(f, "invalid argument: {m}"),
                Self::InvalidState(m) => write!(f, "invalid state: {m}"),
                Self::NotFound(m) => write!(f, "not found: {m}"),
                Self::AlreadyExists(m) => write!(f, "already exists: {m}"),
                Self::Io(e) => write!(f, "I/O error: {e}"),
                Self::Parse(m) => write!(f, "parse error: {m}"),
                Self::Jni(m) => write!(f, "JNI error: {m}"),
                Self::Panic(m) => write!(f, "native panic: {m}"),
            }
        }
    }

    impl std::error::Error for LauncherError {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                Self::Io(e) => Some(e),
                _ => None,
            }
        }
    }

    impl From<io::Error> for LauncherError {
        fn from(e: io::Error) -> Self {
            Self::Io(e)
        }
    }

    pub type Result<T> = std::result::Result<T, LauncherError>;
}

// ---------------------------------------------------------------------------
// Version ids
// ---------------------------------------------------------------------------
pub mod version {
    use crate::error::{LauncherError, Result};

    pub const MAX_VERSION_ID_LEN: usize = 64;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum VersionKind {
        /// Dotted release such as `1.20.4` or `1.8.9`.
        Release,
        /// Weekly snapshot such as `24w14a`.
        Snapshot,
        /// Anything else that is a safe identifier (loader builds, custom versions).
        Custom,
    }

    impl VersionKind {
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Release => "release",
                Self::Snapshot => "snapshot",
                Self::Custom => "custom",
            }
        }
    }

    /// Validates `id` and reports what kind of version it is.
    ///
    /// Ids become file-system path components, so only `[A-Za-z0-9._-]` is allowed,
    /// the first character must be alphanumeric, and `..` is rejected.
    pub fn classify(id: &str) -> Result<VersionKind> {
        if id.is_empty() || id.len() > MAX_VERSION_ID_LEN {
            return Err(LauncherError::InvalidArgument(format!(
                "version id must be 1..={MAX_VERSION_ID_LEN} characters"
            )));
        }
        let bytes = id.as_bytes();
        if !bytes[0].is_ascii_alphanumeric() {
            return Err(LauncherError::InvalidArgument(
                "version id must start with a letter or digit".into(),
            ));
        }
        if !bytes
            .iter()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_'))
        {
            return Err(LauncherError::InvalidArgument(format!(
                "version id '{id}' contains unsupported characters"
            )));
        }
        if id.contains("..") {
            return Err(LauncherError::InvalidArgument(
                "version id must not contain '..'".into(),
            ));
        }
        Ok(if is_release(id) {
            VersionKind::Release
        } else if is_snapshot(id) {
            VersionKind::Snapshot
        } else {
            VersionKind::Custom
        })
    }

    fn is_release(id: &str) -> bool {
        let parts: Vec<&str> = id.split('.').collect();
        (2..=3).contains(&parts.len())
            && parts
                .iter()
                .all(|p| !p.is_empty() && p.len() <= 3 && p.bytes().all(|b| b.is_ascii_digit()))
    }

    fn is_snapshot(id: &str) -> bool {
        let b = id.as_bytes();
        b.len() == 6
            && b[0].is_ascii_digit()
            && b[1].is_ascii_digit()
            && b[2] == b'w'
            && b[3].is_ascii_digit()
            && b[4].is_ascii_digit()
            && b[5].is_ascii_lowercase()
    }
}

// ---------------------------------------------------------------------------
// Profiles
// ---------------------------------------------------------------------------
pub mod profile {
    use crate::error::{LauncherError, Result};
    use crate::version;

    pub const MIN_MEMORY_MB: u32 = 512;
    pub const MAX_MEMORY_MB: u32 = 16_384;
    pub const DEFAULT_MEMORY_MB: u32 = 2_048;
    pub const MAX_NAME_LEN: usize = 48;
    const FORMAT_HEADER: &str = "# rustcraft profiles v1";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Profile {
        pub id: String,
        pub name: String,
        pub version_id: String,
        pub memory_mb: u32,
    }

    impl Profile {
        /// Builds a validated profile. The id is derived from the name.
        pub fn new(name: &str, version_id: &str, memory_mb: u32) -> Result<Self> {
            let name = name.trim();
            if name.is_empty() || name.chars().count() > MAX_NAME_LEN {
                return Err(LauncherError::InvalidArgument(format!(
                    "profile name must be 1..={MAX_NAME_LEN} characters"
                )));
            }
            if name.chars().any(char::is_control) {
                return Err(LauncherError::InvalidArgument(
                    "profile name must not contain control characters".into(),
                ));
            }
            version::classify(version_id)?;
            if !(MIN_MEMORY_MB..=MAX_MEMORY_MB).contains(&memory_mb) {
                return Err(LauncherError::InvalidArgument(format!(
                    "memory must be {MIN_MEMORY_MB}..={MAX_MEMORY_MB} MB, got {memory_mb}"
                )));
            }
            let id = slugify(name);
            if id.is_empty() {
                return Err(LauncherError::InvalidArgument(
                    "profile name must contain at least one letter or digit".into(),
                ));
            }
            Ok(Self {
                id,
                name: name.to_owned(),
                version_id: version_id.to_owned(),
                memory_mb,
            })
        }
    }

    /// Lower-cases `name`, replaces runs of non-alphanumerics with a single `-`.
    pub fn slugify(name: &str) -> String {
        let mut out = String::with_capacity(name.len());
        let mut pending_dash = false;
        for ch in name.chars() {
            if ch.is_ascii_alphanumeric() {
                if pending_dash && !out.is_empty() {
                    out.push('-');
                }
                pending_dash = false;
                out.push(ch.to_ascii_lowercase());
            } else {
                pending_dash = true;
            }
        }
        out
    }

    #[derive(Debug, Default, Clone)]
    pub struct ProfileStore {
        profiles: Vec<Profile>,
    }

    impl ProfileStore {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn len(&self) -> usize {
            self.profiles.len()
        }

        pub fn is_empty(&self) -> bool {
            self.profiles.is_empty()
        }

        pub fn list(&self) -> &[Profile] {
            &self.profiles
        }

        pub fn get(&self, id: &str) -> Result<&Profile> {
            self.profiles
                .iter()
                .find(|p| p.id == id)
                .ok_or_else(|| LauncherError::NotFound(format!("profile '{id}'")))
        }

        pub fn add(&mut self, profile: Profile) -> Result<()> {
            if self.profiles.iter().any(|p| p.id == profile.id) {
                return Err(LauncherError::AlreadyExists(format!(
                    "profile '{}'",
                    profile.id
                )));
            }
            self.profiles.push(profile);
            Ok(())
        }

        pub fn remove(&mut self, id: &str) -> Result<Profile> {
            let idx = self
                .profiles
                .iter()
                .position(|p| p.id == id)
                .ok_or_else(|| LauncherError::NotFound(format!("profile '{id}'")))?;
            Ok(self.profiles.remove(idx))
        }

        /// Serialises to a tab-separated text format: `id<TAB>name<TAB>version<TAB>memory`.
        pub fn to_text(&self) -> String {
            let mut out = String::from(FORMAT_HEADER);
            out.push('\n');
            for p in &self.profiles {
                out.push_str(&format!(
                    "{}\t{}\t{}\t{}\n",
                    p.id, p.name, p.version_id, p.memory_mb
                ));
            }
            out
        }

        /// Parses the format written by [`ProfileStore::to_text`]. Every record is
        /// re-validated, so a hand-edited or corrupted file cannot inject bad data.
        pub fn from_text(text: &str) -> Result<Self> {
            let mut lines = text.lines();
            match lines.next() {
                Some(h) if h.trim() == FORMAT_HEADER => {}
                _ => return Err(LauncherError::Parse("missing profile file header".into())),
            }
            let mut store = Self::new();
            for (idx, line) in lines.enumerate() {
                let line_no = idx + 2;
                if line.trim().is_empty() || line.starts_with('#') {
                    continue;
                }
                let fields: Vec<&str> = line.split('\t').collect();
                if fields.len() != 4 {
                    return Err(LauncherError::Parse(format!(
                        "line {line_no}: expected 4 fields, found {}",
                        fields.len()
                    )));
                }
                let memory: u32 = fields[3].trim().parse().map_err(|_| {
                    LauncherError::Parse(format!(
                        "line {line_no}: bad memory value '{}'",
                        fields[3]
                    ))
                })?;
                let profile = Profile::new(fields[1], fields[2], memory)
                    .map_err(|e| LauncherError::Parse(format!("line {line_no}: {e}")))?;
                if profile.id != fields[0] {
                    return Err(LauncherError::Parse(format!(
                        "line {line_no}: id '{}' does not match name",
                        fields[0]
                    )));
                }
                store
                    .add(profile)
                    .map_err(|e| LauncherError::Parse(format!("line {line_no}: {e}")))?;
            }
            Ok(store)
        }
    }
}

// ---------------------------------------------------------------------------
// Launch command construction
// ---------------------------------------------------------------------------
pub mod launch {
    use crate::profile::Profile;
    use std::path::Path;

    pub const MAIN_CLASS: &str = "net.minecraft.client.main.Main";

    pub struct LaunchSpec<'a> {
        pub java_bin: &'a str,
        pub game_dir: &'a Path,
        pub username: &'a str,
    }

    /// Builds the JVM argument vector for `profile`.
    ///
    /// This only *constructs* the command; spawning the game process is a future
    /// feature that will build on this function.
    pub fn build_command(profile: &Profile, spec: &LaunchSpec<'_>) -> Vec<String> {
        let version_jar = spec
            .game_dir
            .join("versions")
            .join(&profile.version_id)
            .join(format!("{}.jar", profile.version_id));
        let natives = spec.game_dir.join("natives").join(&profile.version_id);
        let initial_heap = profile.memory_mb.min(512);

        vec![
            spec.java_bin.to_owned(),
            format!("-Xms{initial_heap}M"),
            format!("-Xmx{}M", profile.memory_mb),
            "-Dfile.encoding=UTF-8".to_owned(),
            format!("-Djava.library.path={}", natives.to_string_lossy()),
            "-cp".to_owned(),
            version_jar.to_string_lossy().into_owned(),
            MAIN_CLASS.to_owned(),
            "--version".to_owned(),
            profile.version_id.clone(),
            "--gameDir".to_owned(),
            spec.game_dir.to_string_lossy().into_owned(),
            "--username".to_owned(),
            spec.username.to_owned(),
        ]
    }

    /// Joins arguments into one printable line, quoting any that contain whitespace.
    pub fn to_command_line(args: &[String]) -> String {
        args.iter()
            .map(|a| {
                if a.is_empty() || a.chars().any(char::is_whitespace) {
                    format!("\"{}\"", a.replace('"', "\\\""))
                } else {
                    a.clone()
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------
pub mod engine {
    use crate::error::{LauncherError, Result};
    use crate::launch::{self, LaunchSpec};
    use crate::profile::{Profile, ProfileStore, DEFAULT_MEMORY_MB};
    use crate::version;
    use crate::{ENGINE_NAME, ENGINE_VERSION};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Mutex, MutexGuard};

    const PROFILES_FILE: &str = "profiles.txt";
    const USERNAME_FILE: &str = "username.txt";
    const DEFAULT_PROFILE_NAME: &str = "Default";
    const DEFAULT_VERSION_ID: &str = "1.20.4";
    const DEFAULT_USERNAME: &str = "Player";

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum EngineState {
        Ready,
        Stopped,
    }

    impl EngineState {
        pub fn as_str(self) -> &'static str {
            match self {
                Self::Ready => "Ready",
                Self::Stopped => "Stopped",
            }
        }
    }

    #[derive(Debug)]
    pub struct Engine {
        state: EngineState,
        root: PathBuf,
        profiles: ProfileStore,
        username: String,
    }

    impl Engine {
        /// Creates the engine, using `data_dir/rustcraft` for persistent state.
        /// Existing profiles are loaded; on first run a default profile is created.
        pub fn init(data_dir: &str) -> Result<Self> {
            let data_dir = data_dir.trim();
            if data_dir.is_empty() {
                return Err(LauncherError::InvalidArgument(
                    "data directory is empty".into(),
                ));
            }
            let root = Path::new(data_dir).join("rustcraft");
            fs::create_dir_all(&root)?;

            let profiles_path = root.join(PROFILES_FILE);
            let mut engine = Self {
                state: EngineState::Ready,
                root,
                profiles: ProfileStore::new(),
                username: DEFAULT_USERNAME.to_owned(),
            };
            let username_path = engine.root.join(USERNAME_FILE);
            if username_path.exists() {
                engine.username = validate_username(&fs::read_to_string(username_path)?)?;
            } else {
                engine.save_username(DEFAULT_USERNAME)?;
            }
            if profiles_path.exists() {
                engine.profiles = ProfileStore::from_text(&fs::read_to_string(&profiles_path)?)?;
            }
            if engine.profiles.is_empty() {
                let default =
                    Profile::new(DEFAULT_PROFILE_NAME, DEFAULT_VERSION_ID, DEFAULT_MEMORY_MB)?;
                engine.profiles.add(default)?;
                engine.save_profiles()?;
            }
            Ok(engine)
        }

        pub fn state(&self) -> EngineState {
            self.state
        }

        pub fn profiles(&self) -> &ProfileStore {
            &self.profiles
        }

        pub fn username(&self) -> &str {
            &self.username
        }

        /// Validates and persists the offline username used in generated launch arguments.
        pub fn set_username(&mut self, username: &str) -> Result<()> {
            self.ensure_ready()?;
            self.save_username(username)?;
            self.username = validate_username(username)?;
            Ok(())
        }

        pub fn game_dir(&self) -> PathBuf {
            self.root.join("game")
        }

        fn ensure_ready(&self) -> Result<()> {
            if self.state == EngineState::Ready {
                Ok(())
            } else {
                Err(LauncherError::InvalidState("engine is not running"))
            }
        }

        /// Writes profiles atomically (temp file + rename) so a crash mid-write
        /// cannot leave a truncated profile list behind.
        fn save_profiles(&self) -> Result<()> {
            let path = self.root.join(PROFILES_FILE);
            let tmp = self.root.join(format!("{PROFILES_FILE}.tmp"));
            fs::write(&tmp, self.profiles.to_text())?;
            fs::rename(&tmp, &path)?;
            Ok(())
        }

        fn save_username(&self, username: &str) -> Result<()> {
            let username = validate_username(username)?;
            let path = self.root.join(USERNAME_FILE);
            let tmp = self.root.join(format!("{USERNAME_FILE}.tmp"));
            fs::write(&tmp, username)?;
            fs::rename(&tmp, &path)?;
            Ok(())
        }

        /// Creates and persists a profile, returning its id.
        pub fn create_profile(
            &mut self,
            name: &str,
            version_id: &str,
            memory_mb: u32,
        ) -> Result<String> {
            self.ensure_ready()?;
            let profile = Profile::new(name, version_id, memory_mb)?;
            let id = profile.id.clone();
            self.profiles.add(profile)?;
            if let Err(e) = self.save_profiles() {
                // Keep memory and disk consistent if persisting failed.
                let _ = self.profiles.remove(&id);
                return Err(e);
            }
            Ok(id)
        }

        /// Returns the printable JVM command for `profile_id`.
        pub fn launch_command(&self, profile_id: &str) -> Result<String> {
            self.ensure_ready()?;
            let profile = self.profiles.get(profile_id)?;
            if version::classify(&profile.version_id)? == version::VersionKind::Custom {
                return Err(LauncherError::InvalidArgument(
                    "only vanilla releases and snapshots can be launched".into(),
                ));
            }
            let game_dir = self.game_dir();
            let spec = LaunchSpec {
                java_bin: "java",
                game_dir: &game_dir,
                username: &self.username,
            };
            Ok(launch::to_command_line(&launch::build_command(
                profile, &spec,
            )))
        }

        /// Multi-line, human-readable status shown by the Android UI.
        pub fn status_report(&self) -> String {
            let mut out = String::new();
            out.push_str(&format!("{ENGINE_NAME} v{ENGINE_VERSION}\n"));
            out.push_str(&format!("State:    {}\n", self.state.as_str()));
            out.push_str(&format!(
                "Platform: {} / {}\n",
                std::env::consts::OS,
                std::env::consts::ARCH
            ));
            out.push_str(&format!("Data dir: {}\n", self.root.display()));
            out.push_str(&format!("Profiles: {}\n", self.profiles.len()));
            out.push_str(&format!("Username: {}\n", self.username));
            for p in self.profiles.list() {
                let kind = version::classify(&p.version_id)
                    .map(|k| k.as_str())
                    .unwrap_or("invalid");
                out.push_str(&format!(
                    "  - {} [{}] MC {} ({kind}), {} MB\n",
                    p.name, p.id, p.version_id, p.memory_mb
                ));
            }
            out
        }

        /// Persists state and moves to [`EngineState::Stopped`]. Idempotent.
        pub fn shutdown(&mut self) -> Result<()> {
            if self.state == EngineState::Stopped {
                return Ok(());
            }
            self.save_profiles()?;
            self.state = EngineState::Stopped;
            Ok(())
        }
    }

    fn validate_username(username: &str) -> Result<String> {
        let username = username.trim();
        if !(3..=16).contains(&username.len())
            || !username
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            return Err(LauncherError::InvalidArgument(
                "username must be 3..=16 ASCII letters, digits, or underscores".into(),
            ));
        }
        Ok(username.to_owned())
    }

    // ---- process-wide instance used by the JNI layer --------------------------

    static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

    fn lock() -> MutexGuard<'static, Option<Engine>> {
        // A panic while holding the lock must not brick the engine forever.
        ENGINE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Starts the global engine (idempotent while running) and returns its status.
    pub fn global_init(data_dir: &str) -> Result<String> {
        let mut guard = lock();
        if let Some(engine) = guard.as_ref() {
            if engine.state() == EngineState::Ready {
                return Ok(engine.status_report());
            }
        }
        let engine = Engine::init(data_dir)?;
        let report = engine.status_report();
        *guard = Some(engine);
        Ok(report)
    }

    /// Runs `f` against the running global engine.
    pub fn with_engine<T>(f: impl FnOnce(&mut Engine) -> Result<T>) -> Result<T> {
        let mut guard = lock();
        match guard.as_mut() {
            Some(engine) if engine.state() == EngineState::Ready => f(engine),
            _ => Err(LauncherError::InvalidState("engine is not initialised")),
        }
    }

    /// Status text; also valid before init / after shutdown.
    pub fn global_status() -> String {
        match lock().as_ref() {
            Some(engine) => engine.status_report(),
            None => format!("{ENGINE_NAME} v{ENGINE_VERSION}\nState:    Uninitialized\n"),
        }
    }

    pub fn global_shutdown() -> Result<()> {
        match lock().as_mut() {
            Some(engine) => engine.shutdown(),
            None => Ok(()),
        }
    }
}

// ---------------------------------------------------------------------------
// JNI bridge
// ---------------------------------------------------------------------------
//
// Symbol names follow `Java_<package>_<class>_<method>` for
// `com.voidkernel.rustcraftlauncher.MainActivity`. Keep them in sync with the
// `native` declarations in MainActivity.java.
pub mod jni_bridge {
    #![allow(non_snake_case)]

    use crate::engine;
    use crate::error::{LauncherError, Result};
    use crate::ENGINE_VERSION;
    use jni::objects::{JObject, JString};
    use jni::sys::{jint, jstring};
    use jni::JNIEnv;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::ptr;

    const ILLEGAL_STATE: &str = "java/lang/IllegalStateException";

    fn read_string(env: &mut JNIEnv, s: &JString) -> Result<String> {
        env.get_string(s)
            .map(String::from)
            .map_err(|e| LauncherError::Jni(e.to_string()))
    }

    fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
        if let Some(s) = payload.downcast_ref::<&str>() {
            (*s).to_owned()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_owned()
        }
    }

    /// Runs `f`, converting both `Err` and panics into a pending Java exception.
    /// Unwinding across the FFI boundary is undefined behaviour, so every JNI entry
    /// point goes through here.
    fn guarded<T>(env: &mut JNIEnv, f: impl FnOnce(&mut JNIEnv) -> Result<T>) -> Option<T> {
        let outcome = catch_unwind(AssertUnwindSafe(|| f(&mut *env)));
        let result = match outcome {
            Ok(r) => r,
            Err(payload) => Err(LauncherError::Panic(panic_message(payload))),
        };
        match result {
            Ok(v) => Some(v),
            Err(e) => {
                // If an exception is already pending, leave it alone.
                if !env.exception_check().unwrap_or(false) {
                    let _ = env.throw_new(ILLEGAL_STATE, e.to_string());
                }
                None
            }
        }
    }

    fn string_result(env: &mut JNIEnv, value: Option<String>) -> jstring {
        let Some(text) = value else {
            return ptr::null_mut();
        };
        match env.new_string(text) {
            Ok(s) => s.into_raw(),
            Err(e) => {
                let _ = env.throw_new(ILLEGAL_STATE, LauncherError::Jni(e.to_string()).to_string());
                ptr::null_mut()
            }
        }
    }

    /// `String nativeInit(String dataDir)` — starts the engine, returns its status.
    #[no_mangle]
    pub extern "system" fn Java_com_voidkernel_rustcraftlauncher_MainActivity_nativeInit<'local>(
        mut env: JNIEnv<'local>,
        _this: JObject<'local>,
        data_dir: JString<'local>,
    ) -> jstring {
        let out = guarded(&mut env, |env| {
            let dir = read_string(env, &data_dir)?;
            engine::global_init(&dir)
        });
        string_result(&mut env, out)
    }

    /// `String nativeGetStatus()` — current engine status text.
    #[no_mangle]
    pub extern "system" fn Java_com_voidkernel_rustcraftlauncher_MainActivity_nativeGetStatus<
        'local,
    >(
        mut env: JNIEnv<'local>,
        _this: JObject<'local>,
    ) -> jstring {
        let out = guarded(&mut env, |_| Ok(engine::global_status()));
        string_result(&mut env, out)
    }

    /// `String nativeGetVersion()` — engine crate version.
    #[no_mangle]
    pub extern "system" fn Java_com_voidkernel_rustcraftlauncher_MainActivity_nativeGetVersion<
        'local,
    >(
        mut env: JNIEnv<'local>,
        _this: JObject<'local>,
    ) -> jstring {
        let out = guarded(&mut env, |_| Ok(ENGINE_VERSION.to_owned()));
        string_result(&mut env, out)
    }

    /// `String nativeGetUsername()` — the persisted offline username.
    #[no_mangle]
    pub extern "system" fn Java_com_voidkernel_rustcraftlauncher_MainActivity_nativeGetUsername<
        'local,
    >(
        mut env: JNIEnv<'local>,
        _this: JObject<'local>,
    ) -> jstring {
        let out = guarded(&mut env, |_| {
            engine::with_engine(|engine| Ok(engine.username().to_owned()))
        });
        string_result(&mut env, out)
    }

    /// `void nativeSetUsername(String username)` — validates and persists the username.
    #[no_mangle]
    pub extern "system" fn Java_com_voidkernel_rustcraftlauncher_MainActivity_nativeSetUsername<
        'local,
    >(
        mut env: JNIEnv<'local>,
        _this: JObject<'local>,
        username: JString<'local>,
    ) {
        let _ = guarded(&mut env, |env| {
            let username = read_string(env, &username)?;
            engine::with_engine(|engine| engine.set_username(&username))
        });
    }

    /// `String nativeCreateProfile(String name, String versionId, int memoryMb)` —
    /// returns the new profile id.
    #[no_mangle]
    pub extern "system" fn Java_com_voidkernel_rustcraftlauncher_MainActivity_nativeCreateProfile<
        'local,
    >(
        mut env: JNIEnv<'local>,
        _this: JObject<'local>,
        name: JString<'local>,
        version_id: JString<'local>,
        memory_mb: jint,
    ) -> jstring {
        let out = guarded(&mut env, |env| {
            let name = read_string(env, &name)?;
            let version_id = read_string(env, &version_id)?;
            let memory = u32::try_from(memory_mb).map_err(|_| {
                LauncherError::InvalidArgument(format!("memory must not be negative: {memory_mb}"))
            })?;
            engine::with_engine(|e| e.create_profile(&name, &version_id, memory))
        });
        string_result(&mut env, out)
    }

    /// `String nativeGetLaunchCommand(String profileId)` — printable JVM command line.
    #[no_mangle]
    pub extern "system" fn Java_com_voidkernel_rustcraftlauncher_MainActivity_nativeGetLaunchCommand<
        'local,
    >(
        mut env: JNIEnv<'local>,
        _this: JObject<'local>,
        profile_id: JString<'local>,
    ) -> jstring {
        let out = guarded(&mut env, |env| {
            let id = read_string(env, &profile_id)?;
            engine::with_engine(|e| e.launch_command(&id))
        });
        string_result(&mut env, out)
    }

    /// `void nativeShutdown()` — persists state and stops the engine.
    #[no_mangle]
    pub extern "system" fn Java_com_voidkernel_rustcraftlauncher_MainActivity_nativeShutdown<
        'local,
    >(
        mut env: JNIEnv<'local>,
        _this: JObject<'local>,
    ) {
        let _ = guarded(&mut env, |_| engine::global_shutdown());
    }
}

// ---------------------------------------------------------------------------
// Tests (run on the host with `cargo test`)
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::engine::{Engine, EngineState};
    use super::error::LauncherError;
    use super::launch::{build_command, to_command_line, LaunchSpec};
    use super::profile::{slugify, Profile, ProfileStore};
    use super::version::{classify, VersionKind};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn temp_dir(tag: &str) -> PathBuf {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "rustcraft-test-{}-{}-{}",
            tag,
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::SeqCst)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn classifies_versions() {
        assert_eq!(classify("1.20.4").unwrap(), VersionKind::Release);
        assert_eq!(classify("1.8.9").unwrap(), VersionKind::Release);
        assert_eq!(classify("24w14a").unwrap(), VersionKind::Snapshot);
        assert_eq!(
            classify("fabric-loader-0.15.7").unwrap(),
            VersionKind::Custom
        );
    }

    #[test]
    fn rejects_bad_versions() {
        for bad in [
            "",
            "-1.20",
            "1..20",
            "../etc",
            "1.20/4",
            "a b",
            &"x".repeat(65),
        ] {
            assert!(classify(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn slugifies_names() {
        assert_eq!(slugify("My Pack!"), "my-pack");
        assert_eq!(slugify("  --A  b--  "), "a-b");
        assert_eq!(slugify("***"), "");
    }

    #[test]
    fn validates_profiles() {
        assert!(Profile::new("Survival", "1.20.4", 2048).is_ok());
        assert!(Profile::new("", "1.20.4", 2048).is_err());
        assert!(Profile::new("***", "1.20.4", 2048).is_err());
        assert!(Profile::new("x", "1.20.4", 128).is_err());
        assert!(Profile::new("x", "1.20.4", 99_999).is_err());
        assert!(Profile::new("x", "bad/version", 2048).is_err());
        assert!(Profile::new("tab\tname", "1.20.4", 2048).is_err());
    }

    #[test]
    fn store_round_trips_and_rejects_duplicates() {
        let mut store = ProfileStore::new();
        store
            .add(Profile::new("A One", "1.8.9", 1024).unwrap())
            .unwrap();
        store
            .add(Profile::new("B", "24w14a", 4096).unwrap())
            .unwrap();
        assert!(matches!(
            store.add(Profile::new("a one", "1.20.4", 2048).unwrap()),
            Err(LauncherError::AlreadyExists(_))
        ));
        let parsed = ProfileStore::from_text(&store.to_text()).unwrap();
        assert_eq!(parsed.list(), store.list());
    }

    #[test]
    fn store_rejects_corrupt_text() {
        assert!(ProfileStore::from_text("garbage").is_err());
        assert!(ProfileStore::from_text("# rustcraft profiles v1\nonly\ttwo\n").is_err());
        assert!(ProfileStore::from_text("# rustcraft profiles v1\nx\tX\t1.20.4\tabc\n").is_err());
        assert!(
            ProfileStore::from_text("# rustcraft profiles v1\nwrong\tX\t1.20.4\t1024\n").is_err()
        );
    }

    #[test]
    fn builds_launch_command() {
        let p = Profile::new("Test", "1.20.4", 3072).unwrap();
        let spec = LaunchSpec {
            java_bin: "java",
            game_dir: Path::new("/data/game dir"),
            username: "Steve",
        };
        let args = build_command(&p, &spec);
        assert_eq!(args[0], "java");
        assert!(args.contains(&"-Xmx3072M".to_owned()));
        assert!(args.contains(&"-Xms512M".to_owned()));
        let line = to_command_line(&args);
        assert!(line.contains("\"/data/game dir\""));
        assert!(line.ends_with("--username Steve"));
    }

    #[test]
    fn engine_lifecycle_and_persistence() {
        let dir = temp_dir("lifecycle");
        let dir_str = dir.to_str().unwrap();

        let mut engine = Engine::init(dir_str).unwrap();
        assert_eq!(engine.state(), EngineState::Ready);
        assert_eq!(engine.profiles().len(), 1);
        assert_eq!(engine.username(), "Player");
        engine.set_username("Alex_2").unwrap();
        assert!(engine
            .launch_command("default")
            .unwrap()
            .ends_with("--username Alex_2"));
        let id = engine.create_profile("Survival", "1.20.4", 4096).unwrap();
        assert_eq!(id, "survival");
        assert!(engine
            .launch_command("survival")
            .unwrap()
            .contains("-Xmx4096M"));
        let custom = engine
            .create_profile("Custom", "fabric-loader-0.15.7", 2048)
            .unwrap();
        assert!(matches!(
            engine.launch_command(&custom),
            Err(LauncherError::InvalidArgument(_))
        ));
        assert!(matches!(
            engine.launch_command("nope"),
            Err(LauncherError::NotFound(_))
        ));
        assert!(engine.status_report().contains("Profiles: 3"));

        engine.shutdown().unwrap();
        engine.shutdown().unwrap(); // idempotent
        assert_eq!(engine.state(), EngineState::Stopped);
        assert!(matches!(
            engine.create_profile("Late", "1.20.4", 2048),
            Err(LauncherError::InvalidState(_))
        ));

        // A fresh engine reloads what was saved.
        let reloaded = Engine::init(dir_str).unwrap();
        assert_eq!(reloaded.profiles().len(), 3);
        assert_eq!(reloaded.username(), "Alex_2");
        assert!(reloaded.profiles().get("survival").is_ok());
        assert!(matches!(
            reloaded.launch_command("custom"),
            Err(LauncherError::InvalidArgument(_))
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_invalid_usernames_without_replacing_saved_value() {
        let dir = temp_dir("username");
        let mut engine = Engine::init(dir.to_str().unwrap()).unwrap();

        for invalid in [
            "",
            "Al",
            "this_username_is_too_long",
            "Alex Smith",
            "Alex-2",
            "é",
        ] {
            assert!(engine.set_username(invalid).is_err(), "accepted {invalid:?}");
        }
        assert_eq!(engine.username(), "Player");
        assert_eq!(
            Engine::init(dir.to_str().unwrap()).unwrap().username(),
            "Player"
        );
        engine.set_username("abc").unwrap();
        assert_eq!(engine.username(), "abc");
        engine.set_username("A123456789012345").unwrap();
        assert_eq!(engine.username(), "A123456789012345");
    }

    #[test]
    fn init_rejects_empty_dir() {
        assert!(matches!(
            Engine::init("   "),
            Err(LauncherError::InvalidArgument(_))
        ));
    }
}
