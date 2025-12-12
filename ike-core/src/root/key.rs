use keyboard_types::{Key, Modifiers, NamedKey};

use crate::{
    BuildCx, EventCx, KeyEvent, Propagate, Root, WidgetId, WindowId, context::FocusUpdate,
    key::KeyPressEvent, root::focus,
};

impl Root {
    pub fn modifiers_changed(&mut self, window: WindowId, modifiers: Modifiers) {
        if let Some(window) = self.get_window_mut(window) {
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
        let Some(window) = self.get_window_mut(window) else {
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

        let handled = match focus::find_focused(&self.tree, contents) {
            Some(target) => send_key_event(self, contents, target, &event) == Propagate::Stop,
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

    while let Some(id) = current {
        let mut widget = root.get_mut(id).unwrap();

        if let Propagate::Bubble = propagate {
            let (tree, root, widget, state) = widget.split();
            let mut cx = EventCx {
                root,
                tree,
                state,
                focus: &mut focus,
            };

            propagate = widget.on_key_event(&mut cx, event);
        }

        current = widget.state().parent;
    }

    if let Some(mut target) = root.get_mut(target) {
        target.propagate_state();
    }

    focus::update_focus(root, root_widget, focus);

    propagate
}
