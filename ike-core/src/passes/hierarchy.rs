use std::mem;

use crate::{WidgetId, WindowId, World};

pub(crate) fn add_child(world: &mut World, parent: WidgetId, child: WidgetId) {
    if let Some(child) = world.widgets.get_hierarchy_mut(child) {
        child.parent = Some(parent);
    }

    if let Some(parent) = world.widgets.get_hierarchy_mut(parent) {
        parent.children.push(child);
    }

    propagate_up(world, parent);
}

pub(crate) fn set_child(world: &mut World, parent: WidgetId, index: usize, child: WidgetId) {
    todo!()
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

pub(crate) fn swap_children(world: &mut World, parent: WidgetId, index_a: usize, index_b: usize) {
    let Some(parent) = world.widgets.get_hierarchy_mut(parent) else {
        return;
    };

    parent.children.swap(index_a, index_b);
}

pub(crate) fn remove(world: &mut World, widget: WidgetId) {
    let Some(widget) = world.widget(widget) else {
        return;
    };

    let id = widget.cx.id();
    let parent = widget.cx.parent();

    drop(widget);

    if let Some(parent) = parent
        && let Some(parent) = world.widgets.get_hierarchy_mut(parent)
        && let Some(index) = parent.children.iter().position(|child| *child == id)
    {
        parent.children.remove(index);
    }

    world.widgets.remove(id);
}
