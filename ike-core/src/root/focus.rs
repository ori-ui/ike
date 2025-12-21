use crate::{
    Arena, BuildCx, ImeSignal, Point, Rect, Root, RootSignal, WidgetId,
    context::FocusUpdate,
    root::{query, scroll},
};

pub fn update_focus(root: &mut Root, root_widget: WidgetId, update: FocusUpdate) {
    match update {
        FocusUpdate::None => {}

        FocusUpdate::Unfocus => {
            set_focused(root, root_widget, None);
        }

        FocusUpdate::Target(target) => {
            set_focused(root, root_widget, Some(target));
        }

        FocusUpdate::Next => {
            focus_next(root, root_widget, true);
        }

        FocusUpdate::Previous => {
            focus_next(root, root_widget, false);
        }
    }
}

pub fn focus_next(root: &mut Root, root_widget: WidgetId, forward: bool) {
    let focused = find_next_focusable(&root.arena, root_widget, forward);
    set_focused(root, root_widget, focused);
}

pub fn set_focused(root: &mut Root, root_widget: WidgetId, target: Option<WidgetId>) -> bool {
    let current = query::find_focused(&root.arena, root_widget);

    if current == target {
        return false;
    }

    if let Some(current) = current
        && let Some(mut widget) = root.get_widget_mut(current)
    {
        widget.set_focused(false);

        if widget.cx.state.accepts_text {
            widget.cx.root.signal(RootSignal::Ime(ImeSignal::End));
        }
    }

    if let Some(target) = target {
        let rect = if let Some(mut widget) = root.get_widget_mut(target) {
            widget.set_focused(true);

            if widget.cx.state.accepts_text {
                widget.cx.root.signal(RootSignal::Ime(ImeSignal::Start));
            }

            Some(Rect::min_size(
                Point::ORIGIN,
                widget.cx.size(),
            ))
        } else {
            None
        };

        if let Some(rect) = rect {
            scroll::scroll_to(root, target, rect);
        }
    }

    true
}

fn find_first_focusable(arena: &Arena, current: WidgetId, forward: bool) -> Option<WidgetId> {
    let state = arena.get_state(current.index)?;

    if state.accepts_focus {
        return Some(current);
    }

    if forward {
        for &child in state.children.iter() {
            if let Some(focusable) = find_first_focusable(arena, child, forward) {
                return Some(focusable);
            }
        }
    } else {
        for &child in state.children.iter().rev() {
            if let Some(focusable) = find_first_focusable(arena, child, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn find_next_focusable(arena: &Arena, id: WidgetId, forward: bool) -> Option<WidgetId> {
    let state = arena.get_state(id.index)?;

    if !state.has_focused {
        return find_first_focusable(arena, id, forward);
    }

    if forward {
        let mut children = state.children.iter().copied();

        for child in children.by_ref() {
            let child_state = arena.get_state(child.index)?;

            if child_state.has_focused {
                if let Some(focusable) = find_next_focusable(arena, child, forward) {
                    return Some(focusable);
                }

                break;
            }
        }

        for child in children {
            if let Some(focusable) = find_first_focusable(arena, child, forward) {
                return Some(focusable);
            }
        }
    } else {
        let mut children = state.children.iter().copied().rev();

        for child in children.by_ref() {
            let child_state = arena.get_state(child.index)?;

            if child_state.has_focused {
                if let Some(focusable) = find_next_focusable(arena, child, forward) {
                    return Some(focusable);
                }

                break;
            }
        }

        for child in children {
            if let Some(focusable) = find_first_focusable(arena, child, forward) {
                return Some(focusable);
            }
        }
    }

    None
}
