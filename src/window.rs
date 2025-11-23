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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowSizing {
    FitContent,
    Resizable {
        default_size: Size,
        min_size:     Size,
        max_size:     Size,
    },
}

#[derive(Debug)]
pub struct Window {
    pub(crate) id:           WindowId,
    pub(crate) anchor:       Option<WindowId>,
    pub(crate) scale:        f32,
    pub(crate) pointers:     Vec<Pointer>,
    pub(crate) modifiers:    Modifiers,
    pub(crate) is_focused:   bool,
    pub(crate) current_size: Size,

    pub title:     String,
    pub content:   WidgetId,
    pub sizing:    WindowSizing,
    pub visible:   bool,
    pub decorated: bool,
    pub color:     Color,
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

    pub fn current_size(&self) -> Size {
        self.current_size
    }

    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }
}
