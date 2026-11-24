package com.voidkernel.rustcraftlauncher;

import android.app.Activity;
import android.app.AlertDialog;
import android.graphics.Typeface;
import android.os.Bundle;
import android.text.InputType;
import android.util.TypedValue;
import android.view.Gravity;
import android.widget.Button;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ScrollView;
import android.widget.TextView;

/**
 * Thin Android shell for the RustCraft engine.
 *
 * All launcher logic lives in Rust (rust/src/lib.rs). This class only loads the
 * native library, forwards lifecycle events and shows what the engine reports.
 */
public class MainActivity extends Activity {

    /** Non-null when System.loadLibrary failed; shown instead of crashing. */
    private static final String LOAD_ERROR;

    static {
        String error = null;
        try {
            System.loadLibrary("rustcraft_launcher");
        } catch (UnsatisfiedLinkError | SecurityException e) {
            error = e.getMessage();
        }
        LOAD_ERROR = error;
    }

    // JNI declarations. Each matches a Java_com_voidkernel_rustcraftlauncher_MainActivity_*
    // function in rust/src/lib.rs. Native errors surface as IllegalStateException.
    private native String nativeInit(String dataDir);

    private native String nativeGetStatus();

    private native String nativeGetVersion();

    private native String nativeGetUsername();

    private native void nativeSetUsername(String username);

    private native String nativeCreateProfile(String name, String versionId, int memoryMb);

    private native String nativeGetLaunchCommand(String profileId);

    private native void nativeShutdown();

    private TextView statusView;
    private Button editUsernameButton;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        buildUi();
        showStatus();
    }

    @Override
    protected void onDestroy() {
        if (LOAD_ERROR == null && isFinishing()) {
            try {
                nativeShutdown();
            } catch (RuntimeException | UnsatisfiedLinkError ignored) {
                // Nothing useful to do while the activity is going away.
            }
        }
        super.onDestroy();
    }

    private void showStatus() {
        if (LOAD_ERROR != null) {
            showError(getString(R.string.engine_load_failed), LOAD_ERROR);
            return;
        }
        try {
            statusView.setText(nativeInit(getFilesDir().getAbsolutePath()));
            editUsernameButton.setEnabled(true);
        } catch (RuntimeException | UnsatisfiedLinkError e) {
            showError(getString(R.string.engine_load_failed), String.valueOf(e.getMessage()));
        }
    }

    private void showError(String title, String detail) {
        statusView.setTextColor(getResources().getColor(R.color.text_error, getTheme()));
        statusView.setText(title + "\n\n" + detail);
    }

    private void buildUi() {
        int pad = (int) TypedValue.applyDimension(
                TypedValue.COMPLEX_UNIT_DIP, 20, getResources().getDisplayMetrics());
        statusView = new TextView(this);
        statusView.setTypeface(Typeface.MONOSPACE);
        statusView.setTextSize(TypedValue.COMPLEX_UNIT_SP, 14);
        statusView.setTextColor(getResources().getColor(R.color.text_primary, getTheme()));
        statusView.setGravity(Gravity.START);
        statusView.setPadding(pad, pad, pad, pad);
        statusView.setText(R.string.engine_loading);

        ScrollView scroll = new ScrollView(this);
        scroll.setFillViewport(true);
        scroll.addView(statusView);

        editUsernameButton = new Button(this);
        editUsernameButton.setText(R.string.edit_username);
        editUsernameButton.setEnabled(false);
        editUsernameButton.setOnClickListener(view -> showUsernameDialog());

        LinearLayout layout = new LinearLayout(this);
        layout.setOrientation(LinearLayout.VERTICAL);
        layout.addView(scroll, new LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT, 0, 1));
        layout.addView(editUsernameButton);
        setContentView(layout);
    }

    private void showUsernameDialog() {
        EditText input = new EditText(this);
        input.setSingleLine(true);
        input.setInputType(InputType.TYPE_CLASS_TEXT | InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS);
        input.setHint(R.string.username_prompt);
        input.setText(nativeGetUsername());

        new AlertDialog.Builder(this)
                .setTitle(R.string.edit_username)
                .setView(input)
                .setNegativeButton(android.R.string.cancel, null)
                .setPositiveButton(android.R.string.ok, (dialog, which) -> {
                    try {
                        nativeSetUsername(input.getText().toString());
                        statusView.setText(nativeGetStatus());
                    } catch (RuntimeException | UnsatisfiedLinkError e) {
                        showError(getString(R.string.username_save_failed),
                                String.valueOf(e.getMessage()));
                    }
                })
                .show();
    }
}
