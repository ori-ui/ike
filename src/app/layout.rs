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

    let size = {
        let _span = tracing::info_span!("layout").entered();
        let space = Space::new(Size::all(0.0), max_size);
        widget.layout_recursive(window, painter, space)
    };

    if needs_layout {
        let _span = tracing::info_span!("compose").entered();
        widget.compose_recursive(window, Affine::IDENTITY);
    }

    size
}
