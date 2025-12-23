use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::{Duration, Instant},
};

use crate::{Offset, Point, WidgetId};

#[derive(Clone, Debug, PartialEq)]
pub enum TouchEvent {
    Down(TouchPressEvent),
    Up(TouchPressEvent),
    Move(TouchMoveEvent),
    Gesture(Gesture),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TouchPressEvent {
    pub touch:    TouchId,
    pub position: Point,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TouchMoveEvent {
    pub touch:    TouchId,
    pub position: Point,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Gesture {
    Tap(TapGesture),
    LongTap(TapGesture),
    DoubleTap(TapGesture),
    Pan(PanGesture),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TapGesture {
    pub touch:    TouchId,
    pub position: Point,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PanGesture {
    pub touch:    TouchId,
    pub start:    Point,
    pub position: Point,
    pub delta:    Offset,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TouchPropagate {
    Bubble,
    Handled,
    Capture,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TouchSettings {
    pub tap_slop:        f32,
    pub tap_time:        Duration,
    pub double_tap_slop: f32,
    pub double_tap_time: Duration,
    pub long_tap_time:   Duration,
    pub pan_distance:    f32,
}

impl Default for TouchSettings {
    fn default() -> Self {
        Self {
            tap_slop:        10.0,
            tap_time:        Duration::from_millis(200),
            double_tap_slop: 20.0,
            double_tap_time: Duration::from_millis(300),
            long_tap_time:   Duration::from_millis(500),
            pan_distance:    10.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Touch {
    pub(crate) id:               TouchId,
    pub(crate) current_position: Point,
    pub(crate) start_position:   Point,
    pub(crate) start_time:       Instant,
    pub(crate) state:            TouchState,
    pub(crate) capturer:         Option<WidgetId>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum TouchState {
    None,
    Tapped(Point, Instant),
    Panning,
}

impl Touch {
    pub fn distance(&self) -> f32 {
        self.start_position.distance(self.current_position)
    }

    pub fn duration(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TouchId {
    data: u64,
}

impl TouchId {
    pub fn from_hash(hash: impl Hash) -> Self {
        let mut hasher = DefaultHasher::new();
        hash.hash(&mut hasher);

        Self {
            data: hasher.finish(),
        }
    }

    pub fn from_u64(data: u64) -> Self {
        Self { data }
    }
}
