use crate::{Affine, Point, WidgetMut, WindowId, World, passes};

pub(crate) fn compose_window(world: &mut World, window: WindowId) {
    let window_id = window;

    let Some(window) = world.window(window_id) else {
        return;
    };

    let scale = window.scale();

    for i in 0..window.layers.len() {
        // compose the layers widget
        if let Some(window) = world.window(window_id)
            && let Some(layer) = window.layers().get(i)
        {
            // translate the layer to it's position
            let transform = Affine::translate(layer.position - Point::ORIGIN);

            if let Some(mut widget) = world.widget_mut(layer.widget) {
                compose_widget(&mut widget, transform, scale)
            }
        }
    }
}

pub(crate) fn compose_widget(widget: &mut WidgetMut<'_>, transform: Affine, scale: f32) {
    if widget.cx.is_stashed() {
        return;
    }

    let global_transform = transform * widget.cx.state.transform;

    if !widget.cx.hierarchy.needs_compose() && widget.cx.state.global_transform == global_transform
    {
        return;
    }

    let _span = widget.cx.enter_span();
    widget.cx.hierarchy.mark_composed();

    widget.cx.state.global_transform = global_transform;

    let mut cx = widget.cx.as_compose_cx(scale);
    widget.widget.compose(&mut cx);

    widget.cx.for_each_child_mut(|child| {
        compose_widget(child, global_transform, scale);
    });

    passes::hierarchy::update_flags(widget);
}

pub(crate) fn place_widget(widget: &mut WidgetMut<'_>, mut transform: Affine, scale: f32) {
    if !widget.cx.is_subpixel() {
        transform.offset = transform.offset.pixel_align(scale);
    }

    if widget.cx.state.transform != transform {
        widget.cx.state.transform = transform;
        widget.cx.hierarchy.request_draw();
    }
}
