use std::time::Duration;

use crate::{WidgetMut, WindowId, World, passes};

pub(crate) fn animate_window(world: &mut World, window: WindowId, delta_time: Duration) {
    let Some(window) = world.window(window) else {
        return;
    };

    for layer in window.layers.clone().iter() {
        if let Some(mut widget) = world.widget_mut(layer.widget) {
            animate_widget_recursive(&mut widget, delta_time);
        }
    }
}

pub(crate) fn animate_widget_recursive(widget: &mut WidgetMut<'_>, delta_time: Duration) {
    if !widget.cx.hierarchy.needs_animate() || widget.cx.is_stashed() {
        return;
    }

    widget.cx.for_each_child_mut(|child| {
        animate_widget_recursive(child, delta_time);
    });

    passes::propagate::update_hirarchy(widget);

    let mut cx = widget.cx.as_update_cx();
    widget.widget.animate(&mut cx, delta_time);
}
