use crate::{BuildCx, Rect, Root, Update, WidgetId};

pub(crate) fn scroll_to(root: &mut Root, target: WidgetId, mut rect: Rect) {
    let mut current = Some(target);

    while let Some(id) = current {
        if let Some(mut widget) = root.get_mut(id) {
            widget.update_without_propagate(Update::ScrollTo(rect));
            rect.min = widget.cx.transform() * rect.min;
            rect.max = widget.cx.transform() * rect.max;

            current = widget.cx.parent();
        }
    }

    if let Some(mut target) = root.get_mut(target) {
        target.cx.propagate_state();
    }
}
