use std::mem;

use crate::{
    Builder, ChildUpdate, GetError, Update, Widget, WidgetId, WidgetMut, WindowId, World,
    debug::debug_panic, passes, world::Widgets,
};

pub(crate) fn insert_child(world: &mut World, parent: WidgetId, index: usize, child: WidgetId) {
    if let Some(child) = world.widgets.get_hierarchy_mut(child) {
        child.parent = Some(parent);
    }

    if let Some(parent) = world.widgets.get_hierarchy_mut(parent) {
        parent.children_mut().insert(index, child);
    }

    propagate_up(world, parent);

    if let Ok(mut parent) = world.widget_mut(parent) {
        let update = Update::Children(ChildUpdate::Inserted(index));
        passes::update::widget(&mut parent, update);
        parent.cx.request_layout();
        parent.cx.request_draw();
    }
}

pub(crate) fn set_child(world: &mut World, parent: WidgetId, index: usize, child: WidgetId) {
    debug_assert!(!world.is_child(parent, child));

    let parent_id = parent;

    let Some(parent) = world.widgets.get_hierarchy_mut(parent) else {
        return;
    };

    // get the current child
    let Some(current) = parent.children_mut().get_mut(index) else {
        debug_panic!("set_child called with invalid index");
        return;
    };

    // replace the current child with the new child
    let prev = mem::replace(current, child);

    // unset the parent of the previous child
    if let Some(prev) = world.widgets.get_hierarchy_mut(prev) {
        prev.parent = None;
    }

    // unset the window of the previous child
    set_window(world, prev, None);

    // set the parent of the new child
    if let Some(child) = world.widgets.get_hierarchy_mut(child) {
        child.parent = Some(parent_id);
    }

    propagate_up(world, parent_id);

    // run update pass on the parent widget
    if let Ok(mut parent) = world.widget_mut(parent_id) {
        let update = Update::Children(ChildUpdate::Replaced(index));
        passes::update::widget(&mut parent, update);
        parent.cx.request_layout();
        parent.cx.request_draw();
    }
}

pub(crate) fn swap_children(world: &mut World, parent: WidgetId, index_a: usize, index_b: usize) {
    let parent_id = parent;

    if let Some(parent) = world.widgets.get_hierarchy_mut(parent) {
        parent.children_mut().swap(index_a, index_b);
    }

    // run update pass on the parent widget
    if let Ok(mut parent) = world.widget_mut(parent_id) {
        let update = Update::Children(ChildUpdate::Swapped(index_a, index_b));
        passes::update::widget(&mut parent, update);
        parent.cx.request_layout();
        parent.cx.request_draw();
    }
}

pub(crate) fn remove_child(world: &mut World, parent: WidgetId, index: usize) -> Option<WidgetId> {
    let hierarchy = world.widgets.get_hierarchy_mut(parent)?;
    let child = hierarchy.children_mut().remove(index);

    if let Ok(mut parent) = world.widget_mut(parent) {
        let update = Update::Children(ChildUpdate::Removed(index));
        passes::update::widget(&mut parent, update);
        parent.cx.request_layout();
        parent.cx.request_draw();
    }

    // unset the parent of the child
    if let Some(child) = world.widgets.get_hierarchy_mut(parent) {
        child.parent = None;
    }

    // unset the window of the child
    set_window(world, child, None);

    Some(child)
}

