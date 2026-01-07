use std::{any::Any, sync::Arc};

use ike_core::{AnyWidgetId, BuildCx, WidgetId, World};
use ori::{Providable, Proxy, Proxyable, Super};

use crate::Resources;

pub struct Context {
    pub world:     World,
    pub proxy:     Arc<dyn Proxy>,
    pub resources: Resources,
}

impl BuildCx for Context {
    fn world(&self) -> &World {
        &self.world
    }

    fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }
}

impl Proxyable for Context {
    type Proxy = Arc<dyn Proxy>;

    fn proxy(&mut self) -> Self::Proxy {
        self.proxy.cloned()
    }
}

impl Providable for Context {
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

pub trait View<T>: ori::View<Context, T, Element: AnyWidgetId> {}
pub trait Effect<T>: ori::Effect<Context, T> {}

impl<T, V> View<T> for V where V: ori::View<Context, T, Element: AnyWidgetId> {}
impl<T, V> Effect<T> for V where V: ori::Effect<Context, T> {}

impl<S> Super<Context, S> for WidgetId
where
    S: AnyWidgetId,
{
    fn upcast(_cx: &mut Context, sub: S) -> Self {
        sub.upcast()
    }

    fn downcast(self) -> S {
        S::downcast_unchecked(self)
    }

    fn downcast_with<T>(&mut self, f: impl FnOnce(&mut S) -> T) -> T {
        let mut id = self.downcast();
        let output = f(&mut id);
        *self = id.upcast();
        output
    }
}
