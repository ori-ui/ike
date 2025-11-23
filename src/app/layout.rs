use crate::{Affine, Fonts, Size, Space, WidgetMut, Window, window::WindowSizing};

pub(super) fn layout_window(
    widget: &mut WidgetMut,
    fonts: &mut dyn Fonts,
    window: &mut Window,
) -> Option<Size> {
    widget.state_mut().needs_layout = false;

    let max_size = match window.sizing {
        WindowSizing::FitContent => Size::all(f32::INFINITY),
        WindowSizing::Resizable { .. } => window.current_size,
    };

    let (w, mut cx) = widget.split_layout_cx(fonts);
    let space = Space::new(Size::all(0.0), max_size);
    let size = w.layout(&mut cx, space);
    widget.state_mut().size = size;
    widget.compose_recursive(Affine::IDENTITY);

    matches!(window.sizing, WindowSizing::FitContent).then_some(size)
}
