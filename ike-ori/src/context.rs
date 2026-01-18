use std::{any::Any, mem, sync::Arc};

use ike_core::{AnyWidgetId, Builder, Widget, WidgetId, World};
use ori::{BaseElement, Element, Provider, Proxied, Proxy, Super};

use crate::Resources;

pub struct Context {
    pub world:     World,
    pub proxy:     Arc<dyn Proxy>,
    pub resources: Resources,
}

impl Builder for Context {
    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl BaseElement for Context {
    type Element = WidgetId;
}

impl Proxied for Context {
    type Proxy = Arc<dyn Proxy>;

    fn proxy(&mut self) -> Self::Proxy {
        self.proxy.cloned()
    }
}

impl Provider for Context {
    fn push<T: Any>(&mut self, resource: Box<T>) {
        self.resources.push(resource);
    }

    fn pop<T: Any>(&mut self) -> Option<Box<T>> {
        self.resources.pop()
    }

    fn get<T: Any>(&self) -> Option<&T> {
        self.resources.get()
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.resources.get_mut()
    }
}

pub trait View<T>: ori::View<Context, T, Element = WidgetId<Self::Widget>> {
    type Widget: Widget + ?Sized;
}

impl<T, V, W> View<T> for V
where
    V: ori::View<Context, T, Element = WidgetId<W>>,
    W: Widget + ?Sized,
{
    type Widget = W;
}

pub trait Effect<T>: ori::Effect<Context, T> {}

impl<T, V> Effect<T> for V where V: ori::Effect<Context, T> {}

impl<T> Element<Context> for WidgetId<T>
where
    T: ?Sized,
{
    type Mut<'a>
        = &'a mut WidgetId<T>
    where
        T: 'a;
}

impl<T> Super<Context, WidgetId<T>> for WidgetId
where
    T: ?Sized,
{
    fn replace(cx: &mut Context, this: &mut Self, other: WidgetId<T>) -> Self {
        cx.replace_widget(*this, other);
        mem::replace(this, other.upcast())
    }

    fn upcast(_cx: &mut Context, sub: WidgetId<T>) -> Self {
        sub.upcast()
    }

    fn downcast(self) -> WidgetId<T> {
        AnyWidgetId::downcast_unchecked(self)
    }

    fn downcast_with<U>(this: &mut Self, f: impl FnOnce(&mut WidgetId<T>) -> U) -> U {
        let mut id = this.downcast();
        let output = f(&mut id);
        *this = id.upcast();
        output
    }
}
