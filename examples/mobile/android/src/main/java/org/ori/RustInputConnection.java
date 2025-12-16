package org.ori;

import android.view.View;
import android.view.inputmethod.BaseInputConnection;
import android.view.inputmethod.InputConnection;
import android.view.KeyEvent;

public final class RustInputConnection extends BaseInputConnection {
    public RustInputConnection(View view) {
        super(view, true);
    }
}
