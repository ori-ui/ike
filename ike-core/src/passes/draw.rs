use crate::{Canvas, WidgetMut, WindowId, World};

pub(crate) fn draw_window(world: &mut World, window: WindowId, canvas: &mut dyn Canvas) {
    let Some(window) = world.window(window) else {
        return;
    };

    let scale = window.scale();

    for layer in window.layers.clone().iter() {
        if let Ok(mut widget) = world.widget_mut(layer.widget) {
            draw_widget(&mut widget, canvas, scale);
        }
    }
}

pub(crate) fn draw_widget(widget: &mut WidgetMut<'_>, canvas: &mut dyn Canvas, scale: f32) {
    if widget.cx.is_stashed() {
        return;
    }

    let _span = widget.cx.enter_span();

    widget.cx.state.stable_draws += 1;

    if widget.cx.hierarchy.needs_draw() {
        widget.cx.state.stable_draws = 0;
        widget.cx.world.recorder.remove(widget.id());
        widget.cx.hierarchy.mark_drawn();
    }

    if let Some(recording) = widget.cx.world.recorder.get_recording_marked(widget.id()) {
        canvas.transform(
            widget.cx.transform() * widget.cx.global_transform().inverse(),
            &mut |canvas| {
                canvas.draw_recording(widget.cx.state.bounds, &recording);
            },
        );
    } else {
        let _span = widget.cx.enter_span();
        canvas.transform(widget.cx.transform(), &mut |canvas| {
            draw_widget_clipped(widget, canvas, scale);
        });
    }
}

pub(crate) fn draw_widget_clipped(widget: &mut WidgetMut<'_>, canvas: &mut dyn Canvas, scale: f32) {
    if let Some(ref clip) = widget.cx.state.clip {
        canvas.clip(&clip.clone(), &mut |canvas| {
            draw_widget_raw(widget, canvas, scale);
        });
    } else {
        draw_widget_raw(widget, canvas, scale);
    }
}

pub(crate) fn draw_widget_raw(widget: &mut WidgetMut<'_>, canvas: &mut dyn Canvas, scale: f32) {
    widget.widget.draw(&mut widget.cx.as_draw_cx(), canvas);

    for &child in widget.cx.hierarchy.children.iter() {
        if let Ok(mut child) = widget.cx.get_widget_mut(child) {
            draw_widget(&mut child, canvas, scale);
        }
    }

    widget.widget.draw_over(&mut widget.cx.as_draw_cx(), canvas);
}
