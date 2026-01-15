use std::mem;

use crate::{
    ImeEvent, ImeSignal, Signal, TextEvent, WidgetId, Window, WindowId, World,
    context::FocusUpdate, passes,
};

pub(crate) fn update(world: &mut World, window: WindowId, update: FocusUpdate) {
    match update {
        FocusUpdate::None => {}

        FocusUpdate::Take => {
            transfer(world, window, None);
        }

        FocusUpdate::Target(target) => {
            transfer(world, window, Some(target));
        }

        FocusUpdate::Next => {
            next(world, window, true);
        }

        FocusUpdate::Previous => {
            next(world, window, false);
        }
    }
}

pub(crate) fn next(world: &mut World, window: WindowId, forward: bool) {
    if let Some(window) = world.window(window) {
        let focused = find_next(world, window, forward);
        transfer(world, window.id(), focused);
    }
}

pub(crate) fn transfer(world: &mut World, window: WindowId, target: Option<WidgetId>) {
    let window_id = window;

    let Some(window) = world.window_mut(window_id) else {
        return;
    };

    if window.focused == target {
        return;
    }

    let current = mem::replace(&mut window.focused, target);

    // remove focus from the current widget
    if let Some(current) = current
        && let Ok(mut widget) = world.widget_mut(current)
    {
        widget.set_focused(false);

        if widget.cx.hierarchy.accepts_text() && target.is_none() {
            widget.cx.world.emit_signal(Signal::Ime(ImeSignal::End));
        }
    }

    // emit ime end event for currently focused widget
    if let Some(current) = current
        && world
            .widget(current)
            .is_ok_and(|current| current.cx.hierarchy.accepts_text())
    {
        let event = TextEvent::Ime(ImeEvent::End);
        passes::text::send_event(world, window_id, current, &event);
    }

    if let Some(target) = target {
        let rect = world.widget_mut(target).map(|mut widget| {
            widget.set_focused(true);

            if widget.cx.hierarchy.accepts_text() {
                widget.cx.world.emit_signal(Signal::Ime(ImeSignal::Start));
            }

            widget.cx.rect()
        });

        if world
            .widget(target)
            .is_ok_and(|widget| widget.cx.hierarchy.accepts_text())
        {
            let event = TextEvent::Ime(ImeEvent::Start);
            passes::text::send_event(world, window_id, target, &event);
        }

        if let Ok(rect) = rect {
            passes::scroll::scroll_to(world, target, rect);
        }
    }
}

fn find_first(world: &World, window: &Window, forward: bool) -> Option<WidgetId> {
    if forward {
        for layer in window.layers.iter() {
            if let Some(focusable) = find_first_from(world, layer.widget, forward) {
                return Some(focusable);
            }
        }
    } else {
        for layer in window.layers.iter().rev() {
            if let Some(focusable) = find_first_from(world, layer.widget, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn find_first_from(world: &World, widget: WidgetId, forward: bool) -> Option<WidgetId> {
    let widget = world.widget(widget).ok()?;

    if widget.cx.hierarchy.accepts_focus() {
        return Some(widget.cx.id());
    }

    if forward {
        for &child in widget.cx.children().iter() {
            if let Some(focusable) = find_first_from(world, child, forward) {
                return Some(focusable);
            }
        }
    } else {
        for &child in widget.cx.children().iter().rev() {
            if let Some(focusable) = find_first_from(world, child, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn find_next(world: &World, window: &Window, forward: bool) -> Option<WidgetId> {
    if window.focused.is_none() {
        return find_first(world, window, forward);
    }

    if forward {
        let mut layers = window.layers().iter();

        for layer in layers.by_ref() {
            let widget = world.widget(layer.widget).ok()?;

            if widget.cx.hierarchy.has_focused() {
                if let Some(focusable) = find_next_from(world, layer.widget, forward) {
                    return Some(focusable);
                } else {
                    break;
                }
            }
        }

        for layer in layers {
            if let Some(focusable) = find_first_from(world, layer.widget, forward) {
                return Some(focusable);
            }
        }
    } else {
        let mut layers = window.layers().iter().rev();

        for layer in layers.by_ref() {
            let widget = world.widget(layer.widget).ok()?;

            if widget.cx.hierarchy.has_focused() {
                if let Some(focusable) = find_next_from(world, layer.widget, forward) {
                    return Some(focusable);
                } else {
                    break;
                }
            }
        }

        for layer in layers {
            if let Some(focusable) = find_first_from(world, layer.widget, forward) {
                return Some(focusable);
            }
        }
    }

    None
}

fn find_next_from(world: &World, current: WidgetId, forward: bool) -> Option<WidgetId> {
    let widget = world.widget(current).ok()?;

    if !widget.cx.hierarchy.has_focused() {
        return find_first_from(world, current, forward);
    }

    if forward {
        let mut children = widget.cx.children().iter().copied();

        for child in children.by_ref() {
            let child = world.widget(child).ok()?;

            if child.cx.hierarchy.has_focused() {
                if let Some(focusable) = find_next_from(world, child.cx.id(), forward) {
                    return Some(focusable);
                } else {
                    break;
                }
            }
        }

        for child in children {
            if let Some(focusable) = find_first_from(world, child, forward) {
                return Some(focusable);
            }
        }
    } else {
        let mut children = widget.cx.children().iter().copied().rev();

        for child in children.by_ref() {
            let child = world.widget(child).ok()?;

            if child.cx.hierarchy.has_focused() {
                if let Some(focusable) = find_next_from(world, child.cx.id(), forward) {
                    return Some(focusable);
                } else {
                    break;
                }
            }
        }

        for child in children {
            if let Some(focusable) = find_first_from(world, child, forward) {
                return Some(focusable);
            }
        }
    }

    None
}
