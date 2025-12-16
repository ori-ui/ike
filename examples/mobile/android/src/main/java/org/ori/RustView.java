package org.ori;

import android.content.Context;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.view.inputmethod.InputMethodManager;
import android.text.InputType;

public final class RustView extends View {
    final InputMethodManager inputMethodManager;

    public RustView(Context context) {
        super(context);
        setFocusable(true);
        setFocusableInTouchMode(true);
        requestFocus();

        inputMethodManager = (InputMethodManager) context.getSystemService(Context.INPUT_METHOD_SERVICE);
    }

    @Override
    public InputConnection onCreateInputConnection(EditorInfo outAttrs) {
        return new RustInputConnection(this);
    }
}
