use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    Affine, Canvas, DrawCx, EventCx, Fonts, LayoutCx, Point, Pointer, PointerButton,
    PointerButtonEvent, PointerEvent, PointerId, PointerMoveEvent, PointerPropagate, Size, Space,
    Tree, Widget, WidgetId, Window, WindowId,
    context::UpdateCx,
    widget::{Update, WidgetState},
};

pub struct App {
    pub tree: Tree,
    windows: Vec<Window>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            tree: Tree::new(),
            windows: Vec::new(),
        }
    }

    pub fn create_window(&mut self, content: WidgetId) -> &mut Window {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let id = WindowId {
            data: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        };

        self.windows.push(Window {
            id,
            anchor: None,
            content,
            pointers: Vec::new(),
            scale: 1.0,
            size: Size::new(800.0, 600.0),
        });

        self.windows.last_mut().unwrap()
    }

    pub fn remove_window(&mut self, window: WindowId) {
        self.windows.retain(|w| w.id() != window)
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    pub fn windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter()
    }

    pub fn draw(&mut self, window: WindowId, canvas: &mut dyn Canvas) {
        let Some(window) = self.windows.iter().find(|w| w.id == window) else {
            return;
        };

        layout_window(&mut self.tree, canvas.fonts(), window);
        draw_widget(&mut self.tree, window.content, canvas);
    }

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
        remove_hovered(&mut self.tree, window.content);

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

            return match send_pointer_event(self, target, &event) {
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

            match send_pointer_event(self, target, &event) {
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
            let handled = match send_pointer_event(self, target, &event) {
                PointerPropagate::Bubble => false,
                PointerPropagate::Stop => true,

                PointerPropagate::Capture if pressed => {
                    set_active(&mut self.tree, target, true);

                    true
                }

                PointerPropagate::Capture => {
                    panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
                }
            };

            if !pressed && self.tree.widget_state(target.index).is_active {
                set_active(&mut self.tree, target, false);
                update_hovered(&mut self.tree, content, position);
            }

            handled
        } else {
            false
        }
    }

    pub fn window_resized(&mut self, window: WindowId, new_size: Size) {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return;
        };

        window.size = new_size;

        self.tree.request_layout(window.content);
    }

    pub fn window_scaled(&mut self, window: WindowId, scale: f32, new_size: Size) {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return;
        };

        window.scale = scale;
        window.size = new_size;

        self.tree.request_layout(window.content);
    }

    pub fn window_needs_draw(&self, window: WindowId) -> bool {
        match self.windows.iter().find(|w| w.id == window) {
            Some(window) => self.tree.needs_draw(window.content),
            None => false,
        }
    }
}

impl App {
    pub(crate) fn with_entry<T, U>(
        &mut self,
        id: WidgetId<T>,
        f: impl FnOnce(&mut Self, &mut dyn Widget, &mut WidgetState) -> U,
    ) -> U
    where
        T: ?Sized,
    {
        let (mut widget, mut state) = self.tree.take_entry(id).unwrap();
        let output = { f(self, widget.as_mut(), &mut state) };
        self.tree.insert_entry(id, widget, state);
        output
    }
}

fn find_pointer_target(tree: &Tree, root: WidgetId) -> Option<WidgetId> {
    if let Some(target) = find_active(tree, root) {
        Some(target)
    } else {
        find_hovered(tree, root)
    }
}

fn find_hovered(tree: &Tree, root: WidgetId) -> Option<WidgetId> {
    let state = tree.widget_state(root.index);

    if state.is_hovered {
        return Some(root);
    }

    for &child in &state.children {
        let child_state = tree.widget_state(child.index);

        if child_state.has_hovered {
            return find_hovered(tree, child);
        }
    }

    None
}

fn find_active(tree: &Tree, root: WidgetId) -> Option<WidgetId> {
    let state = tree.widget_state(root.index);

    if state.is_active {
        return Some(root);
    }

    for &child in &state.children {
        let child_state = tree.widget_state(child.index);

        if child_state.has_active {
            return find_active(tree, child);
        }
    }

    None
}

fn find_focused(tree: &Tree, root: WidgetId) -> Option<WidgetId> {
    let state = tree.widget_state(root.index);

    if state.is_focused {
        return Some(root);
    }

    for &child in &state.children {
        let child_state = tree.widget_state(child.index);

        if child_state.has_focused {
            return find_focused(tree, child);
        }
    }

    None
}

