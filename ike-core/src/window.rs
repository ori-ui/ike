use std::{any::Any, fmt};

use crate::{Color, CursorIcon, Modifiers, Pointer, Size, WidgetId};

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
    pub(crate) size:         Size,
    pub(crate) properties:   Vec<Box<dyn Any>>,
    pub(crate) cursor:       CursorIcon,
    pub(crate) title:        String,
    pub(crate) contents:     WidgetId,
    pub(crate) sizing:       WindowSizing,
    pub(crate) is_focused:   bool,
    pub(crate) is_visible:   bool,
    pub(crate) is_decorated: bool,
    pub(crate) color:        Color,
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
        self.size
    }

    pub fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    pub fn cursor(&self) -> CursorIcon {
        self.cursor
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn sizing(&self) -> WindowSizing {
        self.sizing
    }

    pub fn contents(&self) -> WidgetId {
        self.contents
    }

    pub fn is_focused(&self) -> bool {
        self.is_focused
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn is_decorated(&self) -> bool {
        self.is_decorated
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn add_property<T: Any>(&mut self, property: T) {
        match self.get_property_mut() {
            Some(prop) => *prop = property,
            None => self.properties.push(Box::new(property)),
        }
    }

    pub fn remove_property<T: Any>(&mut self) -> Option<T> {
        let index = self.properties.iter().position(|p| p.as_ref().is::<T>())?;
        Some(*self.properties.swap_remove(index).downcast().ok()?)
    }

    pub fn get_property<T: Any>(&self) -> Option<&T> {
        self.properties.iter().find_map(|p| p.downcast_ref())
    }

    pub fn get_property_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.properties.iter_mut().find_map(|p| p.downcast_mut())
    }
}
