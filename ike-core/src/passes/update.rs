use crate::{Update, WidgetMut, WindowId, World, passes};

pub(crate) fn window(world: &mut World, window: WindowId, update: &Update) {
    let Some(window) = world.window(window) else {
        return;
    };

    for layer in window.layers.clone().iter() {
        if let Some(mut widget) = world.widget_mut(layer.widget) {
            widget_recursive(&mut widget, update);
        }
    }
}

pub(crate) fn widget_recursive(widget: &mut WidgetMut<'_>, update: &Update) {
    let _span = widget.cx.enter_span();

    widget.cx.for_each_child_mut(|child| {
        widget_recursive(child, update);
    });

    passes::hierarchy::update_flags(widget);

    if let Update::WindowScaled(..) = update
        && !widget.cx.is_subpixel()
    {
        widget.cx.hierarchy.request_layout();
    }

    self::widget(widget, update.clone());
}

pub(crate) fn widget(widget: &mut WidgetMut<'_>, update: Update) {
    let mut cx = widget.cx.as_update_cx();
    widget.widget.update(&mut cx, update);
}
