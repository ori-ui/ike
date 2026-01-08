use crate::{
    Key, KeyEvent, KeyPressEvent, Modifiers, NamedKey, Propagate, WidgetId, WindowId, World, passes,
};

pub(crate) fn modifiers_changed(world: &mut World, window: WindowId, modifiers: Modifiers) -> bool {
    if let Some(window) = world.window_mut(window) {
        window.modifiers = modifiers;
    }

    true
}

pub(crate) fn pressed(
    world: &mut World,
    window: WindowId,
    key: Key,
    repeat: bool,
    text: Option<&str>,
    pressed: bool,
) -> bool {
    let window_id = window;

    let Some(window) = world.window(window_id) else {
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
        Some(target) => send_event(world, window_id, target, &event) == Propagate::Handled,
        None => false,
    };

    if key == Key::Named(NamedKey::Tab) && pressed && !handled {
        passes::focus::next(world, window_id, !modifiers.shift());
    }

    handled
}

pub(crate) fn send_event(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    event: &KeyEvent,
) -> Propagate {
    passes::event::send_event(
        world,
        window,
        target,
        Propagate::Bubble,
        |widget, cx| widget.on_key_event(cx, event),
    )
}
