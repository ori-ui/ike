use std::{
    any::Any,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
};

use crate::{
    Canvas, DrawCx, EventCx, LayoutCx, PointerEvent, PointerPropagate,
    context::UpdateCx,
    math::{Affine, Size, Space},
};

pub trait Widget: Any {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size;

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        let _ = cx;
        let _ = canvas;
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        let _ = cx;
        let _ = update;
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        let _ = cx;
        let _ = event;

        PointerPropagate::Bubble
    }

    fn accepts_pointer() -> bool
    where
        Self: Sized,
    {
        true
    }

    fn accepts_focus() -> bool
    where
        Self: Sized,
    {
        false
    }

    fn accepts_text() -> bool
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
    ChildAdded(usize),
    ChildRemoved(usize),
    ChildReplaced(usize),
    ChildrenSwapped(usize, usize),
}

pub trait AnyWidget: Widget {
    fn downcast_ref(any: &dyn Widget) -> &Self;

    fn downcast_mut(any: &mut dyn Widget) -> &mut Self;
}

impl AnyWidget for dyn Widget {
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
    fn downcast_ref(any: &dyn Widget) -> &Self {
        (any as &dyn Any).downcast_ref().unwrap()
    }

    fn downcast_mut(any: &mut dyn Widget) -> &mut Self {
        (any as &mut dyn Any).downcast_mut().unwrap()
    }
}

#[derive(Debug)]
pub struct WidgetState {
    pub(crate) transform: Affine,
    pub(crate) global_transform: Affine,
    pub(crate) size: Size,
    pub(crate) parent: Option<WidgetId>,
    pub(crate) children: Vec<WidgetId>,

    pub(crate) is_hovered: bool,
    pub(crate) has_hovered: bool,

    pub(crate) is_focused: bool,
    pub(crate) has_focused: bool,

    pub(crate) is_active: bool,
    pub(crate) has_active: bool,

    pub(crate) needs_animation: bool,
    pub(crate) needs_layout: bool,
    pub(crate) needs_draw: bool,

    pub(crate) accepts_pointer: bool,
    pub(crate) accepts_focus: bool,
    pub(crate) accepts_text: bool,
}

impl WidgetState {
    pub fn new<T: Widget>() -> Self {
        Self {
            transform: Affine::IDENTITY,
            global_transform: Affine::IDENTITY,
            size: Size::new(0.0, 0.0),
            parent: None,
            children: Vec::new(),

            is_hovered: false,
            has_hovered: false,

            is_focused: false,
            has_focused: false,

            is_active: false,
            has_active: false,

            needs_animation: false,
            needs_layout: true,
            needs_draw: true,

            accepts_focus: T::accepts_focus(),
            accepts_pointer: T::accepts_pointer(),
            accepts_text: T::accepts_text(),
        }
    }

    pub fn merge(&mut self, child: &Self) {
        self.has_hovered |= child.has_hovered;
        self.has_active |= child.has_active;
        self.has_focused |= child.has_focused;

        self.needs_layout |= child.needs_layout;
        self.needs_draw |= child.needs_draw;
    }
}

#[repr(C)] // we want to be able to cast by reference, so a stable layout is required
pub struct WidgetId<T: ?Sized = dyn Widget> {
    pub(crate) index: u32,
    pub(crate) generation: u32,
    pub(crate) marker: PhantomData<T>,
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

pub trait AnyWidgetId {
    fn upcast(&self) -> WidgetId;
}

pub trait DowncastWidgetId {
    fn downcast_unchecked(id: WidgetId) -> Self;
}

impl<T> AnyWidgetId for WidgetId<T>
where
    T: ?Sized,
{
    fn upcast(&self) -> WidgetId {
        WidgetId {
            index: self.index,
            generation: self.generation,
            marker: PhantomData,
        }
    }
}

impl<T> DowncastWidgetId for WidgetId<T>
where
    T: ?Sized,
{
    fn downcast_unchecked(id: WidgetId) -> Self {
        Self {
            index: id.index,
            generation: id.generation,
            marker: PhantomData,
        }
    }
}

impl<T> AnyWidgetId for &T
where
    T: AnyWidgetId,
{
    fn upcast(&self) -> WidgetId {
        T::upcast(self)
    }
}

impl<T> AnyWidgetId for &mut T
where
    T: AnyWidgetId,
{
    fn upcast(&self) -> WidgetId {
        T::upcast(self)
    }
}
