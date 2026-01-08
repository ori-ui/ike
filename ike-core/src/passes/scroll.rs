use crate::{Rect, Update, WidgetId, World, passes};

pub(crate) fn scroll_to(world: &mut World, target: WidgetId, mut rect: Rect) {
    let mut current = Some(target);

    while let Some(id) = current
        && let Some(mut widget) = world.widget_mut(id)
    {
        passes::update::widget(&mut widget, Update::ScrollTo(rect));
        rect.min = widget.cx.transform() * rect.min;
        rect.max = widget.cx.transform() * rect.max;

        current = widget.cx.parent();
    }
}
