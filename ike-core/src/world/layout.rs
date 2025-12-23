use std::rc::Rc;

use crate::{Affine, Offset, Painter, Size, Space, WindowId, WindowSizing, World};

pub(super) fn layout_window(
    world: &mut World,
    window: WindowId,
    painter: &mut dyn Painter,
) -> Option<Size> {
    let window_id = window;
    let window = world.window(window_id)?;

    let scale = window.scale();
    let sizing = window.sizing();

    let max_size = match window.sizing {
        WindowSizing::FitContent => Size::all(f32::INFINITY),
        WindowSizing::Resizable { .. } => window.size,
    };

    let _span = tracing::info_span!("layout").entered();
    let space = Space::new(Size::all(0.0), max_size);

    let size = if let Some(layer) = window.layers().first()
        && let Some(mut widget) = world.widget_mut(layer.root)
    {
        widget.layout_recursive(scale, painter, space)
    } else {
        return None;
    };

    let window = world.window_mut(window_id)?;

    if let Some(layer) = Rc::make_mut(&mut window.layers).first_mut() {
        layer.size = size;
    }

    if let WindowSizing::FitContent = window.sizing {
        window.size = size;
    }

    for i in 1..window.layers.len() {
        let size = if let Some(window) = world.window(window_id)
            && let Some(layer) = window.layers.get(i)
            && let Some(mut widget) = world.widget_mut(layer.root)
        {
            let space = Space::new(Size::ZERO, Size::INFINITY);
            widget.layout_recursive(scale, painter, space)
        } else {
            continue;
        };

        if let Some(window) = world.window_mut(window_id)
            && let Some(layer) = Rc::make_mut(&mut window.layers).get_mut(i)
        {
            layer.size = size;
        }
    }

    matches!(sizing, WindowSizing::FitContent).then_some(size)
}

pub(super) fn compose_window(world: &mut World, window: WindowId) {
    let window_id = window;
    let Some(window) = world.window(window_id) else {
        tracing::error!(
            window = ?window,
            "tried to compose window that doesn't exist",
        );
        return;
    };

    let scale = window.scale();
    let _span = tracing::info_span!("compose").entered();

    for i in 0..window.layers.len() {
        if let Some(window) = world.window(window_id)
            && let Some(layer) = window.layers.get(i)
        {
            let position = layer.position;

            if let Some(mut widget) = world.widget_mut(layer.root) {
                let transform = Affine::translate(Offset::new(position.x, position.y));
                widget.compose_recursive(scale, transform);
            }
        }
    }
}
