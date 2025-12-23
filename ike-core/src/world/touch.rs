use std::time::Instant;

use crate::{
    EventCx, Gesture, PanGesture, Point, Rect, TapGesture, Touch, TouchEvent, TouchId,
    TouchMoveEvent, TouchPressEvent, TouchPropagate, WidgetId, Window, WindowId, World,
    context::FocusUpdate,
    event::TouchState,
    world::{focus, query},
};

impl World {
    pub fn touch_down(&mut self, window: WindowId, touch: TouchId, position: Point) -> bool {
        self.handle_updates();
        let window_id = window;

        let Some(window) = self.state.window_mut(window) else {
            return false;
        };

        match window.touches.iter_mut().find(|t| t.id == touch) {
            Some(touch) => {
                touch.current_position = position;
                touch.start_position = position;
                touch.start_time = Instant::now();
                touch.capturer = None;
            }

            None => {
                window.touches.push(Touch {
                    id:               touch,
                    current_position: position,
                    start_position:   position,
                    start_time:       Instant::now(),
                    state:            TouchState::None,
                    capturer:         None,
                });
            }
        }

        if let Some(window) = self.window(window_id)
            && let Some(touch) = window.touch(touch)
            && let Some(target) = find_touch_target(self, window, touch, position)
        {
            let event = TouchEvent::Down(TouchPressEvent {
                touch: touch.id,
                position,
            });

            match send_touch_event(self, window_id, target, &event) {
                TouchPropagate::Bubble => false,
                TouchPropagate::Handled => true,

                TouchPropagate::Capture => {
                    if let Some(mut widget) = self.widget_mut(target) {
                        widget.set_active(true);
                    }

                    true
                }
            }
        } else {
            false
        }
    }

    pub fn touch_up(&mut self, window: WindowId, touch: TouchId, position: Point) -> bool {
        self.handle_updates();
        let window_id = window;
        let touch_id = touch;

        let tap_slop = self.state.touch_settings.tap_slop;
        let tap_time = self.state.touch_settings.tap_time;
        let double_tap_slop = self.state.touch_settings.double_tap_slop;
        let double_tap_time = self.state.touch_settings.double_tap_time;

        let Some(window) = self.state.window_mut(window_id) else {
            return false;
        };

        let Some(touch) = window.touches.iter_mut().find(|t| t.id == touch_id) else {
            return false;
        };

        let mut events = Vec::new();

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

            events.push(tap_event);
            events.push(double_tap_event);
        } else if touch.distance() < tap_slop && touch.duration() < tap_time {
            tracing::trace!(?touch_id, ?position, "touch tap");

            touch.state = TouchState::Tapped(position, Instant::now());

            let tap_event = TouchEvent::Gesture(Gesture::Tap(TapGesture {
                touch: touch_id,
                position,
            }));

            events.push(tap_event);
        } else {
            touch.state = TouchState::None;
        }

        let touch_state = touch.state.clone();

        let up_event = TouchEvent::Up(TouchPressEvent {
            touch: touch_id,
            position,
        });

        events.push(up_event);

        let mut handled = false;

        for event in events {
            handled |= send_touch_event_at(
                self, window_id, touch_id, position, &event,
            );
        }

        if !matches!(touch_state, TouchState::Panning)
            && !handled
            && let Some(window) = self.window(window_id)
            && let Some(focused) = window.focused
            && self.widget(focused).is_some_and(|widget| {
                let local = widget.cx.global_transform().inverse() * position;
                !Rect::min_size(Point::ORIGIN, widget.cx.size()).contains(local)
            })
        {
            focus::set_focused(self, window_id, None);
        }

        if let Some(window) = self.window(window_id)
            && let Some(touch) = window.touch(touch_id)
            && let Some(target) = find_touch_target(self, window, touch, position)
            && let Some(mut widget) = self.widget_mut(target)
        {
            widget.set_active(false);
        }

        handled
    }

    pub fn touch_moved(&mut self, window: WindowId, touch: TouchId, position: Point) -> bool {
        self.handle_updates();
        let window_id = window;
        let touch_id = touch;

        let pan_distance = self.state.touch_settings.pan_distance;

        let Some(window) = self.state.window_mut(window_id) else {
            return false;
        };

        let Some(touch) = window.touch_mut(touch_id) else {
            return false;
        };

        let delta = position - touch.current_position;
        touch.current_position = position;

        let mut handled = false;

        if touch.distance() > pan_distance || matches!(touch.state, TouchState::Panning) {
            tracing::trace!(?touch_id, ?position, "touch pan");

            touch.state = TouchState::Panning;

            if let Some(window) = self.window(window_id)
                && let Some(touch) = window.touch(touch_id)
                && let Some(target) = find_touch_target(self, window, touch, position)
            {
                let pan_event = TouchEvent::Gesture(Gesture::Pan(PanGesture {
                    touch: touch_id,
                    start: touch.start_position,
                    position,
                    delta,
                }));

                handled |= match send_touch_event(self, window_id, target, &pan_event) {
                    TouchPropagate::Bubble => false,
                    TouchPropagate::Handled => true,

                    TouchPropagate::Capture => {
                        if let Some(mut widget) = self.widget_mut(target) {
                            widget.set_active(true);
                        }

                        true
                    }
                }
            }
        }

        let event = TouchEvent::Move(TouchMoveEvent {
            touch: touch_id,
            position,
        });

        send_touch_event_at(
            self, window_id, touch_id, position, &event,
        ) || handled
    }
}

fn send_touch_event_at(
    world: &mut World,
    window: WindowId,
    touch: TouchId,
    position: Point,
    event: &TouchEvent,
) -> bool {
    if let Some(window) = world.window(window)
        && let Some(touch) = window.touch(touch)
        && let Some(target) = find_touch_target(world, window, touch, position)
    {
        match send_touch_event(world, window.id, target, event) {
            TouchPropagate::Bubble => false,
            TouchPropagate::Handled => true,
            TouchPropagate::Capture => {
                tracing::error!(
                    "touch capture is only valid in response to `TouchEvent::Down` or `Gesture::Pan`"
                );

                true
            }
        }
    } else {
        false
    }
}

fn find_touch_target(
    world: &World,
    window: &Window,
    touch: &Touch,
    position: Point,
) -> Option<WidgetId> {
    match touch.capturer {
        Some(target) => Some(target),
        None => query::find_widget_at(world, window, position),
    }
}

fn send_touch_event(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    event: &TouchEvent,
) -> TouchPropagate {
    let mut focus = FocusUpdate::None;
    let mut current = Some(target);
    let mut propagate = TouchPropagate::Bubble;

    let _span = tracing::info_span!("touch_event");

    while let Some(id) = current
        && let Some(widget) = world.widget_mut(id)
        && let TouchPropagate::Bubble = propagate
    {
        let mut cx = EventCx {
            world: widget.cx.world,
            arena: widget.cx.arena,
            state: widget.cx.state,
            focus: &mut focus,
        };

        propagate = widget.widget.on_touch_event(&mut cx, event);
        current = widget.cx.parent();
    }

    world.propagate_state(target);
    focus::update_focus(world, window, focus);

    propagate
}
