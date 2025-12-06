use crate::{Affine, Painter, Size, Space, WidgetMut, Window, window::WindowSizing};

pub(super) fn layout_window(
    widget: &mut WidgetMut,
    painter: &mut dyn Painter,
    window: &mut Window,
) -> Size {
    widget.state_mut().needs_layout = false;

    let max_size = match window.sizing {
        WindowSizing::FitContent => Size::all(f32::INFINITY),
        WindowSizing::Resizable { .. } => window.current_size,
    };

    let (w, mut cx) = widget.split_layout_cx(window);
    let space = Space::new(Size::all(0.0), max_size);
    let size = w.layout(&mut cx, painter, space);
    widget.state_mut().size = size.ceil_to_scale(window.scale());
    widget.compose_recursive(window, Affine::IDENTITY);

    size
}
