use crate::{
    Affine, Fonts, LayoutCx, Size, Space, Tree, UpdateCx, WidgetId, Window, window::WindowSizing,
};

pub(super) fn layout_window(
    tree: &mut Tree,
    fonts: &mut dyn Fonts,
    window: &mut Window,
) -> Option<Size> {
    if !tree.needs_layout(window.content) {
        return None;
    }

    let size = tree.with_entry(window.content, |tree, widget, state| {
        state.needs_layout = false;

        let mut cx = LayoutCx { fonts, tree, state };

        let max_size = match window.sizing {
            WindowSizing::FitContent => Size::all(f32::INFINITY),
            WindowSizing::Resizable { .. } => window.current_size,
        };

        let space = Space::new(Size::all(0.0), max_size);
        let size = widget.layout(&mut cx, space);
        state.size = size;
        size
    });

    compose_widget(tree, window.content, Affine::IDENTITY);

    matches!(window.sizing, WindowSizing::FitContent).then_some(size)
}

pub(super) fn compose_widget(tree: &mut Tree, id: WidgetId, transform: Affine) {
    tree.with_entry(id, |tree, widget, state| {
        let transform = transform * state.transform;
        state.global_transform = transform;

        let mut cx = UpdateCx { tree, state };
        widget.compose(&mut cx);

        for &child in &state.children {
            compose_widget(tree, child, transform);
        }
    });
}
