use std::time::Instant;

use crate::{
    Arena, BuildCx, EventCx, Gesture, PanGesture, Point, Rect, Root, TapGesture, Touch, TouchEvent,
    TouchId, TouchMoveEvent, TouchPressEvent, TouchPropagate, WidgetId, WindowId,
    context::FocusUpdate,
    event::TouchState,
    root::{focus, query},
};

impl Root {
    pub fn touch_down(&mut self, window: WindowId, touch_id: TouchId, position: Point) -> bool {
        self.handle_updates();

        let Some(window) = self.state.get_window_mut(window) else {
            return false;
        };

        match window.touches.iter_mut().find(|t| t.id == touch_id) {
            Some(touch) => {
                touch.current_position = position;
                touch.start_position = position;
                touch.start_time = Instant::now();
            }

            None => {
                window.touches.push(Touch {
                    id:               touch_id,
                    current_position: position,
                    start_position:   position,
                    start_time:       Instant::now(),
                    state:            TouchState::None,
                });
            }
        };

        let window_contents = window.contents;

        if let Some(focused) = query::find_focused(&self.arena, window_contents)
            && self.get_widget(focused).is_some_and(|widget| {
                let local = widget.cx.global_transform().inverse() * position;
                !Rect::min_size(Point::ORIGIN, widget.cx.size()).contains(local)
            })
        {
            focus::set_focused(self, window_contents, None);
        }

        if let Some(target) = find_touch_target_at(&self.arena, window_contents, position) {
            let event = TouchEvent::Down(TouchPressEvent {
                touch: touch_id,
                position,
            });

            match send_touch_event(self, window_contents, target, &event) {
                TouchPropagate::Bubble => false,
                TouchPropagate::Handled => true,

                TouchPropagate::Capture => {
                    if let Some(mut widget) = self.get_widget_mut(target) {
                        widget.set_active(true);
                    }

                    true
                }
            }
        } else {
            false
        }
    }

    pub fn touch_up(&mut self, window: WindowId, touch_id: TouchId, position: Point) -> bool {
        self.handle_updates();

        let tap_slop = self.state.touch_settings.tap_slop;
        let tap_time = self.state.touch_settings.tap_time;
        let double_tap_slop = self.state.touch_settings.double_tap_slop;
        let double_tap_time = self.state.touch_settings.double_tap_time;

        let Some(window) = self.state.get_window_mut(window) else {
            return false;
        };

        let Some(touch) = window.touches.iter_mut().find(|t| t.id == touch_id) else {
            return false;
        };

        let window_contents = window.contents;

        let mut handled = false;

        if let TouchState::Tapped(tap_position, tap_time) = touch.state
            && tap_position.distance(position) < double_tap_slop
            && tap_time.elapsed() < double_tap_time
        {
            tracing::trace!(?touch_id, ?position, "touch double tap");

            touch.state = TouchState::None;

            let tap_event = TouchEvent::Gesture(Gesture::Tap(TapGesture {
                touch: touch_id,
                position,
            }));

            let double_tap_event = TouchEvent::Gesture(Gesture::DoubleTap(TapGesture {
                touch: touch_id,
                position,
            }));

            handled |= send_touch_event_at(
                self,
                window_contents,
                position,
                &tap_event,
            );

            handled |= send_touch_event_at(
                self,
                window_contents,
                position,
                &double_tap_event,
            );
        } else if touch.distance() < tap_slop && touch.duration() < tap_time {
            tracing::trace!(?touch_id, ?position, "touch tap");

            touch.state = TouchState::Tapped(position, Instant::now());

            let tap_event = TouchEvent::Gesture(Gesture::Tap(TapGesture {
                touch: touch_id,
                position,
            }));

            handled |= send_touch_event_at(
                self,
                window_contents,
                position,
                &tap_event,
            );
        }

        let event = TouchEvent::Up(TouchPressEvent {
            touch: touch_id,
            position,
        });

        if let Some(target) = find_touch_target_at(&self.arena, window_contents, position) {
            if let Some(mut widget) = self.get_widget_mut(target) {
                widget.set_active(false);
            }

            send_touch_event_at(self, window_contents, position, &event) || handled
        } else {
            handled
        }
    }

    pub fn touch_moved(&mut self, window: WindowId, touch_id: TouchId, position: Point) -> bool {
        self.handle_updates();

        let pan_distance = self.state.touch_settings.pan_distance;

        let Some(window) = self.state.get_window_mut(window) else {
            return false;
        };

        let Some(touch) = window.touches.iter_mut().find(|t| t.id == touch_id) else {
            return false;
        };

        let delta = position - touch.current_position;

        touch.current_position = position;
        let window_contents = window.contents;

        let mut handled = false;

        if touch.distance() > pan_distance || matches!(touch.state, TouchState::Panning) {
            tracing::trace!(?touch_id, ?position, "touch pan");

            touch.state = TouchState::Panning;

            let start_position = touch.start_position;
            let pan_evnet = TouchEvent::Gesture(Gesture::Pan(PanGesture {
                touch: touch_id,
                start: start_position,
                position,
                delta,
            }));

            // NOTE: touch event is sent to the widget at the start of the pan, not the current
            // position
            handled |= send_touch_event_at(
                self,
                window_contents,
                start_position,
                &pan_evnet,
            );
        }

        let event = TouchEvent::Move(TouchMoveEvent {
            touch: touch_id,
            position,
        });

        send_touch_event_at(self, window_contents, position, &event) || handled
    }
}

fn send_touch_event_at(
    root: &mut Root,
    root_widget: WidgetId,
    position: Point,
    event: &TouchEvent,
) -> bool {
    if let Some(target) = find_touch_target_at(&root.arena, root_widget, position) {
        send_touch_event(root, root_widget, target, event) == TouchPropagate::Handled
    } else {
        false
    }
}

fn find_touch_target_at(arena: &Arena, root_widget: WidgetId, position: Point) -> Option<WidgetId> {
    if let Some(target) = query::find_active(arena, root_widget) {
        Some(target)
    } else {
        query::find_widget_at(arena, root_widget, position)
    }
}

fn send_touch_event(
    root: &mut Root,
    root_widget: WidgetId,
    target: WidgetId,
    event: &TouchEvent,
) -> TouchPropagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = TouchPropagate::Bubble;

    let _span = tracing::info_span!("touch_event");

    while let Some(id) = current
        && let Some(widget) = root.get_widget_mut(id)
        && let TouchPropagate::Bubble = propagate
    {
        let mut cx = EventCx {
            root:  widget.cx.root,
            arena: widget.cx.arena,
            state: widget.cx.state,
            focus: &mut focus,
        };

        propagate = widget.widget.on_touch_event(&mut cx, event);
        current = widget.cx.parent();
    }

    root.propagate_state(target);
    focus::update_focus(root, root_widget, focus);

    propagate
}
