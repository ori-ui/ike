use std::{
    any::Any,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    time::Duration,
};

use crate::{
    Canvas, DrawCx, EventCx, KeyEvent, LayoutCx, Painter, PointerEvent, PointerPropagate,
    Propagate,
    context::UpdateCx,
    math::{Affine, Size, Space},
};

pub trait Widget: Any {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size;

    fn compose(&mut self, cx: &mut UpdateCx<'_>) {
        let _ = cx;
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        let _ = cx;
        let _ = canvas;
    }

    fn draw_over(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        let _ = cx;
        let _ = canvas;
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        let _ = cx;
        let _ = update;
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        let _ = cx;
        let _ = dt;
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        let _ = cx;
        let _ = event;

        PointerPropagate::Bubble
    }

    fn on_key_event(&mut self, cx: &mut EventCx<'_>, event: &KeyEvent) -> Propagate {
        let _ = cx;
        let _ = event;

        Propagate::Bubble
    }

    fn accepts_pointer() -> bool
    where
        Self: Sized,
    {
        false
    }

    fn accepts_focus() -> bool
    where
        Self: Sized,
    {
        false
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Update {
    Hovered(bool),
    Active(bool),
    Focused(bool),
    Stashed(bool),
    WindowFocused(bool),
    Children(ChildUpdate),
}

#[derive(Clone, Debug, PartialEq)]
pub enum ChildUpdate {
    Added(usize),
    Removed(usize),
    Replaced(usize),
    Swapped(usize, usize),
}

pub trait AnyWidget: Widget {
    fn upcast_boxed(boxed: Box<Self>) -> Box<dyn Widget>;

    fn downcast_boxed(boxed: Box<dyn Widget>) -> Option<Box<Self>>;

    fn downcast_ref(any: &dyn Widget) -> &Self;

    fn downcast_mut(any: &mut dyn Widget) -> &mut Self;
}

impl AnyWidget for dyn Widget {
    fn upcast_boxed(boxed: Box<Self>) -> Box<dyn Widget> {
        boxed
    }

    fn downcast_boxed(boxed: Box<dyn Widget>) -> Option<Box<Self>> {
        Some(boxed)
    }

    fn downcast_ref(any: &dyn Widget) -> &Self {
        any
    }

    fn downcast_mut(any: &mut dyn Widget) -> &mut Self {
        any
    }
}

impl<T> AnyWidget for T
where
    T: Widget,
{
    fn upcast_boxed(boxed: Box<Self>) -> Box<dyn Widget> {
        boxed
    }

    fn downcast_boxed(boxed: Box<dyn Widget>) -> Option<Box<Self>> {
        (boxed as Box<dyn Any>).downcast().ok()
    }

    fn downcast_ref(any: &dyn Widget) -> &Self {
        (any as &dyn Any).downcast_ref().unwrap()
    }

    fn downcast_mut(any: &mut dyn Widget) -> &mut Self {
        (any as &mut dyn Any).downcast_mut().unwrap()
    }
}

#[derive(Debug)]
pub struct WidgetState {
    pub(crate) transform:        Affine,
    pub(crate) global_transform: Affine,
    pub(crate) size:             Size,
    pub(crate) parent:           Option<WidgetId>,
    pub(crate) children:         Vec<WidgetId>,
    pub(crate) is_pixel_perfect: bool,

    pub(crate) is_hovered:  bool,
    pub(crate) has_hovered: bool,

    pub(crate) is_focused:  bool,
    pub(crate) has_focused: bool,

    pub(crate) is_active:  bool,
    pub(crate) has_active: bool,

    /// Whether the widget is currently stashed, either explicitly, or by an ancestor.
    pub(crate) is_stashed:  bool,
    /// Whether the widget is currently stashing itself and descendants.
    pub(crate) is_stashing: bool,

    pub(crate) needs_animate: bool,
    pub(crate) needs_layout:  bool,
    pub(crate) needs_draw:    bool,

    pub(crate) accepts_pointer: bool,
    pub(crate) accepts_focus:   bool,
}

impl WidgetState {
    pub(crate) fn new<T: Widget>() -> Self {
        Self {
            transform:        Affine::IDENTITY,
            global_transform: Affine::IDENTITY,
            size:             Size::new(0.0, 0.0),
            parent:           None,
            children:         Vec::new(),
            is_pixel_perfect: true,

            is_hovered:  false,
            has_hovered: false,

            is_focused:  false,
            has_focused: false,

            is_active:  false,
            has_active: false,

            is_stashed:  false,
            is_stashing: false,

            needs_animate: false,
            needs_layout:  true,
            needs_draw:    true,

            accepts_focus:   T::accepts_focus(),
            accepts_pointer: T::accepts_pointer(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.has_hovered = self.is_hovered;
        self.has_active = self.is_active;
        self.has_focused = self.is_focused;
    }

    pub(crate) fn merge(&mut self, child: &Self) {
        self.has_hovered |= child.has_hovered;
        self.has_active |= child.has_active;
        self.has_focused |= child.has_focused;

        self.needs_animate |= child.needs_animate && !child.is_stashed;
        self.needs_layout |= child.needs_layout && !child.is_stashed;
        self.needs_draw |= child.needs_draw && !child.is_stashed;
    }

    pub fn accepts_focus(&self) -> bool {
        self.accepts_focus && !self.is_stashed
    }

    pub fn accepts_pointer(&self) -> bool {
        self.accepts_pointer && !self.is_stashed
    }
}

#[repr(C)] // we want to be able to cast by reference, so a stable layout is required
pub struct WidgetId<T: ?Sized = dyn Widget> {
    pub(crate) index:      u32,
    pub(crate) generation: u32,
    pub(crate) marker:     PhantomData<T>,
}

impl WidgetId<dyn Widget> {
    pub fn downcast_ref_unchecked<T>(&self) -> &WidgetId<T>
    where
        T: ?Sized,
    {
        unsafe { &*(self as *const _ as *const WidgetId<T>) }
    }
}

impl<T: ?Sized> Clone for WidgetId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: ?Sized> Copy for WidgetId<T> {}

impl<T: ?Sized, U: ?Sized> PartialEq<WidgetId<U>> for WidgetId<T> {
    fn eq(&self, other: &WidgetId<U>) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T: ?Sized> Eq for WidgetId<T> {}

impl<T: ?Sized> Hash for WidgetId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T: ?Sized> fmt::Debug for WidgetId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.index, self.generation)
    }
}

pub trait AnyWidgetId: Copy {
    fn upcast(&self) -> WidgetId;

    fn downcast_unchecked(id: WidgetId) -> Self;
}

impl<T> AnyWidgetId for WidgetId<T>
where
    T: ?Sized,
{
    fn upcast(&self) -> WidgetId {
        WidgetId {
            index:      self.index,
            generation: self.generation,
            marker:     PhantomData,
        }
    }

    fn downcast_unchecked(id: WidgetId) -> Self {
        WidgetId {
            index:      id.index,
            generation: id.generation,
            marker:     PhantomData,
        }
    }
}
