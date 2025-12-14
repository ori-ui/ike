use crate::{Affine, Painter, Size, Space, WidgetMut, WindowId, window::WindowSizing};

pub(super) fn layout_window(
    widget: &mut WidgetMut,
    window_id: WindowId,
    painter: &mut dyn Painter,
) -> Size {
    let Some(window) = widget.cx.root.get_window(window_id) else {
        tracing::error!(
            window = ?window_id,
            "tried to layout window that doesn't exist",
        );

        return Size::ZERO;
    };

    let scale = window.scale();

    let max_size = match window.sizing {
        WindowSizing::FitContent => Size::all(f32::INFINITY),
        WindowSizing::Resizable { .. } => window.size,
    };

    let _span = tracing::info_span!("layout").entered();
    let space = Space::new(Size::all(0.0), max_size);
    widget.layout_recursive(scale, painter, space)
}

pub(super) fn compose_window(widget: &mut WidgetMut, window: WindowId) {
    if let Some(window) = widget.cx.root.get_window(window) {
        let _span = tracing::info_span!("compose").entered();
        widget.compose_recursive(window.scale, Affine::IDENTITY);
    } else {
        tracing::error!(
            window = ?window,
            "tried to compose window that doesn't exist",
        );
    }
}
