use crate::{Arena, Point, Rect, WidgetId};

pub fn find_hovered(arena: &Arena, root_widget: WidgetId) -> Option<WidgetId> {
    let state = arena.get_state(root_widget.index)?;

    if state.is_hovered {
        return Some(root_widget);
    }

    for &child in &state.children {
        if let Some(child_state) = arena.get_state(child.index)
            && child_state.has_hovered
        {
            return find_hovered(arena, child);
        }
    }

    None
}

pub fn find_active(arena: &Arena, root_widget: WidgetId) -> Option<WidgetId> {
    let state = arena.get_state(root_widget.index)?;

    if state.is_active {
        return Some(root_widget);
    }

    for &child in &state.children {
        if let Some(child_state) = arena.get_state(child.index)
            && child_state.has_active
        {
            return find_active(arena, child);
        }
    }

    None
}

pub fn find_focused(arena: &Arena, root_widget: WidgetId) -> Option<WidgetId> {
    let state = arena.get_state(root_widget.index)?;

    if state.is_focused {
        return Some(root_widget);
    }

    for &child in &state.children {
        let child_state = arena.get_state(child.index)?;

        if child_state.has_focused {
            return find_focused(arena, child);
        }
    }

    None
}

pub fn find_widget_at(arena: &Arena, root_widget: WidgetId, point: Point) -> Option<WidgetId> {
    let state = arena.get_state(root_widget.index)?;
    let local = state.global_transform.inverse() * point;

    if !Rect::min_size(Point::ORIGIN, state.size).contains(local) {
        return None;
    }

    for &child in &state.children {
        if let Some(id) = find_widget_at(arena, child, point) {
            return Some(id);
        }
    }

    if state.accepts_pointer {
        Some(root_widget)
    } else {
        None
    }
}
