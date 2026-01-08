use crate::{Canvas, WidgetMut, WindowId, World};

pub(crate) fn draw_window(world: &mut World, window: WindowId, canvas: &mut dyn Canvas) {
    let Some(window) = world.window(window) else {
        return;
    };

    for layer in window.layers.clone().iter() {
        let Some(mut widget) = world.widget_mut(layer.widget) else {
            return;
        };

        draw_widget(&mut widget, canvas);
    }
}

pub(crate) fn draw_widget(widget: &mut WidgetMut<'_>, canvas: &mut dyn Canvas) {
    if widget.cx.is_stashed() {
        return;
    }

    let _span = widget.cx.enter_span();

    widget.cx.state.stable_draws += 1;

    if widget.cx.hierarchy.needs_draw() {
        widget.cx.state.stable_draws = 0;
        widget.cx.world.recorder.remove(widget.id());
    }

    if let Some(recording) = widget.cx.world.recorder.get_marked(widget.id()) {
        canvas.transform(widget.cx.transform(), &mut |canvas| {
            canvas.draw_recording(&recording);
        })
    } else {
        let _span = widget.cx.enter_span();
        canvas.transform(widget.cx.transform(), &mut |canvas| {
            draw_widget_clipped(widget, canvas);
        });
    }

    canvas.transform(widget.cx.transform(), &mut |canvas| {
        widget.widget.draw_over(&mut widget.cx.as_draw_cx(), canvas);
    });
}

pub(crate) fn draw_widget_clipped(widget: &mut WidgetMut<'_>, canvas: &mut dyn Canvas) {
    if let Some(ref clip) = widget.cx.state.clip {
        canvas.clip(&clip.clone(), &mut |canvas| {
            draw_widget_raw(widget, canvas);
        });
    } else {
        draw_widget_raw(widget, canvas);
    }
}

pub(crate) fn draw_widget_raw(widget: &mut WidgetMut<'_>, canvas: &mut dyn Canvas) {
    widget.widget.draw(&mut widget.cx.as_draw_cx(), canvas);

    widget.cx.for_each_child_mut(|child| {
        draw_widget(child, canvas);
    });
}
