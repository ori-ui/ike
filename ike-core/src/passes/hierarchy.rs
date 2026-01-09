use std::mem;

use crate::{
    Builder, Update, Widget, WidgetId, WidgetMut, WindowId, World, debug::debug_panic, passes,
    widget::ChildUpdate, world::Widgets,
};

pub(crate) fn add_child(world: &mut World, parent: WidgetId, child: WidgetId) {
    if let Some(child) = world.widgets.get_hierarchy_mut(child) {
        child.parent = Some(parent);
    }

    if let Some(parent) = world.widgets.get_hierarchy_mut(parent) {
        parent.children.push(child);
    }

    propagate_up(world, parent);

    if let Some(mut parent) = world.widget_mut(parent) {
        let index = parent.cx.hierarchy.children.len() - 1;
        let update = Update::Children(ChildUpdate::Added(index));
        passes::update::widget(&mut parent, update);
        parent.cx.request_layout();
    }
}

pub(crate) fn set_child(world: &mut World, parent: WidgetId, index: usize, child: WidgetId) {
    debug_assert!(!world.is_child(parent, child));

    let parent_id = parent;

    let Some(parent) = world.widgets.get_hierarchy_mut(parent) else {
        return;
    };

    // get the current child
    let Some(current) = parent.children.get_mut(index) else {
        debug_panic!("set_child called with invalid index");
        return;
    };

    // replace the current child with the new child
    let prev = mem::replace(current, child);

    // unset the parent of the previous child
    if let Some(prev) = world.widgets.get_hierarchy_mut(prev) {
        prev.parent = None;
    }

    // set the parent of the new child
    if let Some(child) = world.widgets.get_hierarchy_mut(child) {
        child.parent = Some(parent_id);
    }

    propagate_up(world, parent_id);

    // run update pass on the parent widget
    if let Some(mut parent) = world.widget_mut(parent_id) {
        let update = Update::Children(ChildUpdate::Replaced(index));
        passes::update::widget(&mut parent, update);
        parent.cx.request_layout();
    }
}

pub(crate) fn swap_children(world: &mut World, parent: WidgetId, index_a: usize, index_b: usize) {
    let parent_id = parent;

    if let Some(parent) = world.widgets.get_hierarchy_mut(parent) {
        parent.children.swap(index_a, index_b);
    }

    // run update pass on the parent widget
    if let Some(mut parent) = world.widget_mut(parent_id) {
        let update = Update::Children(ChildUpdate::Swapped(index_a, index_b));
        passes::update::widget(&mut parent, update);
        parent.cx.request_layout();
    }
}

pub(crate) fn remove(world: &mut World, widget: WidgetId) {
    let Some(widget) = world.widget(widget) else {
        return;
    };

    let id = widget.cx.id();
    let parent = widget.cx.parent();

    drop(widget);
    set_window(world, id, None);

    if let Some(parent) = parent
        && let Some(hierarchy) = world.widgets.get_hierarchy_mut(parent)
        && let Some(index) = hierarchy.children.iter().position(|child| *child == id)
    {
        hierarchy.children.remove(index);

        if let Some(mut parent) = world.widget_mut(parent) {
            let update = Update::Children(ChildUpdate::Removed(index));
            passes::update::widget(&mut parent, update);
            parent.cx.request_layout();
        }
    }

    world.widgets.remove(id);
}

/// Set the `window` of a `widget` and its descendants.
pub(crate) fn set_window(world: &mut World, widget: WidgetId, window: Option<WindowId>) {
    let Some(hierarchy) = world.widgets.get_hierarchy_mut(widget) else {
        return;
    };

    let previous = mem::replace(&mut hierarchy.window, window);

    let has_hovered = hierarchy.has_hovered();
    let has_focused = hierarchy.has_focused();
    let has_active = hierarchy.has_active();

    if has_hovered
        && let Some(window) = previous
        && let Some(window) = world.state.window_mut(window)
    {
        for pointer in &mut window.pointers {
            if let Some(hovered) = pointer.hovering
                && is_descendant(&world.widgets, widget, hovered)
            {
                pointer.hovering = None;

                if let Some(mut widget) = world.widget_mut(hovered) {
                    widget.set_hovered(false);
                }

                break;
            }
        }
    }

    if has_active
        && let Some(window) = previous
        && let Some(window) = world.state.window_mut(window)
    {
        for pointer in &mut window.pointers {
            if let Some(capturer) = pointer.capturer
                && is_descendant(&world.widgets, widget, capturer)
            {
                pointer.capturer = None;

                if let Some(mut widget) = world.widget_mut(capturer) {
                    widget.set_active(false);
                }

                break;
            }
        }
    }

    if has_focused
        && let Some(window) = previous
        && let Some(window) = world.state.window_mut(window)
        && let Some(focused) = window.focused
    {
        window.focused = None;

        if let Some(mut widget) = world.widget_mut(focused) {
            widget.set_focused(false);
        }
    }

    propagate_up(world, widget);
}

fn is_descendant(widgets: &Widgets, ancestor: WidgetId, descendant: WidgetId) -> bool {
    let mut current = Some(descendant);

    while let Some(widget) = current
        && let Some(hierarchy) = widgets.get_hierarchy(widget)
    {
        if ancestor == widget {
            return true;
        }

        current = hierarchy.parent;
    }

    false
}

pub(crate) fn propagate_up(world: &mut World, widget: WidgetId) {
    let Some(hierarchy) = world.widgets.get_hierarchy_mut(widget) else {
        return;
    };

    let window = hierarchy.window;
    let flags = hierarchy.flags.get();

    let children = mem::take(&mut hierarchy.children);
    for &child in &children {
        if let Some(child) = world.widgets.get_hierarchy_mut(child) {
            flags.propagate_up(child.flags.get_mut());
            child.window = window;
        }

        propagate_up(world, child);
    }

    if let Some(hierarchy) = world.widgets.get_hierarchy_mut(widget) {
        debug_assert!(hierarchy.children.is_empty());
        hierarchy.children = children;
    }
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

pub(crate) fn update_flags<T>(widget: &WidgetMut<'_, T>)
where
    T: Widget + ?Sized,
{
    widget.cx.hierarchy.update(widget.cx.widgets)
}

pub(crate) fn set_stashed(widget: &mut WidgetMut<'_>, is_stashed: bool) {
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

pub(crate) fn set_stashed_recursive(widget: &mut WidgetMut<'_>, is_stashed: bool) {
    for &child in &widget.cx.hierarchy.children {
        let Some(mut child) = widget.cx.get_widget_mut(child) else {
            debug_panic!("set_stashed called while descendant is borrowed");
            continue;
        };

        set_stashed_recursive(&mut child, is_stashed);
    }

    update_flags(widget);

    if widget.cx.hierarchy.is_stashed() != is_stashed {
        widget.widget.update(
            &mut widget.cx.as_update_cx(),
            Update::Stashed(is_stashed),
        );
    }

    widget.cx.hierarchy.set_stashed(is_stashed);
}

pub(crate) fn set_disabled<T>(widget: &mut WidgetMut<'_, T>, is_disabled: bool)
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

pub(crate) fn set_disabled_recursive<T>(widget: &mut WidgetMut<'_, T>, is_disabled: bool)
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

    update_flags(widget);

    if widget.cx.hierarchy.is_disabled() != is_disabled {
        widget.widget.update(
            &mut widget.cx.as_update_cx(),
            Update::Disabled(is_disabled),
        );
    }

    widget.cx.hierarchy.set_disabled(is_disabled);
}
