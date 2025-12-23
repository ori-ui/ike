use crate::{Rect, Update, WidgetId, World};

pub(crate) fn scroll_to(world: &mut World, target: WidgetId, mut rect: Rect) {
    world.handle_updates();

    let mut current = Some(target);

    while let Some(id) = current {
        if let Some(mut widget) = world.widget_mut(id) {
            widget.update_without_propagate(Update::ScrollTo(rect));
            rect.min = widget.cx.transform() * rect.min;
            rect.max = widget.cx.transform() * rect.max;

            current = widget.cx.parent();
        }
    }

    world.propagate_state(target);
}
