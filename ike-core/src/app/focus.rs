use crate::{Point, Rect, Tree, WidgetId, app::scroll, context::FocusUpdate};

/// Find the currently focused widget.
pub(super) fn find_focused(tree: &Tree, root: WidgetId) -> Option<WidgetId> {
    let state = tree.get_state_unchecked(root.index);

    if state.is_focused {
        return Some(root);
    }

    for &child in &state.children {
        let child_state = tree.get_state_unchecked(child.index);

        if child_state.has_focused {
            return find_focused(tree, child);
        }
    }

    None
}

pub(super) fn update_focus(tree: &mut Tree, root: WidgetId, update: FocusUpdate) {
    match update {
        FocusUpdate::None => {}

        FocusUpdate::Unfocus => {
            if let Some(focused) = find_focused(tree, root)
                && let Some(mut widget) = tree.get_mut(focused)
            {
                widget.set_focused(false);
            }
        }

        FocusUpdate::Target(target) => {
            give_focus(tree, root, target);
        }

        FocusUpdate::Next => {
            focus_next(tree, root, true);
        }

        FocusUpdate::Previous => {
            focus_next(tree, root, false);
        }
    }
}

pub(super) fn focus_next(tree: &mut Tree, id: WidgetId, forward: bool) -> Option<WidgetId> {
    let current = find_focused(tree, id);
    let focused = find_next_focusable(tree, id, forward);

    if current != focused {
        if let Some(current) = current
            && let Some(mut widget) = tree.get_mut(current)
        {
            widget.set_focused(false);
        }
        if let Some(focused) = focused {
            let rect = if let Some(mut widget) = tree.get_mut(focused) {
                widget.set_focused(true);

                Some(Rect::min_size(
                    Point::ORIGIN,
                    widget.size(),
                ))
            } else {
                None
            };

            if let Some(rect) = rect {
                scroll::scroll_to(tree, focused, rect);
            }
        }
    }

    focused
}

fn find_first_focusable(tree: &Tree, id: WidgetId, forward: bool) -> Option<WidgetId> {
    let state = tree.get_state_unchecked(id.index);

    if state.accepts_focus {
        return Some(id);
    }

    if forward {
        for &child in state.children.iter() {
            if let Some(focusable) = find_first_focusable(tree, child, forward) {
                return Some(focusable);
            }
        }
    } else {
        for &child in state.children.iter().rev() {
            if let Some(focusable) = find_first_focusable(tree, child, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn find_next_focusable(tree: &Tree, id: WidgetId, forward: bool) -> Option<WidgetId> {
    let state = tree.get_state_unchecked(id.index);

    if !state.has_focused {
        return find_first_focusable(tree, id, forward);
    }

    if forward {
        let mut children = state.children.iter().copied();

        for child in children.by_ref() {
            let child_state = tree.get_state_unchecked(child.index);

            if child_state.has_focused {
                if let Some(focusable) = find_next_focusable(tree, child, forward) {
                    return Some(focusable);
                }

                break;
            }
        }

        for child in children {
            if let Some(focusable) = find_first_focusable(tree, child, forward) {
                return Some(focusable);
            }
        }
    } else {
        let mut children = state.children.iter().copied().rev();

        for child in children.by_ref() {
            let child_state = tree.get_state_unchecked(child.index);

            if child_state.has_focused {
                if let Some(focusable) = find_next_focusable(tree, child, forward) {
                    return Some(focusable);
                }

                break;
            }
        }

        for child in children {
            if let Some(focusable) = find_first_focusable(tree, child, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn give_focus(tree: &mut Tree, root: WidgetId, target: WidgetId) {
    let current = find_focused(tree, root);

    if current != Some(target) {
        if let Some(current) = current
            && let Some(mut widget) = tree.get_mut(current)
        {
            widget.set_focused(false);
        }

        let rect = if let Some(mut widget) = tree.get_mut(target) {
            widget.set_focused(true);

            Some(Rect::min_size(
                Point::ORIGIN,
                widget.size(),
            ))
        } else {
            None
        };

        if let Some(rect) = rect {
            scroll::scroll_to(tree, target, rect);
        }
    }
}
