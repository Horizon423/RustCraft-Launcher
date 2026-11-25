🦀

# RustCraft Launcher Edition v2.5

# Rust-Powered Android Launcher Shell

<p align="center">
  Performance | Stability | Innovation | Open Source
</p><p align="center">
  <img src="https://img.shields.io/badge/Platform-Android-3DDC84?style=for-the-badge&logo=android&logoColor=white">
  <img src="https://img.shields.io/badge/Engine-Rust-orange?style=for-the-badge&logo=rust&logoColor=white">
  <img src="https://img.shields.io/badge/Architecture-ARM64-blue?style=for-the-badge">
  <img src="https://img.shields.io/badge/License-Apache%202.0-purple?style=for-the-badge">
</p>---

## About

RustCraft Launcher is an independent Android application with a Java Android shell
and a Rust native engine. Version 2.5 initializes the engine, stores launcher
profiles and an offline username, and can construct a vanilla Minecraft JVM
command line.

This repository does not currently contain a Minecraft client, Android Minecraft
main menu, mod installation/import/launch functionality, Minecraft authentication,
or a 3D skin renderer. The username setting is an offline launch argument; it is
not an authenticated Minecraft account profile.

👨‍💻 Development Team

🏆 Main Developer

**Horizon423**

Founder & Main Developer of RustCraft Launcher.

Responsible for the project's overall development direction, customization, integration, branding, and launcher development.

🤝 Upstream Project Attribution

