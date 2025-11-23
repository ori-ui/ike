use crate::{
    App, EventCx, Point, Pointer, PointerButton, PointerButtonEvent, PointerEvent, PointerId,
    PointerMoveEvent, PointerPropagate, Rect, Tree, WidgetId, WindowId, app::focus,
    context::FocusUpdate,
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

        if let Some(hovered) = find_hovered(&self.tree, window.content) {
            self.tree.set_hovered(hovered, false);
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

        // if there is an active widget, target it with the event
        if let Some(target) = find_active(&self.tree, window.content) {
            let event = PointerMoveEvent { id, position };
            let event = PointerEvent::Move(event);

            let content = window.content;
            let window = window.id;
            return match send_pointer_event(self, window, content, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Stop => true,

                PointerPropagate::Capture => {
                    panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
                }
            };
        }

        // if not, update the hovered state and target the hovered widget
        let hovered = update_hovered(&mut self.tree, window.content, position);

        if let Some(target) = hovered {
            let event = PointerMoveEvent { id, position };
            let event = PointerEvent::Move(event);

            let content = window.content;
            let window = window.id;
            match send_pointer_event(self, window, content, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Stop => true,

                PointerPropagate::Capture => {
                    panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
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

        if let Some(target) = find_pointer_target(&self.tree, window.content) {
            let event = PointerButtonEvent {
                button,
                position: pointer.position,
                id,
            };
            let event = match pressed {
                true => PointerEvent::Down(event),
                false => PointerEvent::Up(event),
            };

            let content = window.content;
            let position = pointer.position;
            let window = window.id;
            let handled = match send_pointer_event(self, window, content, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Stop => true,

                PointerPropagate::Capture if pressed => {
                    self.tree.set_active(target, true);

                    true
                }

                PointerPropagate::Capture => {
                    panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
                }
            };

            if !pressed && self.tree.get_state_unchecked(target.index).is_active {
                self.tree.set_active(target, false);
                update_hovered(&mut self.tree, content, position);
            }

            if let Some(focused) = focus::find_focused(&self.tree, content)
                && button == PointerButton::Primary
                && pressed
            {
                let state = self.tree.get_state_unchecked(focused.index);
                let local = state.global_transform.inverse() * position;

                if !Rect::min_size(Point::ORIGIN, state.size).contains(local) {
                    self.tree.set_focused(focused, false);
                }
            }

            handled
        } else {
            false
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

fn update_hovered(tree: &mut Tree, id: WidgetId, point: Point) -> Option<WidgetId> {
    let current = find_hovered(tree, id);
    let hovered = find_widget_at(tree, id, point);

    if current != hovered {
        if let Some(current) = current {
            tree.set_hovered(current, false);
        }

        if let Some(hovered) = hovered {
            tree.set_hovered(hovered, true);
        }
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

    while let Some(id) = current {
        app.with_entry(id, |app, widget, state| {
            if let PointerPropagate::Bubble = propagate {
                let mut cx = EventCx {
                    window,
                    app,
                    state,
                    id,
                    focus: &mut focus,
                };

                propagate = widget.on_pointer_event(&mut cx, event);
            }

            current = state.parent;
        });
    }

    app.tree.state_changed(target);
    focus::update_focus(&mut app.tree, root, focus);

    propagate
}
