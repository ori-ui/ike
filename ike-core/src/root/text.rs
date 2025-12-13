use crate::{
    BuildCx, EventCx, Propagate, Root, WidgetId, WindowId,
    context::FocusUpdate,
    event::{TextEvent, TextPasteEvent},
    root::focus,
};

impl Root {
    pub fn text_pasted(&mut self, window: WindowId, contents: String) -> bool {
        let Some(window) = self.get_window(window) else {
            return false;
        };

        let event = TextEvent::Paste(TextPasteEvent { contents });

        let contents = window.contents;

        match focus::find_focused(&self.arena, contents) {
            Some(target) => send_text_event(self, contents, target, &event) == Propagate::Stop,
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

    while let Some(id) = current {
        let widget = root.get_mut(id).unwrap();

        if let Propagate::Bubble = propagate {
            let mut cx = EventCx {
                root:  widget.cx.root,
                arena: widget.cx.arena,
                state: widget.cx.state,
                focus: &mut focus,
            };

            propagate = widget.widget.on_text_event(&mut cx, event);
        }

        current = widget.cx.parent();
    }

    if let Some(mut target) = root.get_mut(target) {
        target.cx.propagate_state();
    }

    focus::update_focus(root, root_widget, focus);

    propagate
}
