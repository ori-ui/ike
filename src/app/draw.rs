use crate::{Canvas, DrawCx, Tree, WidgetId, Window};

pub(super) fn draw_widget(
    tree: &mut Tree,
    window: &Window,
    widget: WidgetId,
    canvas: &mut dyn Canvas,
) {
    tree.with_entry(widget, |tree, widget, state| {
        canvas.transform(state.transform, &mut |canvas| {
            let mut cx = DrawCx {
                window,
                tree,
                state,
            };
            widget.draw(&mut cx, canvas);

            for &child in &state.children {
                draw_widget(tree, window, child, canvas);
            }
        });
    });
}

pub(super) fn draw_over_widget(
    tree: &mut Tree,
    window: &Window,
    widget: WidgetId,
    canvas: &mut dyn Canvas,
) {
    tree.with_entry(widget, |tree, widget, state| {
        canvas.transform(state.transform, &mut |canvas| {
            state.needs_draw = false;

            let mut cx = DrawCx {
                window,
                tree,
                state,
            };
            widget.draw_over(&mut cx, canvas);

            for &child in &state.children {
                draw_over_widget(tree, window, child, canvas);
            }
        });
    });
}
