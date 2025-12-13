use crate::{Affine, Painter, Size, Space, WidgetMut, WindowId, window::WindowSizing};

pub(super) fn layout_window(
    widget: &mut WidgetMut,
    window_id: WindowId,
    painter: &mut dyn Painter,
) -> Size {
    let Some(window) = widget.cx.root.get_window(window_id) else {
        return Size::ZERO;
    };

    let scale = window.scale();

    let max_size = match window.sizing {
        WindowSizing::FitContent => Size::all(f32::INFINITY),
        WindowSizing::Resizable { .. } => window.size,
    };

    let needs_layout = widget.cx.state.needs_layout;

    let size = {
        let _span = tracing::info_span!("layout").entered();
        let space = Space::new(Size::all(0.0), max_size);
        widget.layout_recursive(scale, painter, space)
    };

    if needs_layout {
        let _span = tracing::info_span!("compose").entered();
        widget.compose_recursive(scale, Affine::IDENTITY);
    }

    size
}
