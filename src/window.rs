use std::fmt;

use crate::{Color, Modifiers, Pointer, Size, WidgetId};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId {
    pub(crate) data: u64,
}

impl fmt::Debug for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "w{:x}", self.data)
    }
}

#[derive(Debug)]
pub struct Window {
    pub(crate) id:        WindowId,
    pub(crate) anchor:    Option<WindowId>,
    pub(crate) scale:     f32,
    pub(crate) pointers:  Vec<Pointer>,
    pub(crate) modifiers: Modifiers,

    pub content: WidgetId,
    pub size:    Size,
    pub color:   Color,
}

impl Window {
    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn anchor(&self) -> Option<WindowId> {
        self.anchor
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }
}
