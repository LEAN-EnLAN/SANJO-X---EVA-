package tech.sanjox.eva;

import android.os.Build;
import android.view.View;
import android.view.WindowInsets;

/**
 * Android 15+ draws every app edge to edge (targetSdk 35+), ignoring the black status / navigation bar colours.
 * Padding the root by the system bar (and keyboard) insets keeps content out from under them; the black root
 * background then shows through as black bars, as on older versions.
 */
final class Insets {
    private Insets() {}

    static void applySystemBarPadding(View root) {
        if (Build.VERSION.SDK_INT < 35) return; // older versions lay out below the bars already
        root.setOnApplyWindowInsetsListener((v, insets) -> {
            android.graphics.Insets b = insets.getInsets(
                    WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout() | WindowInsets.Type.ime());
            v.setPadding(b.left, b.top, b.right, b.bottom);
            return WindowInsets.CONSUMED;
        });
        root.requestApplyInsets();
    }
}