fn set_active(tree: &mut Tree, id: WidgetId, is_active: bool) {
    tree.with_entry(id, |tree, widget, state| {
        state.is_active = is_active;
        state.has_active = is_active;

        let mut cx = UpdateCx { tree, state };

        widget.update(&mut cx, Update::Active(is_active));
    });

    tree.propagate_state(id);
}

fn set_focused(tree: &mut Tree, id: WidgetId, is_focused: bool) {
    tree.with_entry(id, |tree, widget, state| {
        state.is_focused = is_focused;
        state.has_focused = is_focused;

        let mut cx = UpdateCx { tree, state };

        widget.update(&mut cx, Update::Focused(is_focused));
    });

    tree.propagate_state(id);
}

fn send_pointer_event(app: &mut App, target: WidgetId, event: &PointerEvent) -> PointerPropagate {
    let mut current = Some(target);
    let mut propagate = PointerPropagate::Bubble;

    while let Some(id) = current {
        app.with_entry(id, |app, widget, state| {
            if let PointerPropagate::Bubble = propagate {
                let mut cx = EventCx { app, state };
                propagate = widget.on_pointer_event(&mut cx, event);
            }

            if let Some(parent) = state.parent {
                let parent_state = app.tree.widget_state_mut(parent.index);
                parent_state.merge(state);
            }

            current = state.parent;
        });
    }

    propagate
}

fn update_hovered(tree: &mut Tree, id: WidgetId, pointer: Point) -> Option<WidgetId> {
    let state = tree.widget_state(id.index);
    let local = state.global_transform.inverse() * pointer;

    let is_over = local.x >= 0.0
        && local.y >= 0.0
        && local.x <= state.size.width
        && local.y <= state.size.height;

    if !is_over {
        if state.has_hovered {
            remove_hovered(tree, id);
        }

        return None;
    }

    tree.with_entry(id, |tree, widget, state| {
        let mut hovered = None;

        for child in state.children.clone() {
            let child_hovered = update_hovered(tree, child, pointer);

            let child_state = tree.widget_state(child.index);
            state.merge(child_state);

            if child_hovered.is_some() {
                hovered = child_hovered;
                break;
            }
        }

        if hovered.is_some() && state.is_hovered {
            state.is_hovered = false;
            state.has_hovered = false;

            let mut cx = UpdateCx { tree, state };
            widget.update(&mut cx, Update::Hovered(false));
        }

        if hovered.is_none() && state.accepts_pointer {
            hovered = Some(id);
            if !state.is_hovered {
                state.is_hovered = true;
                state.has_hovered = true;

                let mut cx = UpdateCx { tree, state };
                widget.update(&mut cx, Update::Hovered(true));
            }
        }

        hovered
    })
}

fn remove_hovered(tree: &mut Tree, id: WidgetId) {
    tree.with_entry(id, |tree, widget, state| {
        if !state.has_hovered {
            return;
        }

        for child in state.children.clone() {
            remove_hovered(tree, child);

            let child_state = tree.widget_state(child.index);
            state.merge(child_state);
        }

        if state.is_hovered {
            state.is_hovered = false;
            state.has_hovered = false;

            let mut cx = UpdateCx { tree, state };
            widget.update(&mut cx, Update::Hovered(false));
        }
    });
}

fn draw_widget(tree: &mut Tree, widget: WidgetId, canvas: &mut dyn Canvas) {
    tree.with_entry(widget, |tree, widget, state| {
        canvas.transform(state.transform, &mut |canvas| {
            state.needs_draw = false;

            let mut cx = DrawCx { tree, state };

            widget.draw(&mut cx, canvas);

            for &child in &state.children {
                draw_widget(tree, child, canvas);
            }
        });
    });
}

fn layout_window(tree: &mut Tree, fonts: &mut dyn Fonts, window: &Window) {
    if !tree.needs_layout(window.content) {
        return;
    }

    tree.with_entry(window.content, |tree, widget, state| {
        state.needs_layout = false;

        let mut cx = LayoutCx { fonts, tree, state };

        let space = Space::new(Size::all(0.0), window.size);
        let size = widget.layout(&mut cx, space);
        state.size = size;
    });

    compose_widget(tree, window.content, Affine::IDENTITY);
}

fn compose_widget(tree: &mut Tree, id: WidgetId, transform: Affine) {
    tree.with_entry(id, |tree, _, state| {
        let transform = transform * state.transform;
        state.global_transform = transform;

        for &child in &state.children {
            compose_widget(tree, child, transform);
        }
    });
}
