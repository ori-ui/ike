use crate::{BuildCx, Point, WidgetId, Window, World};

pub fn find_widget_at(world: &World, window: &Window, point: Point) -> Option<WidgetId> {
    for layer in window.layers.clone().iter().rev() {
        if let Some(widget) = world.get_widget(layer.root)
            && let Some(widget) = widget.widget.find_widget_at(&widget.cx, point)
        {
            return Some(widget);
        }
    }

    None
}
