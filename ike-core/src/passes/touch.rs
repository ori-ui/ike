use std::time::Instant;

use crate::{
    Gesture, PanGesture, Point, TapGesture, Touch, TouchEvent, TouchId, TouchMoveEvent,
    TouchPressEvent, TouchPropagate, WidgetId, Window, WindowId, World, event::TouchState, passes,
};

pub(crate) fn down(world: &mut World, window: WindowId, touch: TouchId, position: Point) -> bool {
    let window_id = window;

    let Some(window) = world.state.window_mut(window) else {
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

    if let Some(window) = world.window(window_id)
        && let Some(touch) = window.touch(touch)
        && let Some(target) = find_touch_target(world, window, touch, position)
    {
        let event = TouchEvent::Down(TouchPressEvent {
            touch: touch.id,
            position,
        });

        match send_event(world, window_id, target, &event) {
            TouchPropagate::Bubble => false,
            TouchPropagate::Handled => true,

            TouchPropagate::Capture => {
                if let Some(mut widget) = world.widget_mut(target) {
                    widget.set_active(true);
                }

                true
            }
        }
    } else {
        false
    }
}

pub(crate) fn up(world: &mut World, window: WindowId, touch: TouchId, position: Point) -> bool {
    let window_id = window;
    let touch_id = touch;

    let tap_slop = world.state.touch_settings.tap_slop;
    let tap_time = world.state.touch_settings.tap_time;
    let double_tap_slop = world.state.touch_settings.double_tap_slop;
    let double_tap_time = world.state.touch_settings.double_tap_time;

    let Some(window) = world.state.window_mut(window_id) else {
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
        handled |= send_event_at(
            world, window_id, touch_id, position, &event,
        );
    }

    if !matches!(touch_state, TouchState::Panning)
        && !handled
        && let Some(window) = world.window(window_id)
        && let Some(focused) = window.focused
        && world.widget(focused).is_some_and(|widget| {
            let local = widget.cx.global_transform().inverse() * position;
            !widget.cx.rect().contains(local)
        })
    {
        passes::focus::transfer(world, window_id, None);
    }

    if let Some(window) = world.window(window_id)
        && let Some(touch) = window.touch(touch_id)
        && let Some(target) = find_touch_target(world, window, touch, position)
        && let Some(mut widget) = world.widget_mut(target)
    {
        widget.set_active(false);
    }

    handled
}

pub(crate) fn moved(world: &mut World, window: WindowId, touch: TouchId, position: Point) -> bool {
    let window_id = window;
    let touch_id = touch;

    let pan_distance = world.state.touch_settings.pan_distance;

    let Some(window) = world.state.window_mut(window_id) else {
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

        if let Some(window) = world.window(window_id)
            && let Some(touch) = window.touch(touch_id)
            && let Some(target) = find_touch_target(world, window, touch, position)
        {
            let pan_event = TouchEvent::Gesture(Gesture::Pan(PanGesture {
                touch: touch_id,
                start: touch.start_position,
                position,
                delta,
            }));

            handled |= match send_event(world, window_id, target, &pan_event) {
                TouchPropagate::Bubble => false,
                TouchPropagate::Handled => true,

                TouchPropagate::Capture => {
                    if let Some(mut widget) = world.widget_mut(target) {
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

    send_event_at(
        world, window_id, touch_id, position, &event,
    ) || handled
}

fn send_event_at(
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
        match send_event(world, window.id, target, event) {
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
        None => passes::query::find_widget_at(world, window, position),
    }
}

pub(crate) fn send_event(
    world: &mut World,
    window: WindowId,
    target: WidgetId,
    event: &TouchEvent,
) -> TouchPropagate {
    passes::event::send_event(
        world,
        window,
        target,
        TouchPropagate::Bubble,
        |widget, cx| widget.on_touch_event(cx, event),
    )
}
