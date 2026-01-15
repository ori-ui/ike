use std::time::Duration;

use crate::{WidgetMut, WindowId, World, passes};

pub(crate) fn animate_window(world: &mut World, window: WindowId, delta_time: Duration) {
    let Some(window) = world.window(window) else {
        return;
    };

    for layer in window.layers.clone().iter() {
        if let Ok(widget) = world.widget_mut(layer.widget) {
            animate_widget_recursive(widget, delta_time);
        }
    }
}

pub(crate) fn animate_widget_recursive(mut widget: WidgetMut<'_>, delta_time: Duration) {
    if !widget.cx.hierarchy.needs_animate() || widget.cx.is_stashed() {
        return;
    }

    let _span = widget.cx.enter_span();
    widget.cx.hierarchy.mark_animated();

    let mut cx = widget.cx.as_update_cx();
    widget.widget.animate(&mut cx, delta_time);

    passes::hierarchy::for_each_child(widget, |child| {
        if let Ok(child) = child {
            animate_widget_recursive(child, delta_time);
        }
    });
}
