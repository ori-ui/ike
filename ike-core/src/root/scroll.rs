use crate::{BuildCx, Rect, Root, Update, WidgetId};

pub(crate) fn scroll_to(root: &mut Root, target: WidgetId, mut rect: Rect) {
    root.handle_updates();

    let mut current = Some(target);

    while let Some(id) = current {
        if let Some(mut widget) = root.get_widget_mut(id) {
            widget.update_without_propagate(Update::ScrollTo(rect));
            rect.min = widget.cx.transform() * rect.min;
            rect.max = widget.cx.transform() * rect.max;

            current = widget.cx.parent();
        }
    }

    root.propagate_state(target);
}
