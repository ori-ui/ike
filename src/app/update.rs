use crate::{Tree, Update, UpdateCx, WidgetId};

pub(super) fn update_widget(tree: &mut Tree, id: WidgetId, update: &Update) {
    tree.with_entry(id, |tree, _, state| {
        for &child in &state.children {
            update_widget(tree, child, update);
        }
    });

    tree.update_state(id);

    tree.with_entry(id, |tree, widget, state| {
        let mut cx = UpdateCx { tree, state };
        widget.update(&mut cx, update.clone());
    });
}
