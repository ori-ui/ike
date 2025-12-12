use crate::{BuildCx, Rect, Root, Update, WidgetId};

pub(crate) fn scroll_to(root: &mut Root, target: WidgetId, mut rect: Rect) {
    let mut current = Some(target);

    while let Some(id) = current {
        let mut widget = root.get_mut(id).unwrap();

        widget.update_without_propagate(Update::ScrollTo(rect));
        rect.min = widget.transform() * rect.min;
        rect.max = widget.transform() * rect.max;

        current = widget.state().parent;
    }

    if let Some(mut target) = root.get_mut(target) {
        target.propagate_state();
    }
}
