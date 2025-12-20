use std::ops::Range;

use crate::{
    BuildCx, EventCx, ImeEvent, Propagate, Root, WidgetId, WindowId,
    context::FocusUpdate,
    event::{TextEvent, TextPasteEvent},
    root::{focus, query},
};

impl Root {
    pub fn text_pasted(&mut self, window: WindowId, contents: String) -> bool {
        self.handle_updates();

        let Some(window) = self.get_window(window) else {
            return false;
        };

        let event = TextEvent::Paste(TextPasteEvent { contents });

        let contents = window.contents;

        match query::find_focused(&self.arena, contents) {
            Some(target) => send_text_event(self, contents, target, &event) == Propagate::Handled,
            None => false,
        }
    }

    pub fn ime_commit_text(&mut self, window: WindowId, text: String) -> bool {
        self.handle_updates();

        let Some(window) = self.get_window(window) else {
            return false;
        };

        let event = TextEvent::Ime(ImeEvent::Commit(text));

        let contents = window.contents;

        match query::find_focused(&self.arena, contents) {
            Some(target) => send_text_event(self, contents, target, &event) == Propagate::Handled,
            None => false,
        }
    }

    pub fn ime_select(&mut self, window: WindowId, selection: Range<usize>) -> bool {
        self.handle_updates();

        let Some(window) = self.get_window(window) else {
            return false;
        };

        let event = TextEvent::Ime(ImeEvent::Select(selection));

        let contents = window.contents;

        match query::find_focused(&self.arena, contents) {
            Some(target) => send_text_event(self, contents, target, &event) == Propagate::Handled,
            None => false,
        }
    }
}

fn send_text_event(
    root: &mut Root,
    root_widget: WidgetId,
    target: WidgetId,
    event: &TextEvent,
) -> Propagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = Propagate::Bubble;

    let _span = tracing::info_span!("text_event");

    while let Some(id) = current
        && let Some(widget) = root.get_mut(id)
        && let Propagate::Bubble = propagate
    {
        let mut cx = EventCx {
            root:  widget.cx.root,
            arena: widget.cx.arena,
            state: widget.cx.state,
            focus: &mut focus,
        };

        propagate = widget.widget.on_text_event(&mut cx, event);
        current = widget.cx.parent();
    }

    root.propagate_state(target);

    focus::update_focus(root, root_widget, focus);

    propagate
}