pub(crate) fn remove(world: &mut World, widget: WidgetId) {
    if let Some(hierarchy) = world.widgets.get_hierarchy(widget) {
        for &child in hierarchy.children.clone().iter() {
            remove(world, child);
        }
    }

    let Ok(widget) = world.get_widget(widget) else {
        return;
    };

    let id = widget.cx.id();
    let parent = widget.cx.parent();

    drop(widget);

    if let Ok(mut widget) = world.widget_mut(id) {
        passes::update::widget(&mut widget, Update::Removed);
    }

    // set the window to `None` to handle the removal of hover, focus, and active
    set_window(world, id, None);

    if let Some(parent) = parent
        && let Some(hierarchy) = world.widgets.get_hierarchy_mut(parent)
        && let Some(index) = hierarchy.children.iter().position(|child| *child == id)
    {
        hierarchy.children_mut().remove(index);

        if let Ok(mut parent) = world.widget_mut(parent) {
            let update = Update::Children(ChildUpdate::Removed(index));
            passes::update::widget(&mut parent, update);
            parent.cx.request_layout();
            parent.cx.request_draw();
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

    // remove widget from pointer hovered
    if has_hovered
        && let Some(window) = previous
        && let Some(window) = world.state.window_mut(window)
    {
        for pointer in &mut window.pointers {
            if let Some(hovered) = pointer.hovering
                && is_descendant(&world.widgets, widget, hovered)
            {
                pointer.hovering = None;

                if let Ok(mut widget) = world.widget_mut(hovered) {
                    widget.set_hovered(false);
                }

                break;
            }
        }
    }

    if has_active {
        // remove widget from pointer capturer
        if let Some(window) = previous
            && let Some(window) = world.state.window_mut(window)
        {
            for pointer in &mut window.pointers {
                if let Some(capturer) = pointer.capturer
                    && is_descendant(&world.widgets, widget, capturer)
                {
                    pointer.capturer = None;

                    if let Ok(mut widget) = world.widget_mut(capturer) {
                        widget.set_active(false);
                    }

                    break;
                }
            }
        }

        // remove widget from touch capturer
        if let Some(window) = previous
            && let Some(window) = world.state.window_mut(window)
        {
            for touch in &mut window.touches {
                if let Some(capturer) = touch.capturer
                    && is_descendant(&world.widgets, widget, capturer)
                {
                    touch.capturer = None;

                    if let Ok(mut widget) = world.widget_mut(capturer) {
                        widget.set_active(false);
                    }

                    break;
                }
            }
        }
    }

    // remove widget from window focused
    if has_focused
        && let Some(window) = previous
        && let Some(window) = world.state.window_mut(window)
        && let Some(focused) = window.focused
    {
        window.focused = None;

        if let Ok(mut widget) = world.widget_mut(focused) {
            widget.set_focused(false);
        }
    }

    // TODO: handle stashed and disabled, it only matters for when a widget is moved from one
    //       window to another while a parent is stashed or disabled, which is something that will
    //       not happen often

    propagate_up(world, widget);
}

pub(crate) fn is_descendant(widgets: &Widgets, ancestor: WidgetId, descendant: WidgetId) -> bool {
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

pub(crate) fn for_each_child(
    widget: WidgetMut<'_>,
    mut f: impl FnMut(Result<WidgetMut<'_>, GetError>),
) {
    drop(widget.widget);
    drop(widget.cx.state);

    for &child in widget.cx.hierarchy.children.iter() {
        f(widget.cx.widgets.get_mut(widget.cx.world, child));
    }

    widget.cx.hierarchy.propagate_down(widget.cx.widgets);
}

pub(crate) fn propagate_up(world: &mut World, widget: WidgetId) {
    let Some(hierarchy) = world.widgets.get_hierarchy_mut(widget) else {
        return;
    };

    let window = hierarchy.window;
    let flags = hierarchy.flags.get();

    for &child in hierarchy.children.clone().iter() {
        if let Some(child) = world.widgets.get_hierarchy_mut(child) {
            flags.propagate_up(child.flags.get_mut());
            child.window = window;
        }

        propagate_up(world, child);
    }
}

pub(crate) fn propagate_down(widgets: &Widgets, widget: WidgetId) {
    let mut current = Some(widget);

    while let Some(widget) = current
        && let Some(hierarchy) = widgets.get_hierarchy(widget)
    {
        hierarchy.propagate_down(widgets);
        current = hierarchy.parent;
    }
}

pub(crate) fn set_stashed<T>(widget: &mut WidgetMut<'_, T>, is_stashed: bool)
where
    T: Widget + ?Sized,
{
    // if we want to be un-stashed, but our parent is stashed, do nothing
    if !widget.cx.is_stashed()
        && let Ok(parent) = widget.cx.get_parent()
        && parent.cx.is_stashed()
    {
        return;
    }

    set_stashed_recursive(widget, is_stashed);
    propagate_down(widget.cx.widgets, widget.cx.id());
}

pub(crate) fn set_stashed_recursive<T>(widget: &mut WidgetMut<'_, T>, is_stashed: bool)
where
    T: Widget + ?Sized,
{
    for &child in widget.cx.hierarchy.children.iter() {
        let Ok(mut child) = widget.cx.get_widget_mut(child) else {
            debug_panic!("set_stashed called while descendant is borrowed");
            continue;
        };

        set_stashed_recursive(&mut child, is_stashed);
    }

    widget.cx.hierarchy.propagate_down(widget.cx.widgets);

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
        && let Ok(parent) = widget.cx.get_parent()
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
    for &child in widget.cx.hierarchy.children.iter() {
        let Ok(mut child) = widget.cx.get_widget_mut(child) else {
            debug_panic!("set_disabled called while descendant is borrowed");
            continue;
        };

        set_disabled_recursive(&mut child, is_disabled);
    }

    widget.cx.hierarchy.propagate_down(widget.cx.widgets);

    if widget.cx.hierarchy.is_disabled() != is_disabled {
        widget.widget.update(
            &mut widget.cx.as_update_cx(),
            Update::Disabled(is_disabled),
        );
    }

    widget.cx.hierarchy.set_disabled(is_disabled);
}
