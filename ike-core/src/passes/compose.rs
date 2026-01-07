use crate::{Affine, WidgetMut, WindowId, World, passes};

pub(crate) fn compose_window(world: &mut World, window: WindowId) {
    let window_id = window;

    let Some(window) = world.window(window_id) else {
        return;
    };

    let scale = window.scale();

    for i in 0..window.layers.len() {
        // compute the size of the layers widget
        if let Some(window) = world.window(window_id)
            && let Some(layer) = window.layers().get(i)
            && let Some(mut widget) = world.widget_mut(layer.widget)
        {
            compose_widget(&mut widget, Affine::IDENTITY, scale)
        }
    }
}

pub(crate) fn compose_widget(widget: &mut WidgetMut<'_>, transform: Affine, scale: f32) {
    if widget.cx.is_stashed() {
        return;
    }

    let _span = widget.cx.enter_span();

    let global_transform = transform * widget.cx.state.transform;

    if !widget.cx.hierarchy.needs_compose() && widget.cx.state.global_transform == global_transform
    {
        return;
    }

    widget.cx.state.global_transform = global_transform;
    widget.cx.hierarchy.mark_composed();

    let mut cx = widget.cx.as_compose_cx(scale);
    widget.widget.compose(&mut cx);

    widget.cx.for_each_child_mut(|child| {
        compose_widget(child, global_transform, scale);
    });

    passes::propagate::update_hirarchy(widget);
}

pub(crate) fn place_widget(widget: &mut WidgetMut<'_>, mut transform: Affine, scale: f32) {
    if widget.cx.state.is_pixel_perfect {
        transform.offset = transform.offset.round_to_scale(scale);
    }

    if widget.cx.state.transform != transform {
        widget.cx.state.transform = transform;
    }
}
