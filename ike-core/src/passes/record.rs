use crate::{Canvas, WidgetMut, WindowId, World, passes};

pub(crate) fn record_window(world: &mut World, window: WindowId, canvas: &mut dyn Canvas) {
    let Some(window) = world.window(window) else {
        return;
    };

    let scale = window.scale();

    for layer in window.layers.clone().iter() {
        if let Ok(widget) = world.widget_mut(layer.widget) {
            record_widget(widget, canvas, scale);
        }
    }

    world.state.recorder.frame(
        &world.state.settings.record,
        &world.widgets,
    );
}

fn record_widget(mut widget: WidgetMut<'_>, canvas: &mut dyn Canvas, scale: f32) {
    if widget.cx.world.recorder.contains(widget.id()) {
        return;
    }

    let _span = widget.cx.enter_span();

    let bounds = widget.cx.state.bounds;
    let draw_cost = (bounds.size().area() * scale * scale).sqrt()
        + f32::min(
            widget.cx.state.stable_draws as f32 / 8.0,
            1.0,
        );

    let memory_estimate = (bounds.size().area() * scale * scale * 4.0) as u64;
    let total_memory_estimate = widget.cx.world.recorder.memory_usage() + memory_estimate;

    if draw_cost >= widget.cx.world.settings.record.cost_threshold
        && widget.cx.state.stable_draws >= 3
        && bounds.size().area() > 256.0
        && total_memory_estimate < widget.cx.world.settings.record.max_memory_usage
    {
        let rect = widget.cx.state.bounds.to_pixels(scale);
        let recording = canvas.record(rect, &mut |canvas| {
            canvas.transform(
                widget.cx.global_transform(),
                &mut |canvas| passes::draw::draw_widget_clipped(&mut widget, canvas, scale),
            );
        });

        if let Some(recording) = recording {
            (widget.cx.world.recorder).insert(widget.cx.id(), draw_cost, recording);
        }

        return;
    }

    widget.cx.world.recorder.remove(widget.id());

    passes::hierarchy::for_each_child(widget, |child| {
        if let Ok(child) = child {
            record_widget(child, canvas, scale);
        }
    });
}
