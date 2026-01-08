use std::ops::Range;

use crate::{ImeEvent, Propagate, TextEvent, TextPasteEvent, WidgetId, WindowId, World, passes};

pub(crate) fn pasted(world: &mut World, window: WindowId, contents: String) -> bool {
    if let Some(window) = world.window(window)
        && let Some(focused) = window.focused
    {
        let event = TextEvent::Paste(TextPasteEvent { contents });
        send_event(world, window.id, focused, &event) == Propagate::Handled
    } else {
        false
    }
}

pub(crate) fn ime_commit(world: &mut World, window: WindowId, text: String) -> bool {
    if let Some(window) = world.window(window)
        && let Some(focused) = window.focused
    {
        let event = TextEvent::Ime(ImeEvent::Commit(text));
        send_event(world, window.id, focused, &event) == Propagate::Handled
    } else {
        false
    }
}

pub(crate) fn ime_select(world: &mut World, window: WindowId, selection: Range<usize>) -> bool {
    if let Some(window) = world.window(window)
        && let Some(focused) = window.focused
    {
        let event = TextEvent::Ime(ImeEvent::Select(selection));
        send_event(world, window.id, focused, &event) == Propagate::Handled
    } else {
        false
    }
}

pub(crate) fn send_event(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    event: &TextEvent,
) -> Propagate {
    passes::event::send_event(
        world,
        window,
        target,
        Propagate::Bubble,
        |widget, cx| widget.on_text_event(cx, event),
    )
}
