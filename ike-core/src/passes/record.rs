use crate::{Canvas, WidgetMut, WindowId, World, passes};

pub(crate) fn record_window(world: &mut World, window: WindowId, canvas: &mut dyn Canvas) {
    let Some(window) = world.window(window) else {
        return;
    };

    for layer in window.layers.clone().iter() {
        let Some(mut widget) = world.widget_mut(layer.widget) else {
            return;
        };

        record_widget(&mut widget, canvas);
    }

    world.state.recorder.frame(&world.widgets);
}

fn record_widget(widget: &mut WidgetMut<'_>, canvas: &mut dyn Canvas) {
    if widget.cx.world.recorder.contains(widget.id()) {
        return;
    }

    let _span = widget.cx.enter_span();

    let scale = widget.cx.get_window().map_or(1.0, |x| x.scale);
    let draw_cost = (widget.cx.size().area() * scale * scale).sqrt()
        + f32::min(
            widget.cx.state.stable_draws as f32 / 8.0,
            1.0,
        );

    let memory_estimate = (widget.cx.size().area() * scale * scale * 4.0) as u64;
    let total_memory_estimate = widget.cx.world.recorder.memory_usage() + memory_estimate;

    if draw_cost >= widget.cx.world.recorder.cost_threshold
        && widget.cx.state.stable_draws >= 3
        && widget.cx.size().area() > 256.0
        && total_memory_estimate < widget.cx.world.recorder.max_memory_usage
    {
        tracing::trace!(
            widget = ?widget.cx.id(),
            cost = draw_cost,
            "recording",
        );

        let recording = canvas.record(widget.cx.size(), &mut |canvas| {
            passes::draw::draw_widget_clipped(widget, canvas);
        });

        if let Some(recording) = recording {
            (widget.cx.world.recorder).insert(widget.cx.id(), draw_cost, recording);
        }

        return;
    }

    widget.cx.world.recorder.remove(widget.id());

    widget.cx.for_each_child_mut(|child| {
        record_widget(child, canvas);
    });
}
