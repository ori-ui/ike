use keyboard_types::{Key, Modifiers, NamedKey};

use crate::{
    EventCx, KeyEvent, KeyPressEvent, Propagate, WidgetId, WindowId, World, context::FocusUpdate,
    world::focus,
};

impl World {
    pub fn modifiers_changed(&mut self, window: WindowId, modifiers: Modifiers) {
        if let Some(window) = self.window_mut(window) {
            window.modifiers = modifiers;
        }
    }

    pub fn key_pressed(
        &mut self,
        window: WindowId,
        key: Key,
        repeat: bool,
        text: Option<&str>,
        pressed: bool,
    ) -> bool {
        self.handle_updates();
        let window_id = window;

        let Some(window) = self.window(window) else {
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

        let modifiers = window.modifiers;

        let handled = match window.focused {
            Some(target) => send_key_event(self, window_id, target, &event) == Propagate::Handled,
            None => false,
        };

        if key == Key::Named(NamedKey::Tab) && pressed && !handled {
            focus::focus_next(self, window_id, !modifiers.shift());
        }

        handled
    }
}

fn send_key_event(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    event: &KeyEvent,
) -> Propagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = Propagate::Bubble;

    let _span = tracing::info_span!("key_event");

    while let Some(id) = current
        && let Some(widget) = world.widget_mut(id)
        && let Propagate::Bubble = propagate
    {
        let mut cx = EventCx {
            world: widget.cx.world,
            arena: widget.cx.arena,
            state: widget.cx.state,
            focus: &mut focus,
        };

        propagate = widget.widget.on_key_event(&mut cx, event);

        current = widget.cx.parent();
    }

    world.propagate_state(target);
    focus::update_focus(world, window, focus);

    propagate
}
