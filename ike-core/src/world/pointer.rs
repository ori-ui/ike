use crate::{
    CursorIcon, EventCx, Point, Pointer, PointerButton, PointerButtonEvent, PointerEvent,
    PointerId, PointerMoveEvent, PointerPropagate, PointerScrollEvent, Rect, ScrollDelta, WidgetId,
    WindowId, World,
    context::FocusUpdate,
    world::{focus, query},
};

impl World {
    pub fn pointer_entered(&mut self, window: WindowId, pointer: PointerId) -> bool {
        self.handle_updates();

        let Some(window) = self.state.window_mut(window) else {
            return false;
        };

        window.pointers.push(Pointer {
            id:       pointer,
            position: Point::all(0.0),
            hovering: None,
            capturer: None,
        });

        false
    }

    pub fn pointer_left(&mut self, window: WindowId, pointer: PointerId) -> bool {
        self.handle_updates();

        let Some(window) = self.state.window_mut(window) else {
            return false;
        };

        let Some(index) = window.pointers.iter().position(|p| p.id == pointer) else {
            return false;
        };

        let pointer = window.pointers.swap_remove(index);

        if let Some(hovered) = pointer.hovering
            && let Some(mut widget) = self.widget_mut(hovered)
        {
            widget.set_hovered(false);
        }

        false
    }

    pub fn pointer_moved(&mut self, window: WindowId, pointer: PointerId, position: Point) -> bool {
        self.handle_updates();

        let window_id = window;
        let pointer_id = pointer;

        let capturer = if let Some(window) = self.state.window_mut(window_id)
            && let Some(pointer) = window.pointer_mut(pointer_id)
        {
            pointer.position = position;
            pointer.capturer
        } else {
            None
        };

        let hovered = update_pointer_hovered(self, window_id, pointer_id);

        // if there is an active widget, target it with the event
        if let Some(target) = capturer {
            let event = PointerMoveEvent {
                pointer: pointer_id,
                position,
            };
            let event = PointerEvent::Move(event);

            match send_pointer_event(self, window_id, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Handled => true,

                PointerPropagate::Capture => {
                    tracing::error!(
                        "pointer capture is only valid in response to PointerEvent::Down(..)"
                    );

                    true
                }
            }
        } else if let Some(target) = hovered {
            let event = PointerMoveEvent {
                pointer: pointer_id,
                position,
            };
            let event = PointerEvent::Move(event);

            match send_pointer_event(self, window_id, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Handled => true,

                PointerPropagate::Capture => {
                    tracing::error!(
                        "pointer capture is only valid in response to PointerEvent::Down(..)"
                    );

                    true
                }
            }
        } else {
            false
        }
    }

    pub fn pointer_pressed(
        &mut self,
        window: WindowId,
        pointer: PointerId,
        button: PointerButton,
        pressed: bool,
    ) -> bool {
        self.handle_updates();

        let window_id = window;
        let pointer_id = pointer;

        let Some(window) = self.state.window_mut(window) else {
            return false;
        };

        let Some(pointer) = window.pointer_mut(pointer) else {
            return false;
        };

        let pointer_position = pointer.position;

        let handled = if let Some(target) = pointer.target() {
            let event = PointerButtonEvent {
                button,
                position: pointer_position,
                pointer: pointer_id,
            };
            let event = match pressed {
                true => PointerEvent::Down(event),
                false => PointerEvent::Up(event),
            };

            let handled = match send_pointer_event(self, window_id, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Handled => true,

                PointerPropagate::Capture if pressed => {
                    if let Some(mut widget) = self.widget_mut(target) {
                        widget.set_active(true);
                    }

                    true
                }

                PointerPropagate::Capture => {
                    tracing::error!(
                        "pointer capture is only valid in response to PointerEvent::Down(..)"
                    );

                    true
                }
            };

            let target_is_active = self
                .arena
                .get_state(target.index)
                .is_some_and(|s| s.is_active);

            if !pressed && target_is_active {
                if let Some(mut widget) = self.widget_mut(target) {
                    widget.set_active(false);
                }

                update_pointer_hovered(self, window_id, pointer_id);
            }

            handled
        } else {
            false
        };

        if button == PointerButton::Primary
            && pressed
            && !handled
            && let Some(window) = self.window(window_id)
            && let Some(focused) = window.focused
            && self.widget(focused).is_some_and(|widget| {
                let local = widget.cx.global_transform().inverse() * pointer_position;
                !Rect::min_size(Point::ORIGIN, widget.cx.size()).contains(local)
            })
        {
            focus::set_focused(self, window_id, None);
        }

        handled
    }

    pub fn pointer_scrolled(
        &mut self,
        window: WindowId,
        pointer: PointerId,
        delta: ScrollDelta,
    ) -> bool {
        self.handle_updates();

        let window_id = window;
        let pointer_id = pointer;

        let Some(window) = self.state.window_mut(window_id) else {
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

        match send_pointer_event(self, window_id, target, &event) {
            PointerPropagate::Bubble => false,
            PointerPropagate::Handled => true,

            PointerPropagate::Capture => {
                tracing::error!(
                    "pointer capture is only valid in response to PointerEvent::Down(..)"
                );

                true
            }
        }
    }
}

pub fn update_window_hovered(world: &mut World, window: WindowId) {
    let window_id = window;

    let Some(window) = world.window(window) else {
        return;
    };

    let pointers = window
        .pointers
        .iter()
        .map(|pointer| pointer.id)
        .collect::<Vec<_>>();

    for pointer in pointers {
        update_pointer_hovered(world, window_id, pointer);
    }
}

pub fn update_pointer_hovered(
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
        if let Some(mut widget) = world.arena.get_mut(&mut world.state, active) {
            (widget.cx.world).set_window_cursor(window_id, widget.cx.cursor());

            let local = widget.cx.global_transform().inverse() * position;
            let is_hovered = widget.cx.rect().contains(local);

            if is_hovered != widget.cx.is_hovered() {
                widget.set_hovered(is_hovered);
            }

            return is_hovered.then_some(active);
        } else {
            tracing::error!("failed getting active widget");

            return None;
        }
    }

    let hovered = query::find_widget_at(world, window, position);

    if pointer.hovering != hovered {
        if let Some(current) = pointer.hovering
            && let Some(mut widget) = world.arena.get_mut(&mut world.state, current)
        {
            widget.set_hovered(false);
        }

        if let Some(hovered) = hovered
            && let Some(mut widget) = world.arena.get_mut(&mut world.state, hovered)
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

fn send_pointer_event(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    event: &PointerEvent,
) -> PointerPropagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = PointerPropagate::Bubble;

    let _span = tracing::info_span!("key_event");

    while let Some(id) = current
        && let Some(widget) = world.widget_mut(id)
        && let PointerPropagate::Bubble = propagate
    {
        let mut cx = EventCx {
            world: widget.cx.world,
            arena: widget.cx.arena,
            state: widget.cx.state,
            focus: &mut focus,
        };

        propagate = widget.widget.on_pointer_event(&mut cx, event);
        current = widget.cx.parent();
    }

    world.propagate_state(target);
    focus::update_focus(world, window, focus);

    propagate
}
