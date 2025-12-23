use std::{
    any::Any,
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    time::Duration,
};

use crate::{
    Canvas, Clip, ComposeCx, DrawCx, EventCx, KeyEvent, LayoutCx, Painter, Point, PointerEvent,
    PointerPropagate, Propagate, Rect, RefCx, Size, Space, TextEvent, TouchEvent, TouchPropagate,
    UpdateCx,
};

mod state;

pub use state::WidgetState;

pub trait Widget: Any {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size;

    fn compose(&mut self, cx: &mut ComposeCx<'_>) {
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

    fn on_touch_event(&mut self, cx: &mut EventCx<'_>, event: &TouchEvent) -> TouchPropagate {
        let _ = cx;
        let _ = event;

        TouchPropagate::Bubble
    }

    fn on_key_event(&mut self, cx: &mut EventCx<'_>, event: &KeyEvent) -> Propagate {
        let _ = cx;
        let _ = event;

        Propagate::Bubble
    }

    fn on_text_event(&mut self, cx: &mut EventCx<'_>, event: &TextEvent) -> Propagate {
        let _ = cx;
        let _ = event;

        Propagate::Bubble
    }

    fn find_widget_at(&self, cx: &RefCx<'_>, point: Point) -> Option<WidgetId> {
        let local = cx.global_transform().inverse() * point;

        if !cx.rect().contains(local) || cx.is_stashed() {
            return None;
        }

        match cx.clip() {
            Some(Clip::Rect(rect, _)) => {
                if !rect.contains(local) {
                    return None;
                }
            }

            None => {}
        }

        for child in cx.iter_children() {
            if let Some(widget) = child.widget.find_widget_at(&child.cx, point) {
                return Some(widget);
            }
        }

        if cx.state.accepts_pointer {
            Some(cx.id())
        } else {
            None
        }
    }

    fn accepts_pointer() -> bool
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
    Disabled(bool),
    ScrollTo(Rect),
    WindowFocused(bool),
    WindowResized(Size),
    WindowScaleChanged(f32),
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
    fn upcast_mut(&mut self) -> &mut dyn Widget;

    fn downcast_ptr(ptr: *mut dyn Widget) -> *mut Self;

    fn is(widget: &dyn Widget) -> bool;
}

impl AnyWidget for dyn Widget {
    fn upcast_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn downcast_ptr(ptr: *mut dyn Widget) -> *mut Self {
        ptr
    }

    fn is(_widget: &dyn Widget) -> bool {
        true
    }
}

impl<T> AnyWidget for T
where
    T: Widget,
{
    fn upcast_mut(&mut self) -> &mut dyn Widget {
        self
    }

    fn downcast_ptr(ptr: *mut dyn Widget) -> *mut Self {
        ptr as *mut Self
    }

    fn is(widget: &dyn Widget) -> bool {
        (widget as &dyn Any).is::<T>()
    }
}

// we want to be able to cast by reference, so a stable layout is required
#[repr(C)]
pub struct WidgetId<T: ?Sized = dyn Widget> {
    pub(crate) index:      u32,
    pub(crate) generation: u32,
    pub(crate) marker:     PhantomData<T>,
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

impl<T: ?Sized, U: ?Sized> PartialOrd<WidgetId<U>> for WidgetId<T> {
    fn partial_cmp(&self, other: &WidgetId<U>) -> Option<Ordering> {
        Some(
            self.index
                .cmp(&other.index)
                .then(self.generation.cmp(&other.generation)),
        )
    }
}

impl<T: ?Sized> Ord for WidgetId<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.index
            .cmp(&other.index)
            .then(self.generation.cmp(&other.generation))
    }
}

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
