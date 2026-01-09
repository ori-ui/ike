package org.ori;

import android.app.NativeActivity;
import android.os.Bundle;
import android.view.WindowInsets;
import android.graphics.Insets;

public final class RustActivity extends NativeActivity {
    RustView rustView;

    @Override
    protected void onCreate(Bundle savedInstanceData) {
        super.onCreate(savedInstanceData);

        rustView = new RustView(this);
        setContentView(rustView);
    }

    @Override
    protected void onResume() {
        super.onResume();

        getWindow().getDecorView().post(this::attachInsetListener);
    }

    private void attachInsetListener() {
        getWindow().setDecorFitsSystemWindows(false);

        getWindow().getDecorView().setOnApplyWindowInsetsListener((v, insets) -> {
            Insets systemBars = insets.getInsets(WindowInsets.Type.systemBars());
            Insets ime = insets.getInsets(WindowInsets.Type.ime());
            Insets cutout = insets.getInsets(WindowInsets.Type.displayCutout());

            rustView.onApplyWindowInsetsNative(
                    systemBars.left,
                    systemBars.top,
                    systemBars.right,
                    systemBars.bottom,
                    ime.left,
                    ime.top,
                    ime.right,
                    ime.bottom,
                    cutout.left,
                    cutout.top,
                    cutout.right,
                    cutout.bottom);

            return insets;
        });
    }
}
