use crate::{
    Arena, BuildCx, ImeEvent, ImeSignal, Point, Rect, Signal, TextEvent, WidgetId, Window,
    WindowId, World,
    context::FocusUpdate,
    debug::debug_panic,
    world::{scroll, text},
};

pub fn update_focus(world: &mut World, window: WindowId, update: FocusUpdate) {
    match update {
        FocusUpdate::None => {}

        FocusUpdate::Unfocus => {
            set_focused(world, window, None);
        }

        FocusUpdate::Target(target) => {
            set_focused(world, window, Some(target));
        }

        FocusUpdate::Next => {
            focus_next(world, window, true);
        }

        FocusUpdate::Previous => {
            focus_next(world, window, false);
        }
    }
}

pub fn focus_next(world: &mut World, window: WindowId, forward: bool) {
    if let Some(window) = world.window(window) {
        let focused = find_next_focusable(&world.arena, window, forward);
        set_focused(world, window.id, focused);
    }
}

pub fn set_focused(world: &mut World, window: WindowId, target: Option<WidgetId>) -> bool {
    let window_id = window;

    let Some(window) = world.window_mut(window_id) else {
        debug_panic!("failed to get_window");
        return false;
    };

    if window.focused == target {
        return false;
    }

    let current = window.focused;
    window.focused = target;

    // remove focus from the currently focused widget
    if let Some(current) = current
        && let Some(mut widget) = world.get_widget_mut(current)
    {
        widget.set_focused(false);

        // if there is no new focused widget, end the ime session
        if widget.cx.state.accepts_text && target.is_none() {
            widget.cx.world.signal(Signal::Ime(ImeSignal::End));
        }
    }

    // emit an ime end event for the currently focused widget
    if let Some(current) = current
        && world
            .get_widget(current)
            .is_some_and(|widget| widget.cx.state.accepts_text)
    {
        let event = TextEvent::Ime(ImeEvent::End);
        text::send_text_event(world, window_id, current, &event);
    }

    if let Some(target) = target {
        // set focused for the new target
        let rect = if let Some(mut widget) = world.get_widget_mut(target) {
            widget.set_focused(true);

            // if the target accepts text start an ime session, this will serve to restart it if
            // an ime session already exists
            if widget.cx.state.accepts_text {
                widget.cx.world.signal(Signal::Ime(ImeSignal::Start));
            }

            Some(Rect::min_size(
                Point::ORIGIN,
                widget.cx.size(),
            ))
        } else {
            None
        };

        // if the target accepts text we send it an ime start event
        if world
            .get_widget(target)
            .is_some_and(|widget| widget.cx.state.accepts_text)
        {
            let event = TextEvent::Ime(ImeEvent::Start);
            text::send_text_event(world, window_id, target, &event);
        }

        // scroll to the target widget
        if let Some(rect) = rect {
            scroll::scroll_to(world, target, rect);
        }
    }

    true
}

fn find_first_focusable(arena: &Arena, window: &Window, forward: bool) -> Option<WidgetId> {
    if forward {
        for layer in window.layers.iter() {
            if let Some(focusable) = find_first_focusable_from(arena, layer.root, forward) {
                return Some(focusable);
            }
        }
    } else {
        for layer in window.layers.iter().rev() {
            if let Some(focusable) = find_first_focusable_from(arena, layer.root, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn find_first_focusable_from(arena: &Arena, widget: WidgetId, forward: bool) -> Option<WidgetId> {
    let state = arena.get_state(widget.index)?;

    if state.accepts_focus {
        return Some(widget);
    }

    if forward {
        for &child in state.children.iter() {
            if let Some(focusable) = find_first_focusable_from(arena, child, forward) {
                return Some(focusable);
            }
        }
    } else {
        for &child in state.children.iter().rev() {
            if let Some(focusable) = find_first_focusable_from(arena, child, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn find_next_focusable(arena: &Arena, window: &Window, forward: bool) -> Option<WidgetId> {
    if window.focused.is_none() {
        return find_first_focusable(arena, window, forward);
    }

    if forward {
        let mut layers = window.layers().iter();

        for layer in layers.by_ref() {
            let state = arena.get_state(layer.root.index)?;

            if state.has_focused {
                if let Some(focusable) = find_next_focusable_from(arena, layer.root, forward) {
                    return Some(focusable);
                } else {
                    break;
                }
            }
        }

        for layer in layers {
            if let Some(focusable) = find_first_focusable_from(arena, layer.root, forward) {
                return Some(focusable);
            }
        }
    } else {
        let mut layers = window.layers().iter().rev();

        for layer in layers.by_ref() {
            let state = arena.get_state(layer.root.index)?;

            if state.has_focused {
                if let Some(focusable) = find_next_focusable_from(arena, layer.root, forward) {
                    return Some(focusable);
                } else {
                    break;
                }
            }
        }

        for layer in layers {
            if let Some(focusable) = find_first_focusable_from(arena, layer.root, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn find_next_focusable_from(arena: &Arena, current: WidgetId, forward: bool) -> Option<WidgetId> {
    let state = arena.get_state(current.index)?;

    if !state.has_focused {
        return find_first_focusable_from(arena, current, forward);
    }

    if forward {
        let mut children = state.children.iter().copied();

        for child in children.by_ref() {
            let child_state = arena.get_state(child.index)?;

            if child_state.has_focused {
                if let Some(focusable) = find_next_focusable_from(arena, child, forward) {
                    return Some(focusable);
                } else {
                    break;
                }
            }
        }

        for child in children {
            if let Some(focusable) = find_first_focusable_from(arena, child, forward) {
                return Some(focusable);
            }
        }
    } else {
        let mut children = state.children.iter().copied().rev();

        for child in children.by_ref() {
            let child_state = arena.get_state(child.index)?;

            if child_state.has_focused {
                if let Some(focusable) = find_next_focusable_from(arena, child, forward) {
                    return Some(focusable);
                } else {
                    break;
                }
            }
        }

        for child in children {
            if let Some(focusable) = find_first_focusable_from(arena, child, forward) {
                return Some(focusable);
            }
        }
    }

    None
}
