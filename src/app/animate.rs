use std::time::Duration;

use crate::{Tree, UpdateCx, WidgetId};

pub(super) fn animate_widget(tree: &mut Tree, id: WidgetId, dt: Duration) {
    tree.with_entry(id, |tree, widget, state| {
        for &child in &state.children {
            animate_widget(tree, child, dt);
        }

        state.needs_animate = false;

        let mut cx = UpdateCx { tree, state };
        widget.animate(&mut cx, dt);
    });

    tree.update_state(id);
}
