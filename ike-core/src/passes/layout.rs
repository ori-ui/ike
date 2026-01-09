use crate::{Affine, Painter, Size, Space, WidgetMut, WindowId, WindowSizing, World, passes};

pub(crate) fn layout_window(
    world: &mut World,
    window: WindowId,
    painter: &mut dyn Painter,
) -> Option<Size> {
    let window_id = window;

    let window = world.window(window_id)?;

    let scale = window.scale();
    let sizing = window.sizing();

    // compute the max size of the base layer
    let max_size = match sizing {
        WindowSizing::FitContent => Size::INFINITY,
        WindowSizing::Resizable { .. } => window.size(),
    };

    let _span = tracing::info_span!("layout");

    // compute layout of the base layer
    let size = if let Some(layer) = window.base_layer()
        && let Some(mut widget) = world.widget_mut(layer.widget)
    {
        let space = Space::new(Size::ZERO, max_size);
        layout_widget(&mut widget, space, painter, scale)
    } else {
        return None;
    };

    let window = world.window_mut(window_id)?;

    // update layer size
    if let Some(layer) = window.base_layer_mut() {
        layer.size = size;
    }

    // update window size
    if let WindowSizing::FitContent = sizing {
        window.size = size;
    }

    // compute layout of layers in reverse
    for i in (1..window.layers.len()).rev() {
        // compute the size of the layers widget
        let size = if let Some(window) = world.window(window_id)
            && let Some(layer) = window.layers().get(i)
            && let Some(mut widget) = world.widget_mut(layer.widget)
        {
            let space = Space::new(Size::ZERO, Size::INFINITY);
            layout_widget(&mut widget, space, painter, scale)
        } else {
            continue;
        };

        // update the layer size
        if let Some(window) = world.window_mut(window_id)
            && let Some(layer) = window.layers_mut().get_mut(i)
        {
            layer.size = size;
        }
    }

    matches!(sizing, WindowSizing::FitContent).then_some(size)
}

pub(crate) fn layout_widget(
    widget: &mut WidgetMut<'_>,
    space: Space,
    painter: &mut dyn Painter,
    scale: f32,
) -> Size {
    if widget.cx.is_stashed() {
        widget.cx.state.size = space.min;
        return space.min;
    };

    if !widget.cx.hierarchy.needs_layout() && widget.cx.state.previous_space == Some(space) {
        return widget.cx.state.size;
    }

    let _span = widget.cx.enter_span();
    widget.cx.hierarchy.mark_laid_out();

    let mut cx = widget.cx.as_layout_cx(painter, scale);
    let mut size = widget.widget.layout(&mut cx, space);

    if !widget.cx.is_subpixel() {
        size = size.pixel_align(scale);

        if size.width > space.max.width {
            size.width -= scale.recip();
        }

        if size.height > space.max.height {
            size.height -= scale.recip();
        }
    }

    if !size.is_finite() {
        tracing::warn!(size = ?size, "size is not finite");
    }

    if widget.cx.state.size != size {
        widget.cx.hierarchy.request_draw();
    }

    passes::hierarchy::update_flags(widget);
    widget.cx.state.size = size;
    widget.cx.state.previous_space = Some(space);

    size
}

pub(crate) fn place_widget(widget: &mut WidgetMut<'_>, mut transform: Affine, scale: f32) {
    if !widget.cx.is_subpixel() {
        transform.offset = transform.offset.pixel_align(scale);
    }

    if widget.cx.state.transform != transform {
        widget.cx.state.transform = transform;
        widget.cx.hierarchy.request_compose();
        widget.cx.hierarchy.request_draw();
    }
}
