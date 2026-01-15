use crate::{Affine, ComposeCx, WidgetId, WidgetMut, WindowId, World, passes};

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
            && let Ok(widget) = world.widget_mut(layer.widget)
        {
            compose_widget(widget, Affine::IDENTITY, scale)
        }
    }
}

pub(crate) fn compose_widget(mut widget: WidgetMut<'_>, transform: Affine, scale: f32) {
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

    passes::hierarchy::for_each_child(widget, |child| {
        if let Ok(child) = child {
            compose_widget(child, global_transform, scale);
        }
    });
}

pub(crate) fn place_child(cx: &mut ComposeCx<'_>, child: WidgetId, mut transform: Affine) {
    let id = cx.id();
    if let Ok(mut child) = cx.widgets.get_mut(cx.world, child) {
        debug_assert!(child.cx.parent() == Some(id));

        if !child.cx.is_subpixel() && child.cx.settings().render.pixel_align {
            transform.offset = transform.offset.pixel_align(cx.scale);
        }

        if child.cx.state.transform != transform {
            child.cx.state.transform = transform;
            cx.hierarchy.request_draw();
        }
    }
}