The separate Horizon Android project is maintained by **[Ludovic Marie and the
Horizon contributors](https://github.com/MocLG/horizon-android)**. The current
RustCraft source tree does not include Horizon's game engine or client UI.


## Implemented

- Android application shell backed by the Rust JNI engine.
- Persistent local launcher profiles and a Minecraft-format offline username
  (3-16 ASCII letters, digits, or underscores).
- Generated vanilla release/snapshot JVM command lines using that username.
- Native Android builds for ARM64 and ARMv7.

## Partially Implemented

- Profiles and launch commands are stored/constructed, but the app does not
  download Minecraft files, authenticate accounts, or start a Minecraft process.
- The username is local/offline only; it is not synchronized with an authenticated
  Minecraft account or an in-game profile.

## Unsupported

- Minecraft client and in-game main menu, including branding and options screens.
- Minecraft skin retrieval, rendering, and 3D player preview.
- World and resource-pack management.
- Mod installation, import, and launching. Custom version profiles can remain
  stored for compatibility but the engine refuses to generate their launch commands.

## v2.5 Release Downloads

- Public v2.5 APK: [RustCraft-Launcher-v2.5.apk](https://github.com/Horizon423/RustCraft-Launcher/releases/download/v2.5/RustCraft-Launcher-v2.5.apk).
- Previous working GUI reference: [RustCraft.Launcher-1.2.0-reference.apk](https://github.com/Horizon423/RustCraft-Launcher/releases/download/v2.5/RustCraft.Launcher-1.2.0-reference.apk).

The previous 1.2.0 APK is provided only as an original-GUI reference. It is not a v2.5 implementation and has not been modified or rebuilt in this repository.

## v2.5 Release Notes

- Updated Android and native engine version metadata to 2.5.
- Added verified open-source attribution and license notices.
- Added a saved, validated offline username setting (3-16 ASCII letters, digits,
  or underscores) used by generated vanilla launch commands.
- Restricted generated launch commands to vanilla releases and snapshots; custom
  version profiles remain stored but cannot be launched.
- No Mods menu or mod installation/import pipeline exists in this repository.
- A Minecraft game client, in-game menu, authentication, and 3D skin preview are
  not included in the current architecture.
- Because the Minecraft client and Options screen are not included, this build
  does not alter or verify an in-game Horizon Credits entry.

📱 Project Information

Information| Details
Project Name| RustCraft Launcher
Version| 2.5
Main Developer| Horizon423
Platform| Android
Architecture| ARM64 ("arm64-v8a")
Game Integration| JVM command construction only
Native Technology| Rust
Application Format| APK
Project Type| Independent Open-Source Project
License| Apache License 2.0

🏗️ Project Structure
```bash
RustCraft-Launcher/
├── app/
│   ├── build.gradle                  Android + cargo-ndk integration
│   └── src/main/
│       ├── AndroidManifest.xml
│       ├── java/com/voidkernel/rustcraftlauncher/MainActivity.java
│       ├── jniLibs/                  generated .so files (git-ignored)
│       └── res/values/               strings, colors, theme
├── rust/
│   ├── Cargo.toml
│   └── src/lib.rs                    the engine
├── gradle/wrapper/                   gradle-wrapper.jar + .properties
├── build.gradle
├── settings.gradle
├── gradle.properties
├── gradlew / gradlew.bat
├── .gitmodules
├── .gitignore
├── README.md
└── LICENSE
```

## Build Requirements

The Android build was verified with JDK 17, Gradle 8.9, Android SDK Platform 34,
Build Tools 34.0.0, NDK 26.1.10909125, Rust stable, and `cargo-ndk`. Java 25 is
not compatible with this repository's Gradle/Groovy setup. Install the Android
SDK components using Android Studio's SDK Manager or `sdkmanager`:

```sh
sdkmanager "platforms;android-34" "build-tools;34.0.0" "ndk;26.1.10909125"
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-ndk --locked
```

Set paths for the installed tools (adjust them for your machine), then run tests
and builds:

```sh
export JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
export ANDROID_HOME="$HOME/Android/Sdk"
export ANDROID_SDK_ROOT="$ANDROID_HOME"
export ANDROID_NDK_HOME="$ANDROID_HOME/ndk/26.1.10909125"

cargo test --manifest-path rust/Cargo.toml
./gradlew :app:assembleDebug :app:assembleRelease
```

The release build currently produces an **unsigned** APK at
`app/build/outputs/apk/release/app-release-unsigned.apk`. Configure a private
release signing key before distributing an installable release; do not use a
debug keystore for public releases.


## Credits & Open-Source Licenses

- The repository-level license is Apache License 2.0. Its complete text is in
  [LICENSE](./LICENSE).
- [Horizon Android](https://github.com/MocLG/horizon-android) is an independent
  upstream project. Its `LICENSE` credits “Ludovic Marie and the Horizon
  contributors” and grants the MIT License; the complete notice and permission
  text are preserved in [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md).
  Horizon source and its game client are not included in this RustCraft source
  tree.
- The Rust JNI dependency `jni` 0.21.1 is dual-licensed MIT or Apache-2.0.
  Its verified MIT notice is in [THIRD_PARTY_NOTICES.md](./THIRD_PARTY_NOTICES.md);
  its Apache-2.0 terms are available in [LICENSE](./LICENSE). Other direct and
  transitive Rust dependencies retain the licenses declared by their upstream
  package manifests and recorded in [Cargo.lock](./rust/Cargo.lock).
- `rust/Cargo.toml` declares the RustCraft crate as MIT, while the repository's
  `LICENSE` is Apache-2.0. No separate RustCraft MIT license text or copyright
  notice is present in this checkout, so that metadata conflict remains
  unresolved; the `authors` field is not treated as a copyright notice.
- No license called “LP2.0” is present in this repository or identified in
  Horizon's upstream notices. Horizon's notices separately identify GPL-2.0
  with the Classpath Exception for the OpenJDK 25 runtime distributed by
  Horizon's Android build; that runtime is not part of this RustCraft source or
  APK build.
- Minecraft/Mojang/Microsoft trademarks and proprietary game materials are
  separate from these open-source licenses and are not covered by them.

⚠️ Disclaimer

RustCraft Launcher is an independent and unofficial third-party project.

- Not affiliated with, endorsed by, or sponsored by Mojang Studios or Microsoft.
- Minecraft is a trademark of Mojang Studios.
- RustCraft Launcher does not claim ownership of Minecraft or its proprietary assets.
- Users must obtain Minecraft through legitimate channels.
- Third-party libraries and components remain subject to their respective licenses.

📜 License

See [LICENSE](./LICENSE) and
[Credits & Open-Source Licenses](#credits--open-source-licenses) for the
repository license and third-party licensing details.

🤝 Contributions

Contributions, suggestions, bug reports, and improvements are welcome.

We appreciate everyone who helps improve RustCraft Launcher and its ecosystem.

---

<p align="center">🦀 RustCraft Launcher

Built for Android. Powered by Rust.

Developed by Horizon423

Independent Development • Native Technology • Open Source Spirit

</p>
