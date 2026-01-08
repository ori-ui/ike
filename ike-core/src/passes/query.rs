use crate::{Point, WidgetId, Window, World};

pub(crate) fn find_widget_at(world: &World, window: &Window, position: Point) -> Option<WidgetId> {
    for layer in window.layers.clone().iter().rev() {
        if let Some(root) = world.widget(layer.widget)
            && let Some(root) = root.widget.find_widget_at(&root.cx, position)
        {
            return Some(root);
        }
    }

    None
}
