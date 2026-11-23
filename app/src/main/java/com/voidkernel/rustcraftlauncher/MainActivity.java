package com.voidkernel.rustcraftlauncher;

import android.app.Activity;
import android.graphics.Typeface;
import android.os.Bundle;
import android.util.TypedValue;
import android.view.Gravity;
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

    private native String nativeCreateProfile(String name, String versionId, int memoryMb);

    private native String nativeGetLaunchCommand(String profileId);

    private native void nativeShutdown();

    private TextView statusView;

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
        setContentView(scroll);
    }
}
