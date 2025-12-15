use std::{
    any::{Any, TypeId},
    pin::Pin,
    sync::{Arc, mpsc::Sender},
};

use ike_core::{AnyWidgetId, BuildCx, Root, WidgetId};
use winit::event_loop::EventLoopProxy;

use crate::winit::Event;

pub struct Context {
    pub(super) root:    Root,
    pub(super) proxy:   EventLoopProxy<()>,
    pub(super) entries: Vec<Entry>,
    pub(super) sender:  Sender<Event>,

    pub(super) use_type_names_unsafe: bool,
}

impl Context {
    pub fn create_window(&mut self, contents: impl AnyWidgetId) -> ike_core::WindowId {
        self.root.create_window(contents.upcast())
    }

    pub fn remove_window(&mut self, window: ike_core::WindowId) {
        self.root.remove_window(window)
    }

    #[allow(dead_code)]
    pub(crate) fn is_using_type_names(&self) -> bool {
        self.use_type_names_unsafe
    }

    #[allow(dead_code)]
    pub(crate) unsafe fn use_type_names(&mut self, enabled: bool) {
        self.use_type_names_unsafe = enabled;
    }
}

pub(super) struct Entry {
    value:     Box<dyn Any>,
    type_id:   TypeId,
    type_name: &'static str,
}

#[derive(Clone)]
pub struct Proxy {
    sender: Sender<Event>,
    proxy:  EventLoopProxy<()>,
}

impl BuildCx for Context {
    fn root(&self) -> &Root {
        &self.root
    }

    fn root_mut(&mut self) -> &mut Root {
        &mut self.root
    }
}

impl ori::BaseElement for Context {
    type Element = WidgetId;
}

impl ori::AsyncContext for Context {
    type Proxy = Proxy;

    fn proxy(&mut self) -> Self::Proxy {
        Proxy {
            sender: self.sender.clone(),
            proxy:  self.proxy.clone(),
        }
    }
}

impl ori::ProviderContext for Context {
    fn push_context<T: Any>(&mut self, context: Box<T>) {
        self.entries.push(Entry {
            value:     context,
            type_id:   TypeId::of::<T>(),
            type_name: std::any::type_name::<T>(),
        })
    }

    fn pop_context<T: Any>(&mut self) -> Option<Box<T>> {
        self.entries.pop()?.value.downcast().ok()
    }

    fn get_context<T: Any>(&self) -> Option<&T> {
        let entry = match self.use_type_names_unsafe {
            true => self
                .entries
                .iter()
                .rfind(|e| e.type_name == std::any::type_name::<T>())?,
            false => self
                .entries
                .iter()
                .rfind(|e| e.type_id == TypeId::of::<T>())?,
        };

        Some(unsafe { &*(entry.value.as_ref() as *const _ as *const T) })
    }

    fn get_context_mut<T: Any>(&mut self) -> Option<&mut T> {
        let entry = match self.use_type_names_unsafe {
            true => self
                .entries
                .iter_mut()
                .rfind(|e| e.type_name == std::any::type_name::<T>())?,
            false => self
                .entries
                .iter_mut()
                .rfind(|e| e.type_id == TypeId::of::<T>())?,
        };

        Some(unsafe { &mut *(entry.value.as_mut() as *mut _ as *mut T) })
    }
}

impl ori::Proxy for Proxy {
    fn clone(&self) -> Arc<dyn ori::Proxy> {
        Arc::new(Clone::clone(self))
    }

    fn rebuild(&self) {
        let _ = self.sender.send(Event::Rebuild);
        let _ = self.proxy.send_event(());
    }

    fn event(&self, event: ori::Event) {
        let _ = self.sender.send(Event::Event(event));
        let _ = self.proxy.send_event(());
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        let _ = self.sender.send(Event::Spawn(future));
        let _ = self.proxy.send_event(());
    }
}
