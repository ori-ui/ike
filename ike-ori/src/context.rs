use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
    sync::Arc,
};

use ike_core::{AnyWidgetId, BuildCx, Root, WidgetId};
use ori::{Providable, Proxy, Proxyable, Super};

pub struct Context {
    root:      Root,
    proxy:     Arc<dyn Proxy>,
    resources: Vec<Resouce>,

    use_type_names_unsafe: bool,
}

impl Context {
    pub fn new(root: Root, proxy: Arc<dyn Proxy>) -> Self {
        Self {
            root,
            proxy,
            resources: Vec::new(),
            use_type_names_unsafe: false,
        }
    }
}

impl Deref for Context {
    type Target = Root;

    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

impl DerefMut for Context {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.root
    }
}

struct Resouce {
    value:     Box<dyn Any>,
    type_id:   TypeId,
    type_name: &'static str,
}

impl BuildCx for Context {
    fn root(&self) -> &Root {
        &self.root
    }

    fn root_mut(&mut self) -> &mut Root {
        &mut self.root
    }
}

impl Proxyable for Context {
    type Proxy = Arc<dyn Proxy>;

    fn proxy(&mut self) -> Self::Proxy {
        self.proxy.clone()
    }
}

impl Providable for Context {
    fn push<T: Any>(&mut self, context: Box<T>) {
        self.resources.push(Resouce {
            value:     context,
            type_id:   TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        })
    }

    fn pop<T: Any>(&mut self) -> Option<Box<T>> {
        self.resources.pop()?.value.downcast().ok()
    }

    fn get<T: Any>(&self) -> Option<&T> {
        let entry = match self.use_type_names_unsafe {
            true => self
                .resources
                .iter()
                .rfind(|e| e.type_name == std::any::type_name::<T>())?,
            false => self
                .resources
                .iter()
                .rfind(|e| e.type_id == TypeId::of::<T>())?,
        };

        Some(unsafe { &*(entry.value.as_ref() as *const _ as *const T) })
    }

    fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        let entry = match self.use_type_names_unsafe {
            true => self
                .resources
                .iter_mut()
                .rfind(|e| e.type_name == std::any::type_name::<T>())?,
            false => self
                .resources
                .iter_mut()
                .rfind(|e| e.type_id == TypeId::of::<T>())?,
        };

        Some(unsafe { &mut *(entry.value.as_mut() as *mut _ as *mut T) })
    }
}

pub trait View<T>: ori::View<Context, T, Element: AnyWidgetId> {}
pub trait Effect<T>: ori::Effect<Context, T> {}

impl<T, V> View<T> for V
where
    V: ori::View<Context, T>,
    V::Element: AnyWidgetId,
{
}
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
