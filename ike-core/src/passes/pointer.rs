use crate::{
    CursorIcon, Point, Pointer, PointerButton, PointerButtonEvent, PointerEvent, PointerId,
    PointerMoveEvent, PointerPropagate, PointerScrollEvent, ScrollDelta, WidgetId, WindowId, World,
    debug::debug_panic, passes,
};

pub(crate) fn entered(world: &mut World, window: WindowId, pointer: PointerId) -> bool {
    if let Some(window) = world.state.window_mut(window) {
        window.pointers.push(Pointer {
            id:       pointer,
            position: Point::ORIGIN,
            hovering: None,
            capturer: None,
        });
    }

    true
}

pub(crate) fn left(world: &mut World, window: WindowId, pointer: PointerId) -> bool {
    if let Some(window) = world.state.window_mut(window)
        && let Some(index) = window.pointers.iter().position(|p| p.id == pointer)
    {
        let pointer = window.pointers.swap_remove(index);

        if let Some(hovered) = pointer.hovering
            && let Some(mut widget) = world.widget_mut(hovered)
        {
            widget.set_hovered(false);
        }
    }

    true
}

pub(crate) fn moved(
    world: &mut World,
    window: WindowId,
    pointer: PointerId,
    position: Point,
) -> bool {
    let window_id = window;
    let pointer_id = pointer;

    let capturer = if let Some(window) = world.state.window_mut(window_id)
        && let Some(pointer) = window.pointer_mut(pointer_id)
    {
        pointer.position = position;
        pointer.capturer
    } else {
        None
    };

    let hovered = update_pointer_hovered(world, window_id, pointer_id);

    if let Some(target) = capturer.or(hovered) {
        let event = PointerMoveEvent {
            pointer: pointer_id,
            position,
        };
        let event = PointerEvent::Move(event);

        match send_event(world, window_id, target, &event) {
            PointerPropagate::Bubble => false,
            PointerPropagate::Handled => true,

            PointerPropagate::Capture => {
                debug_panic!("pointer capture is only valid in response to PointerEvent::Down(..)");

                true
            }
        }
    } else {
        false
    }
}

pub(crate) fn pressed(
    world: &mut World,
    window: WindowId,
    pointer: PointerId,
    button: PointerButton,
    pressed: bool,
) -> bool {
    let window_id = window;
    let pointer_id = pointer;

    let Some(window) = world.window(window_id) else {
        return false;
    };

    let Some(pointer) = window.pointer(pointer_id) else {
        return false;
    };

    let position = pointer.position;
    let handled = pointer.target().is_some_and(|target| {
        let event = PointerButtonEvent {
            button,
            position,
            pointer: pointer_id,
        };
        let event = match pressed {
            true => PointerEvent::Down(event),
            false => PointerEvent::Up(event),
        };

        let handled = match send_event(world, window_id, target, &event) {
            PointerPropagate::Bubble => false,
            PointerPropagate::Handled => true,

            PointerPropagate::Capture if pressed => {
                if let Some(mut widget) = world.widget_mut(target) {
                    widget.set_active(true);
                }

                true
            }

            PointerPropagate::Capture => {
                debug_panic!("pointer capture is only valid in response to PointerEvent::Down(..)");

                true
            }
        };

        let target_is_active = world.widget(target).is_some_and(|t| t.cx.is_active());

        if !pressed && target_is_active {
            if let Some(mut widget) = world.widget_mut(target) {
                widget.set_active(false);
            }

            update_pointer_hovered(world, window_id, pointer_id);
        }

        handled
    });

    if pressed
        && !handled
        && let Some(window) = world.window(window_id)
        && let Some(focused) = window.focused
        && world.widget(focused).is_some_and(|widget| {
            let local = widget.cx.global_transform().inverse() * position;
            !widget.cx.rect().contains(local)
        })
    {
        passes::focus::transfer(world, window_id, None);
    }

    handled
}

pub(crate) fn scrolled(
    world: &mut World,
    window: WindowId,
    pointer: PointerId,
    delta: ScrollDelta,
) -> bool {
    let window_id = window;
    let pointer_id = pointer;

    let Some(window) = world.state.window_mut(window_id) else {
        return false;
    };

    let Some(pointer) = window.pointer_mut(pointer_id) else {
        return false;
    };

    let Some(target) = pointer.target() else {
        return false;
    };

    let event = PointerEvent::Scroll(PointerScrollEvent {
        position: pointer.position,
        pointer: pointer_id,
        delta,
    });

    match send_event(world, window_id, target, &event) {
        PointerPropagate::Bubble => false,
        PointerPropagate::Handled => true,

        PointerPropagate::Capture => {
            tracing::error!("pointer capture is only valid in response to PointerEvent::Down(..)");

            true
        }
    }
}

pub(crate) fn send_event(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    event: &PointerEvent,
) -> PointerPropagate {
    passes::event::send_event(
        world,
        window,
        target,
        PointerPropagate::Bubble,
        |widget, cx| widget.on_pointer_event(cx, event),
    )
}

pub(crate) fn update_window_hovered(world: &mut World, window: WindowId) {
    let window_id = window;
    if let Some(window) = world.window(window) {
        let pointers: Vec<_> = window.pointers.iter().map(|p| p.id).collect();

        for pointer in pointers {
            update_pointer_hovered(world, window_id, pointer);
        }
    }
}

pub(crate) fn update_pointer_hovered(
    world: &mut World,
    window: WindowId,
    pointer: PointerId,
) -> Option<WidgetId> {
    let window_id = window;
    let pointer_id = pointer;

    let window = world.window(window_id)?;
    let pointer = window.pointer(pointer_id)?;

    let position = pointer.position;

    if let Some(active) = pointer.capturer {
        if let Some(mut widget) = world.widget_mut(active) {
            (widget.cx.world).set_window_cursor(window_id, widget.cx.cursor());

            let local = widget.cx.global_transform().inverse() * position;
            let is_hovered = widget.cx.rect().contains(local);
            widget.set_hovered(is_hovered);

            return is_hovered.then_some(active);
        } else {
            return None;
        }
    }

    let hovered = passes::query::find_widget_at(world, window, position);

    if pointer.hovering != hovered {
        if let Some(current) = pointer.hovering
            && let Some(mut widget) = world.widget_mut(current)
        {
            widget.set_hovered(false);
        }

        if let Some(hovered) = hovered
            && let Some(mut widget) = world.widget_mut(hovered)
        {
            widget.set_hovered(true);
        }
    }

    if let Some(window) = world.window_mut(window_id)
        && let Some(pointer) = window.pointer_mut(pointer_id)
    {
        pointer.hovering = hovered;
    }

    if let Some(hovered) = hovered
        && let Some(widget) = world.widget_mut(hovered)
    {
        (widget.cx.world).set_window_cursor(window_id, widget.cx.cursor());
    } else {
        (world.state).set_window_cursor(window_id, CursorIcon::Default);
    }

    hovered
}
