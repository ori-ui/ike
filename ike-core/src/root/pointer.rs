use crate::{
    Arena, BuildCx, CursorIcon, EventCx, Point, Pointer, PointerButton, PointerButtonEvent,
    PointerEvent, PointerId, PointerMoveEvent, PointerPropagate, PointerScrollEvent, Rect, Root,
    ScrollDelta, WidgetId, WindowId, context::FocusUpdate, root::focus,
};

impl Root {
    pub fn pointer_entered(&mut self, window: WindowId, id: PointerId) -> bool {
        let Some(window) = self.state.get_window_mut(window) else {
            return false;
        };

        window.pointers.push(Pointer {
            id,
            position: Point::all(0.0),
        });

        false
    }

    pub fn pointer_left(&mut self, window: WindowId, id: PointerId) -> bool {
        let Some(window) = self.state.get_window_mut(window) else {
            return false;
        };

        window.pointers.retain(|p| p.id == id);

        if let Some(hovered) = find_hovered(&self.arena, window.contents)
            && let Some(mut widget) = self.get_mut(hovered)
        {
            widget.set_hovered(false);
        }

        false
    }

    pub fn pointer_moved(&mut self, window: WindowId, id: PointerId, position: Point) -> bool {
        let window_id = window;
        let Some(window) = self.state.get_window_mut(window) else {
            return false;
        };

        if let Some(pointer) = window.pointers.iter_mut().find(|p| p.id == id) {
            pointer.position = position;
        }

        let window_contents = window.contents;
        let hovered = update_hovered(
            self,
            window_id,
            window_contents,
            position,
        );

        // if there is an active widget, target it with the event
        if let Some(target) = find_active(&self.arena, window_contents) {
            let event = PointerMoveEvent {
                pointer: id,
                position,
            };
            let event = PointerEvent::Move(event);

            match send_pointer_event(self, window_contents, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Stop => true,

                PointerPropagate::Capture => {
                    tracing::warn!(
                        "pointer capture is only valid in response to PointerEvent::Down(..)"
                    );

                    true
                }
            }
        } else if let Some(target) = hovered {
            let event = PointerMoveEvent {
                pointer: id,
                position,
            };
            let event = PointerEvent::Move(event);

            match send_pointer_event(self, window_contents, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Stop => true,

                PointerPropagate::Capture => {
                    tracing::warn!(
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
        id: PointerId,
        button: PointerButton,
        pressed: bool,
    ) -> bool {
        let window_id = window;
        let Some(window) = self.state.get_window_mut(window) else {
            return false;
        };

        let Some(pointer) = window.pointers.iter_mut().find(|p| p.id == id) else {
            return false;
        };

        let window_contents = window.contents;
        let pointer_position = pointer.position;

        let handled = if let Some(target) = find_pointer_target(&self.arena, window_contents) {
            let event = PointerButtonEvent {
                button,
                position: pointer.position,
                pointer: id,
            };
            let event = match pressed {
                true => PointerEvent::Down(event),
                false => PointerEvent::Up(event),
            };

            let handled = match send_pointer_event(self, window_contents, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Stop => true,

                PointerPropagate::Capture if pressed => {
                    if let Some(mut widget) = self.get_mut(target) {
                        widget.set_active(true);
                    }

                    true
                }

                PointerPropagate::Capture => {
                    panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
                }
            };

            let target_is_active = self
                .arena
                .get_state(target.index)
                .is_some_and(|s| s.is_active);

            if !pressed && target_is_active {
                if let Some(mut widget) = self.get_mut(target) {
                    widget.set_active(false);
                }

                update_hovered(
                    self,
                    window_id,
                    window_contents,
                    pointer_position,
                );
            }

            handled
        } else {
            false
        };

        if let Some(focused) = focus::find_focused(&self.arena, window_contents)
            && let Some(mut widget) = self.get_mut(focused)
            && button == PointerButton::Primary
            && pressed
        {
            let local = widget.cx.global_transform().inverse() * pointer_position;

            if !Rect::min_size(Point::ORIGIN, widget.cx.size()).contains(local) {
                widget.set_focused(false);
            }
        }

        handled
    }

    pub fn pointer_scrolled(
        &mut self,
        window: WindowId,
        delta: ScrollDelta,
        id: PointerId,
    ) -> bool {
        let Some(window) = self.state.get_window_mut(window) else {
            return false;
        };

        let Some(pointer) = window.pointers.iter_mut().find(|p| p.id == id) else {
            return false;
        };

        let Some(target) = find_pointer_target(&self.arena, window.contents) else {
            return false;
        };

        let event = PointerEvent::Scroll(PointerScrollEvent {
            position: pointer.position,
            pointer: id,
            delta,
        });

        let window_contents = window.contents;
        match send_pointer_event(self, window_contents, target, &event) {
            PointerPropagate::Bubble => false,
            PointerPropagate::Stop => true,

            PointerPropagate::Capture => {
                panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
            }
        }
    }
}

fn find_pointer_target(arena: &Arena, root: WidgetId) -> Option<WidgetId> {
    if let Some(target) = find_active(arena, root) {
        Some(target)
    } else {
        find_hovered(arena, root)
    }
}

fn find_active(arena: &Arena, id: WidgetId) -> Option<WidgetId> {
    let state = arena.get_state(id.index)?;

    if state.is_active {
        return Some(id);
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

fn find_hovered(arena: &Arena, id: WidgetId) -> Option<WidgetId> {
    let state = arena.get_state(id.index)?;

    if state.is_hovered {
        return Some(id);
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

pub fn update_hovered(
    root: &mut Root,
    window: WindowId,
    id: WidgetId,
    point: Point,
) -> Option<WidgetId> {
    if let Some(active) = find_active(&root.arena, id)
        && let Some(mut widget) = root.arena.get_mut(&mut root.state, active)
    {
        widget.cx.root.set_window_cursor(window, widget.cx.cursor());

        let local = widget.cx.global_transform().inverse() * point;
        let is_hovered = Rect::min_size(Point::ORIGIN, widget.cx.size()).contains(local);

        if is_hovered != widget.cx.is_hovered() {
            widget.set_hovered(is_hovered);
        }

        return is_hovered.then_some(active);
    }

    let current = find_hovered(&root.arena, id);
    let hovered = find_widget_at(&root.arena, id, point);

    if current != hovered {
        if let Some(current) = current
            && let Some(mut widget) = root.arena.get_mut(&mut root.state, current)
        {
            widget.set_hovered(false);
        }

        if let Some(hovered) = hovered
            && let Some(mut widget) = root.arena.get_mut(&mut root.state, hovered)
        {
            widget.set_hovered(true);
        }
    }

    if let Some(hovered) = hovered
        && let Some(widget) = root.get_mut(hovered)
    {
        widget.cx.root.set_window_cursor(window, widget.cx.cursor());
    } else {
        root.state.set_window_cursor(window, CursorIcon::Default);
    }

    hovered
}

fn find_widget_at(arena: &Arena, id: WidgetId, point: Point) -> Option<WidgetId> {
    let state = arena.get_state(id.index)?;
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
        Some(id)
    } else {
        None
    }
}

fn send_pointer_event(
    root: &mut Root,
    root_widget: WidgetId,
    target: WidgetId,
    event: &PointerEvent,
) -> PointerPropagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = PointerPropagate::Bubble;

    let _span = tracing::info_span!("key_event");

    while let Some(id) = current
        && let Some(widget) = root.get_mut(id)
    {
        if let PointerPropagate::Bubble = propagate {
            let mut cx = EventCx {
                root:  widget.cx.root,
                arena: widget.cx.arena,
                state: widget.cx.state,
                focus: &mut focus,
            };

            propagate = widget.widget.on_pointer_event(&mut cx, event);
        }

        current = widget.cx.parent();
    }

    if let Some(mut target) = root.get_mut(target) {
        target.cx.propagate_state();
    }

    focus::update_focus(root, root_widget, focus);

    propagate
}
