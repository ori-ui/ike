use keyboard_types::{Key, Modifiers, NamedKey};

use crate::{
    BuildCx, EventCx, KeyEvent, KeyPressEvent, Propagate, Root, WidgetId, WindowId,
    context::FocusUpdate,
    root::{focus, query},
};

impl Root {
    pub fn modifiers_changed(&mut self, window: WindowId, modifiers: Modifiers) {
        if let Some(window) = self.get_window_mut(window) {
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

        let Some(window) = self.get_window(window) else {
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

        let contents = window.contents;
        let modifiers = window.modifiers;

        let handled = match query::find_focused(&self.arena, contents) {
            Some(target) => send_key_event(self, contents, target, &event) == Propagate::Handled,
            None => false,
        };

        if key == Key::Named(NamedKey::Tab) && pressed && !handled {
            focus::focus_next(self, contents, !modifiers.shift());
        }

        handled
    }
}

fn send_key_event(
    root: &mut Root,
    root_widget: WidgetId,
    target: WidgetId,
    event: &KeyEvent,
) -> Propagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = Propagate::Bubble;

    let _span = tracing::info_span!("key_event");

    while let Some(id) = current
        && let Some(widget) = root.get_widget_mut(id)
        && let Propagate::Bubble = propagate
    {
        let mut cx = EventCx {
            root:  widget.cx.root,
            arena: widget.cx.arena,
            state: widget.cx.state,
            focus: &mut focus,
        };

        propagate = widget.widget.on_key_event(&mut cx, event);

        current = widget.cx.parent();
    }

    root.propagate_state(target);
    focus::update_focus(root, root_widget, focus);

    propagate
}
