use std::ops::Range;

use crate::{
    EventCx, ImeEvent, Propagate, WidgetId, WindowId, World,
    context::FocusUpdate,
    event::{TextEvent, TextPasteEvent},
    world::focus,
};

impl World {
    pub fn text_pasted(&mut self, window: WindowId, contents: String) -> bool {
        self.handle_updates();

        if let Some(window) = self.window(window)
            && let Some(focused) = window.focused
        {
            let event = TextEvent::Paste(TextPasteEvent { contents });
            send_text_event(self, window.id, focused, &event) == Propagate::Handled
        } else {
            false
        }
    }

    pub fn ime_commit_text(&mut self, window: WindowId, text: String) -> bool {
        self.handle_updates();

        if let Some(window) = self.window(window)
            && let Some(focused) = window.focused
        {
            let event = TextEvent::Ime(ImeEvent::Commit(text));
            send_text_event(self, window.id, focused, &event) == Propagate::Handled
        } else {
            false
        }
    }

    pub fn ime_select(&mut self, window: WindowId, selection: Range<usize>) -> bool {
        self.handle_updates();

        if let Some(window) = self.window(window)
            && let Some(focused) = window.focused
        {
            let event = TextEvent::Ime(ImeEvent::Select(selection));
            send_text_event(self, window.id, focused, &event) == Propagate::Handled
        } else {
            false
        }
    }
}

pub(super) fn send_text_event(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    event: &TextEvent,
) -> Propagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = Propagate::Bubble;

    let _span = tracing::info_span!("text_event");

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

        propagate = widget.widget.on_text_event(&mut cx, event);
        current = widget.cx.parent();
    }

    world.propagate_state(target);

    focus::update_focus(world, window, focus);

    propagate
}
