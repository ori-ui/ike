use keyboard_types::{Key, Modifiers, NamedKey};

use crate::{
    App, EventCx, KeyEvent, Propagate, WidgetId, WindowId, app::focus, context::FocusUpdate,
    key::KeyPressEvent,
};

impl App {
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

        let contents = window.contents;
        let modifiers = window.modifiers;
        let window = window.id;

        let handled = match focus::find_focused(&self.tree, contents) {
            Some(target) => {
                send_key_event(self, window, contents, target, &event) == Propagate::Stop
            }
            None => false,
        };

        if key == Key::Named(NamedKey::Tab) && pressed && !handled {
            focus::focus_next(&mut self.tree, contents, !modifiers.shift());
        }

        handled
    }
}

fn send_key_event(
    app: &mut App,
    window: WindowId,
    root: WidgetId,
    target: WidgetId,
    event: &KeyEvent,
) -> Propagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = Propagate::Bubble;

    let _span = tracing::info_span!("key_event");

    while let Some(id) = current {
        let mut widget = app.tree.get_mut(id).unwrap();

        if let Propagate::Bubble = propagate {
            let (tree, widget, state) = widget.split();
            let mut cx = EventCx {
                window,
                windows: &mut app.windows,
                tree,
                state,
                id,
                focus: &mut focus,
            };

            propagate = widget.on_key_event(&mut cx, event);
        }

        current = widget.state().parent;
    }

    if let Some(mut target) = app.tree.get_mut(target) {
        target.propagate_state();
    }

    focus::update_focus(&mut app.tree, root, focus);

    propagate
}
