use crate::{Affine, Painter, Size, Space, WidgetMut, Window, window::WindowSizing};

pub(super) fn layout_window(
    widget: &mut WidgetMut,
    painter: &mut dyn Painter,
    window: &mut Window,
) -> Size {
    let max_size = match window.sizing {
        WindowSizing::FitContent => Size::all(f32::INFINITY),
        WindowSizing::Resizable { .. } => window.current_size,
    };

    let needs_layout = widget.state().needs_layout;
    widget.state_mut().needs_layout = false;

    let (w, mut cx) = widget.split_layout_cx(window);
    let space = Space::new(Size::all(0.0), max_size);
    let mut size = w.layout(&mut cx, painter, space);

    if widget.is_pixel_perfect() {
        size = size.ceil_to_scale(window.scale());
    }

    widget.state_mut().size = size;

    if needs_layout {
        widget.compose_recursive(window, Affine::IDENTITY);
    }

    size
}
