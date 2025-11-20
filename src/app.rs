use std::{
    mem,
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use crate::{
    Affine, Canvas, Color, DrawCx, EventCx, Fonts, Key, KeyEvent, LayoutCx, Modifiers, NamedKey,
    Point, Pointer, PointerButton, PointerButtonEvent, PointerEvent, PointerId, PointerMoveEvent,
    PointerPropagate, Propagate, Size, Space, Tree, Widget, WidgetId, Window, WindowId,
    context::UpdateCx, key::KeyPressEvent, widget::WidgetState,
};

pub struct App {
    pub tree: Tree,
    windows:  Vec<Window>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            tree:    Tree::new(),
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
            modifiers: Modifiers::empty(),
            scale: 1.0,
            size: Size::new(800.0, 600.0),
            color: Color::WHITE,
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
        draw_over_widget(&mut self.tree, window.content, canvas);
    }

    pub fn animate(&mut self, window: WindowId, delta_time: Duration) {
        let Some(window) = self.windows.iter().find(|w| w.id == window) else {
            return;
        };

        animate_widget(&mut self.tree, window.content, delta_time);
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
                    self.tree.set_active(target, true);

                    true
                }

                PointerPropagate::Capture => {
                    panic!("pointer capture is only valid in response to PointerEvent::Down(..)");
                }
            };

            if !pressed && self.tree.widget_state(target.index).is_active {
                self.tree.set_active(target, false);
                update_hovered(&mut self.tree, content, position);
            }

            handled
        } else {
            false
        }
    }

    pub fn modifiers_changed(&mut self, window: WindowId, modifiers: Modifiers) {
        if let Some(window) = self.windows.iter_mut().find(|w| w.id == window) {
            window.modifiers = modifiers;
        }
    }

    pub fn key_press(
        &mut self,
        window: WindowId,
        key: Key,
        repeat: bool,
        text: Option<&str>,
        pressed: bool,
    ) -> bool {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return false;
        };

        let event = KeyPressEvent {
            key: key.clone(),
            modifiers: window.modifiers,
            text: text.map(Into::into),
            repeat,
        };

        let event = match pressed {
            true => KeyEvent::Down(event),
            false => KeyEvent::Up(event),
        };

        let content = window.content;
        let modifiers = window.modifiers;

        let handled = match find_focused(&self.tree, window.content) {
            Some(target) => send_key_event(self, target, &event) == Propagate::Stop,
            None => false,
        };

        if key == Key::Named(NamedKey::Tab) && pressed && !handled {
            focus_next(&mut self.tree, content, !modifiers.shift());
        }

        handled
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

    pub fn window_needs_animate(&self, window: WindowId) -> bool {
        match self.windows.iter().find(|w| w.id == window) {
            Some(window) => self.tree.needs_animate(window.content),
            None => false,
        }
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

fn find_hovered(tree: &Tree, id: WidgetId) -> Option<WidgetId> {
    let state = tree.widget_state(id.index);

    if state.is_hovered {
        return Some(id);
    }

    for &child in &state.children {
        let child_state = tree.widget_state(child.index);

        if child_state.has_hovered {
            return find_hovered(tree, child);
        }
    }

    None
}

fn find_active(tree: &Tree, id: WidgetId) -> Option<WidgetId> {
    let state = tree.widget_state(id.index);

    if state.is_active {
        return Some(id);
    }

    for &child in &state.children {
        let child_state = tree.widget_state(child.index);

        if child_state.has_active {
            return find_active(tree, child);
        }
    }

    None
}

fn find_focused(tree: &Tree, id: WidgetId) -> Option<WidgetId> {
    let state = tree.widget_state(id.index);

    if state.is_focused {
        return Some(id);
    }

    for &child in &state.children {
        let child_state = tree.widget_state(child.index);

        if child_state.has_focused {
            return find_focused(tree, child);
        }
    }

    None
}

fn find_first_focusable(tree: &Tree, id: WidgetId, forward: bool) -> Option<WidgetId> {
    let state = tree.widget_state(id.index);

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
    let state = tree.widget_state(id.index);

    if !state.has_focused {
        return find_first_focusable(tree, id, forward);
    }

    if forward {
        let mut children = state.children.iter().copied();

        for child in children.by_ref() {
            let child_state = tree.widget_state(child.index);

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
            let child_state = tree.widget_state(child.index);

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

fn focus_next(tree: &mut Tree, id: WidgetId, forward: bool) -> Option<WidgetId> {
    let current = find_focused(tree, id);
    let focused = find_next_focusable(tree, id, forward);

    if let Some(current) = current {
        tree.set_focused(current, false);
    }

    if let Some(focused) = focused {
        tree.set_focused(focused, true);
    }

    focused
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

            current = state.parent;
        });
    }

    app.tree.state_changed(target);

    propagate
}

fn send_key_event(app: &mut App, target: WidgetId, event: &KeyEvent) -> Propagate {
    let mut current = Some(target);
    let mut propagate = Propagate::Bubble;

    while let Some(id) = current {
        app.with_entry(id, |app, widget, state| {
            if let Propagate::Bubble = propagate {
                let mut cx = EventCx { app, state };
                propagate = widget.on_key_event(&mut cx, event);
            }

            current = state.parent;
        });
    }

    app.tree.state_changed(target);

    propagate
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
    let state = tree.widget_state(id.index);
    let local = state.global_transform.inverse() * point;

    let is_over = local.x >= 0.0
        && local.y >= 0.0
        && local.x <= state.size.width
        && local.y <= state.size.height;

    if !is_over {
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

fn draw_widget(tree: &mut Tree, widget: WidgetId, canvas: &mut dyn Canvas) {
    tree.with_entry(widget, |tree, widget, state| {
        canvas.transform(state.transform, &mut |canvas| {
            let mut cx = DrawCx { tree, state };
            widget.draw(&mut cx, canvas);

            for &child in &state.children {
                draw_widget(tree, child, canvas);
            }
        });
    });
}

fn draw_over_widget(tree: &mut Tree, widget: WidgetId, canvas: &mut dyn Canvas) {
    tree.with_entry(widget, |tree, widget, state| {
        canvas.transform(state.transform, &mut |canvas| {
            state.needs_draw = false;

            let mut cx = DrawCx { tree, state };
            widget.draw_over(&mut cx, canvas);

            for &child in &state.children {
                draw_over_widget(tree, child, canvas);
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
    tree.with_entry(id, |tree, widget, state| {
        let transform = transform * state.transform;
        state.global_transform = transform;

        let mut cx = UpdateCx { tree, state };
        widget.compose(&mut cx);

        for &child in &state.children {
            compose_widget(tree, child, transform);
        }
    });
}

fn animate_widget(tree: &mut Tree, id: WidgetId, dt: Duration) {
    tree.with_entry(id, |tree, widget, state| {
        state.needs_animate = false;

        let mut cx = UpdateCx { tree, state };
        widget.animate(&mut cx, dt);

        state.reset();

        let children = mem::take(&mut state.children);
        for &child in &children {
            animate_widget(tree, child, dt);

            let child_state = tree.widget_state(child.index);
            state.merge(child_state);
        }

        state.children = children;
    });
}
