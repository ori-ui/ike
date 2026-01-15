use crate::{Update, WidgetMut, WindowId, World, passes};

pub(crate) fn window(world: &mut World, window: WindowId, update: &Update) {
    let Some(window) = world.window(window) else {
        return;
    };

    for layer in window.layers.clone().iter() {
        if let Ok(widget) = world.widget_mut(layer.widget) {
            widget_recursive(widget, update);
        }
    }
}

pub(crate) fn widget_recursive(mut widget: WidgetMut<'_>, update: &Update) {
    let _span = widget.cx.enter_span();

    if let Update::WindowScaled(..) = update
        && !widget.cx.is_subpixel()
    {
        widget.cx.hierarchy.request_layout();
    }

    self::widget(&mut widget, update.clone());

    passes::hierarchy::for_each_child(widget, |child| {
        if let Ok(child) = child {
            widget_recursive(child, update);
        }
    });
}

pub(crate) fn widget(widget: &mut WidgetMut<'_>, update: Update) {
    let mut cx = widget.cx.as_update_cx();
    widget.widget.update(&mut cx, update);
}
