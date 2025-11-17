use crate::{Point, Tree};

#[derive(Clone, Debug, PartialEq)]
pub enum PointerEvent {
    Down(PointerButtonEvent),
    Up(PointerButtonEvent),
    Move(PointerMoveEvent),
    Enter(PointerMoveEvent),
    Leave(PointerMoveEvent),
}

impl PointerEvent {
    pub fn position(&self) -> Point {
        match self {
            PointerEvent::Down(event) => event.position,
            PointerEvent::Up(event) => event.position,
            PointerEvent::Move(event) => event.position,
            PointerEvent::Enter(event) => event.position,
            PointerEvent::Leave(event) => event.position,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerMoveEvent {
    pub position: Point,
    pub id: Option<PointerId>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerButtonEvent {
    pub button: PointerButton,
    pub position: Point,
    pub id: Option<PointerId>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Tertiary,
    Other(u16),
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerId {}
