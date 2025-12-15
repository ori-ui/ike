use std::{
    any::{Any, TypeId},
    pin::Pin,
    sync::{Arc, mpsc::Sender},
};

use ike_core::{AnyWidgetId, BuildCx, Root, WidgetId};

use crate::android::Event;

pub struct Context {
    pub(super) root:    Root,
    pub(super) entries: Vec<Entry>,
    pub(super) proxy:   Proxy,
}

impl Context {
    pub(super) fn new(proxy: Proxy) -> Self {
        Self {
            root: Root::new({
                let proxy = proxy.clone();
                move |signal| proxy.send(Event::Signal(signal))
            }),

            entries: Vec::new(),
            proxy,
        }
    }
}

pub(super) struct Entry {
    value:   Box<dyn Any>,
    type_id: TypeId,
}

impl Context {
    pub fn create_window(&mut self, contents: impl AnyWidgetId) -> ike_core::WindowId {
        self.root.create_window(contents.upcast())
    }

    pub fn remove_window(&mut self, window: ike_core::WindowId) {
        self.root.remove_window(window)
    }
}

impl BuildCx for Context {
    fn root(&self) -> &Root {
        &self.root
    }

    fn root_mut(&mut self) -> &mut Root {
        &mut self.root
    }
}

#[derive(Clone)]
pub struct Proxy {
    sender: Sender<Event>,
    looper: *mut ndk_sys::ALooper,
}

unsafe impl Send for Proxy {}
unsafe impl Sync for Proxy {}

impl Proxy {
    pub(super) fn new(sender: Sender<Event>, looper: *mut ndk_sys::ALooper) -> Self {
        Self { sender, looper }
    }

    pub(super) fn send(&self, event: Event) {
        let _ = self.sender.send(event);
        unsafe { ndk_sys::ALooper_wake(self.looper) };
    }
}

impl ori::BaseElement for Context {
    type Element = WidgetId;
}

impl ori::AsyncContext for Context {
    type Proxy = Proxy;

    fn proxy(&mut self) -> Self::Proxy {
        self.proxy.clone()
    }
}

impl ori::ProviderContext for Context {
    fn push_context<T: Any>(&mut self, context: Box<T>) {
        self.entries.push(Entry {
            value:   context,
            type_id: TypeId::of::<T>(),
        })
    }

    fn pop_context<T: Any>(&mut self) -> Option<Box<T>> {
        self.entries.pop()?.value.downcast().ok()
    }

    fn get_context<T: Any>(&self) -> Option<&T> {
        let entry = self
            .entries
            .iter()
            .rfind(|e| e.type_id == TypeId::of::<T>())?;

        Some(unsafe { &*(entry.value.as_ref() as *const _ as *const T) })
    }

    fn get_context_mut<T: Any>(&mut self) -> Option<&mut T> {
        let entry = self
            .entries
            .iter_mut()
            .rfind(|e| e.type_id == TypeId::of::<T>())?;

        Some(unsafe { &mut *(entry.value.as_mut() as *mut _ as *mut T) })
    }
}

impl ori::Proxy for Proxy {
    fn clone(&self) -> Arc<dyn ori::Proxy> {
        Arc::new(Clone::clone(self))
    }

    fn rebuild(&self) {
        self.send(Event::Rebuild);
    }

    fn event(&self, event: ori::Event) {
        self.send(Event::Event(event));
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.send(Event::Future(future));
    }
}
