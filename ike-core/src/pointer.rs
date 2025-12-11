use std::hash::{DefaultHasher, Hash, Hasher};

pub use cursor_icon::CursorIcon;

use crate::{Offset, Point};

#[derive(Clone, Debug, PartialEq)]
pub struct Pointer {
    pub(crate) id:       PointerId,
    pub(crate) position: Point,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PointerEvent {
    Down(PointerButtonEvent),
    Up(PointerButtonEvent),
    Move(PointerMoveEvent),
    Scroll(PointerScrollEvent),
    Enter(PointerId),
    Leave(PointerId),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerButtonEvent {
    pub button:   PointerButton,
    pub position: Point,
    pub pointer:  PointerId,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerMoveEvent {
    pub position: Point,
    pub pointer:  PointerId,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ScrollDelta {
    Line(Offset),
    Pixel(Offset),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerScrollEvent {
    pub position: Point,
    pub delta:    ScrollDelta,
    pub pointer:  PointerId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Primary,
    Secondary,
    Tertiary,
    Backward,
    Forward,
    Other(u16),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PointerPropagate {
    Bubble,
    Stop,
    Capture,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerId {
    data: u64,
}

impl PointerId {
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
