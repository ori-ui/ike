use crate::{Update, Widget, WidgetId, WidgetMut, Widgets, debug::debug_panic};

pub(crate) fn set_stashed(widget: &mut WidgetMut, is_stashed: bool) {
    // if we want to be un-stashed, but our parent is stashed, do nothing
    if !widget.cx.is_stashed()
        && let Some(parent) = widget.cx.get_parent()
        && parent.cx.is_stashed()
    {
        return;
    }

    set_stashed_recursive(widget, is_stashed);
    propagate_down(widget.cx.widgets, widget.id());
}

pub(crate) fn set_stashed_recursive(widget: &mut WidgetMut, is_stashed: bool) {
    for &child in &widget.cx.hierarchy.children {
        let Some(mut child) = widget.cx.get_widget_mut(child) else {
            debug_panic!("set_stashed called while descendant is borrowed");
            continue;
        };

        set_stashed_recursive(&mut child, is_stashed);
    }

    update(widget);

    if widget.cx.hierarchy.is_stashed() != is_stashed {
        widget.widget.update(
            &mut widget.cx.as_update_cx(),
            Update::Stashed(is_stashed),
        );
    }

    widget.cx.hierarchy.set_stashed(is_stashed);
}

pub(crate) fn set_disabled<T>(widget: &mut WidgetMut<T>, is_disabled: bool)
where
    T: Widget + ?Sized,
{
    // if we want to be un-disabled, but our parent is disabled, do nothing
    if !widget.cx.is_disabled()
        && let Some(parent) = widget.cx.get_parent()
        && parent.cx.is_disabled()
    {
        return;
    }

    set_disabled_recursive(widget, is_disabled);
    propagate_down(widget.cx.widgets, widget.cx.id());
}

pub(crate) fn set_disabled_recursive<T>(widget: &mut WidgetMut<T>, is_disabled: bool)
where
    T: Widget + ?Sized,
{
    for &child in &widget.cx.hierarchy.children {
        let Some(mut child) = widget.cx.get_widget_mut(child) else {
            debug_panic!("set_disabled called while descendant is borrowed");
            continue;
        };

        set_disabled_recursive(&mut child, is_disabled);
    }

    update(widget);

    if widget.cx.hierarchy.is_disabled() != is_disabled {
        widget.widget.update(
            &mut widget.cx.as_update_cx(),
            Update::Disabled(is_disabled),
        );
    }

    widget.cx.hierarchy.set_disabled(is_disabled);
}

pub(crate) fn propagate_down(widgets: &Widgets, widget: WidgetId) {
    let mut current = Some(widget);

    while let Some(widget) = current
        && let Some(hierarchy) = widgets.get_hierarchy(widget)
    {
        hierarchy.update(widgets);
        current = hierarchy.parent;
    }
}

pub(crate) fn update<T>(widget: &WidgetMut<T>)
where
    T: Widget + ?Sized,
{
    widget.cx.hierarchy.update(widget.cx.widgets)
}
