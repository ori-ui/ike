use std::{any::Any, fmt, rc::Rc};

use crate::{
    Color, CursorIcon, Modifiers, Point, Pointer, PointerId, Rect, Size, Touch, TouchId, WidgetId,
    debug::debug_panic,
};

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

#[derive(Clone, Debug)]
pub struct Layer {
    pub widget:   WidgetId,
    pub size:     Size,
    pub position: Point,
}

#[derive(Debug)]
pub struct Window {
    pub(crate) id:     WindowId,
    pub(crate) layers: Rc<Vec<Layer>>,

    pub(crate) modifiers: Modifiers,
    pub(crate) pointers:  Vec<Pointer>,
    pub(crate) touches:   Vec<Touch>,

    pub(crate) focused: Option<WidgetId>,

    pub(crate) properties: Vec<Box<dyn Any>>,

    pub(crate) scale:        f32,
    pub(crate) size:         Size,
    pub(crate) safe_area:    Option<Rect>,
    pub(crate) is_visible:   bool,
    pub(crate) is_focused:   bool,
    pub(crate) is_decorated: bool,

    pub(crate) cursor: CursorIcon,
    pub(crate) title:  String,
    pub(crate) sizing: WindowSizing,
    pub(crate) color:  Color,
}

impl Window {
    pub fn id(&self) -> WindowId {
        self.id
    }

    pub fn scale(&self) -> f32 {
        self.scale
    }

    pub fn size(&self) -> Size {
        self.size
    }

    /// Get the rect of the window that is safe to use.
    ///
    /// On mobile platforms it is common to have areas of the screen occupied by system bars, IMEs,
    /// or screen cutouts. The returned [`Rect`] is a rect that completely avoids all of those.
    ///
    /// If [`None`] is returned the platform hasn't specified a safe area, all of the screen is
    /// safe to use.
    pub fn safe_area(&self) -> Option<Rect> {
        self.safe_area
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

    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> &mut [Layer] {
        Rc::make_mut(&mut self.layers).as_mut_slice()
    }

    pub fn base_layer(&self) -> Option<&Layer> {
        self.layers().first()
    }

    pub fn base_layer_mut(&mut self) -> Option<&mut Layer> {
        self.layers_mut().first_mut()
    }

    pub(crate) fn pointer(&self, id: PointerId) -> Option<&Pointer> {
        match self.pointers.iter().find(|pointer| pointer.id == id) {
            Some(pointer) => Some(pointer),
            None => {
                debug_panic!("invalid pointer `{id:?}`");
                None
            }
        }
    }

    pub(crate) fn pointer_mut(&mut self, id: PointerId) -> Option<&mut Pointer> {
        match self.pointers.iter_mut().find(|pointer| pointer.id == id) {
            Some(pointer) => Some(pointer),
            None => {
                debug_panic!("invalid pointer `{id:?}`");
                None
            }
        }
    }

    pub(crate) fn touch(&self, id: TouchId) -> Option<&Touch> {
        match self.touches.iter().find(|touch| touch.id == id) {
            Some(touch) => Some(touch),
            None => {
                debug_panic!("invalid touch `{id:?}`");
                None
            }
        }
    }

    pub(crate) fn touch_mut(&mut self, id: TouchId) -> Option<&mut Touch> {
        match self.touches.iter_mut().find(|touch| touch.id == id) {
            Some(touch) => Some(touch),
            None => {
                debug_panic!("invalid touch `{id:?}`");
                None
            }
        }
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
