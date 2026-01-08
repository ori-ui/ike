use std::mem;

use crate::{Update, WidgetId, WindowId, World, debug::debug_panic, passes, widget::ChildUpdate};

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

    // run update pass on the parent widget
    if let Some(mut parent) = world.widget_mut(parent_id) {
        let update = Update::Children(ChildUpdate::Replaced(index));
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
    let window = widget.cx.hierarchy.window;

    drop(widget);

    if let Some(window_id) = window
        && let Some(window) = world.window_mut(window_id)
    {
        for pointer in window.pointers.iter_mut() {
            if pointer.hovering == Some(id) {
                pointer.hovering = None;
            }

            if pointer.capturer == Some(id) {
                pointer.capturer = None;
            }
        }

        for touch in window.touches.iter_mut() {
            if touch.capturer == Some(id) {
                touch.capturer = None;
            }
        }

        if window.focused == Some(id) {
            passes::focus::next(world, window_id, true);
        }
    }

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
    if let Some(hierarchy) = world.widgets.get_hierarchy_mut(widget) {
        hierarchy.window = window;
    }

    propagate_up(world, widget);
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
