use crate::{
    App, CursorIcon, EventCx, Offset, Point, Pointer, PointerButton, PointerButtonEvent,
    PointerEvent, PointerId, PointerMoveEvent, PointerPropagate, PointerScrollEvent, Rect, Tree,
    WidgetId, Window, WindowId, app::focus, context::FocusUpdate,
};

impl App {
    pub fn pointer_entered(&mut self, window: WindowId, id: PointerId) -> bool {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return false;
        };

        window.pointers.push(Pointer {
            id,
            position: Point::all(0.0),
        });

        false
    }

    pub fn pointer_left(&mut self, window: WindowId, id: PointerId) -> bool {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return false;
        };

        window.pointers.retain(|p| p.id == id);

        if let Some(hovered) = find_hovered(&self.tree, window.contents)
            && let Some(mut widget) = self.tree.get_mut(hovered)
        {
            widget.set_hovered(false);
        }

        false
    }

    pub fn pointer_moved(&mut self, window: WindowId, id: PointerId, position: Point) -> bool {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return false;
        };

        if let Some(pointer) = window.pointers.iter_mut().find(|p| p.id == id) {
            pointer.position = position;
        }

        let hovered = update_hovered(&mut self.tree, window, window.contents, position);

        // if there is an active widget, target it with the event
        if let Some(target) = find_active(&self.tree, window.contents) {
            let event = PointerMoveEvent {
                pointer: id,
                position,
            };
            let event = PointerEvent::Move(event);

            let contents = window.contents;
            let window = window.id;
            match send_pointer_event(self, window, contents, target, &event) {
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

            let contents = window.contents;
            let window = window.id;
            match send_pointer_event(self, window, contents, target, &event) {
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

    pub fn pointer_button(
        &mut self,
        window: WindowId,
        id: PointerId,
        button: PointerButton,
        pressed: bool,
    ) -> bool {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return false;
        };

        let Some(pointer) = window.pointers.iter_mut().find(|p| p.id == id) else {
            return false;
        };

        let contents = window.contents;
        let position = pointer.position;

        let handled = if let Some(target) = find_pointer_target(&self.tree, contents) {
            let event = PointerButtonEvent {
                button,
                position: pointer.position,
                pointer: id,
            };
            let event = match pressed {
                true => PointerEvent::Down(event),
                false => PointerEvent::Up(event),
            };

            let window_id = window.id;
            let handled = match send_pointer_event(self, window_id, contents, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Stop => true,

                PointerPropagate::Capture if pressed => {
                    if let Some(mut widget) = self.tree.get_mut(target) {
                        widget.set_active(true);
                    }

                    true
                }

                PointerPropagate::Capture => {
                    panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
                }
            };

            if !pressed && self.tree.get_state_unchecked(target.index).is_active {
                if let Some(mut widget) = self.tree.get_mut(target) {
                    widget.set_active(false);
                }

                if let Some(window) = self.windows.iter_mut().find(|w| w.id == window_id) {
                    update_hovered(&mut self.tree, window, contents, position);
                }
            }

            handled
        } else {
            false
        };

        if let Some(focused) = focus::find_focused(&self.tree, contents)
            && let Some(mut widget) = self.tree.get_mut(focused)
            && button == PointerButton::Primary
            && pressed
        {
            let local = widget.state().global_transform.inverse() * position;

            if !Rect::min_size(Point::ORIGIN, widget.state().size).contains(local) {
                widget.set_focused(false);
            }
        }

        handled
    }

    pub fn pointer_scrolled(&mut self, window: WindowId, delta: Offset, id: PointerId) -> bool {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return false;
        };

        let Some(pointer) = window.pointers.iter_mut().find(|p| p.id == id) else {
            return false;
        };

        let Some(target) = find_pointer_target(&self.tree, window.contents) else {
            return false;
        };

        let event = PointerEvent::Scroll(PointerScrollEvent {
            position: pointer.position,
            pointer: id,
            delta,
        });

        let contents = window.contents;
        let window_id = window.id;
        match send_pointer_event(self, window_id, contents, target, &event) {
            PointerPropagate::Bubble => false,
            PointerPropagate::Stop => true,

            PointerPropagate::Capture => {
                panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
            }
        }
    }
}

fn find_pointer_target(tree: &Tree, root: WidgetId) -> Option<WidgetId> {
    if let Some(target) = find_active(tree, root) {
        Some(target)
    } else {
        find_hovered(tree, root)
    }
}

fn find_active(tree: &Tree, id: WidgetId) -> Option<WidgetId> {
    let state = tree.get_state_unchecked(id.index);

    if state.is_active {
        return Some(id);
    }

    for &child in &state.children {
        let child_state = tree.get_state_unchecked(child.index);

        if child_state.has_active {
            return find_active(tree, child);
        }
    }

    None
}

fn find_hovered(tree: &Tree, id: WidgetId) -> Option<WidgetId> {
    let state = tree.get_state_unchecked(id.index);

    if state.is_hovered {
        return Some(id);
    }

    for &child in &state.children {
        let child_state = tree.get_state_unchecked(child.index);

        if child_state.has_hovered {
            return find_hovered(tree, child);
        }
    }

    None
}

pub fn update_hovered(
    tree: &mut Tree,
    window: &mut Window,
    id: WidgetId,
    point: Point,
) -> Option<WidgetId> {
    if let Some(active) = find_active(tree, id)
        && let Some(mut widget) = tree.get_mut(active)
    {
        window.cursor = widget.cursor();

        let local = widget.state().global_transform.inverse() * point;
        let is_hovered = Rect::min_size(Point::ORIGIN, widget.size()).contains(local);

        if is_hovered != widget.is_hovered() {
            widget.set_hovered(is_hovered);
        }

        return is_hovered.then_some(active);
    }

    let current = find_hovered(tree, id);
    let hovered = find_widget_at(tree, id, point);

    if current != hovered {
        if let Some(current) = current
            && let Some(mut widget) = tree.get_mut(current)
        {
            widget.set_hovered(false);
        }

        if let Some(hovered) = hovered
            && let Some(mut widget) = tree.get_mut(hovered)
        {
            widget.set_hovered(true);
        }
    }

    if let Some(hovered) = hovered
        && let Some(widget) = tree.get(hovered)
    {
        window.cursor = widget.state().cursor;
    } else {
        window.cursor = CursorIcon::Default;
    }

    hovered
}

fn find_widget_at(tree: &Tree, id: WidgetId, point: Point) -> Option<WidgetId> {
    let state = tree.get_state_unchecked(id.index);
    let local = state.global_transform.inverse() * point;

    if !Rect::min_size(Point::ORIGIN, state.size).contains(local) {
        return None;
    }

    for &child in &state.children {
        if let Some(id) = find_widget_at(tree, child, point) {
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
    app: &mut App,
    window: WindowId,
    root: WidgetId,
    target: WidgetId,
    event: &PointerEvent,
) -> PointerPropagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = PointerPropagate::Bubble;

    let _span = tracing::info_span!("key_event");

    while let Some(id) = current {
        let mut widget = app.tree.get_mut(id).unwrap();

        if let PointerPropagate::Bubble = propagate {
            let (tree, widget, state) = widget.split();
            let mut cx = EventCx {
                window,
                windows: &mut app.windows,
                tree,
                state,
                id,
                focus: &mut focus,
            };

            propagate = widget.on_pointer_event(&mut cx, event);
        }

        current = widget.state().parent;
    }

    if let Some(mut target) = app.tree.get_mut(target) {
        target.propagate_state();
    }

    focus::update_focus(&mut app.tree, root, focus);

    propagate
}
