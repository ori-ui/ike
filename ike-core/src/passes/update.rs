use crate::{Update, WidgetMut, WindowId, World, passes};

pub(crate) fn update_window(world: &mut World, window: WindowId, update: &Update) {
    let Some(window) = world.window(window) else {
        return;
    };

    for layer in window.layers.clone().iter() {
        if let Some(mut widget) = world.widget_mut(layer.widget) {
            update_widget_recursive(&mut widget, update);
        }
    }
}

pub(crate) fn update_widget_recursive(widget: &mut WidgetMut<'_>, update: &Update) {
    widget.cx.for_each_child_mut(|child| {
        update_widget_recursive(child, update);
    });

    passes::propagate::update_hirarchy(widget);
    update_widget(widget, update.clone());
}

pub(crate) fn update_widget(widget: &mut WidgetMut<'_>, update: Update) {
    let mut cx = widget.cx.as_update_cx();
    widget.widget.update(&mut cx, update);
}
