use std::{
    pin::Pin,
    sync::{Arc, mpsc::Sender},
};

use crate::Event;

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
        self.wake();
    }

    pub(super) fn wake(&self) {
        unsafe { ndk_sys::ALooper_wake(self.looper) };
    }
}

impl ori::Proxy for Proxy {
    fn cloned(&self) -> Arc<dyn ori::Proxy> {
        Arc::new(self.clone())
    }

    fn rebuild(&self) {
        self.send(Event::Rebuild);
    }

    fn event(&self, event: ori::Event) {
        self.send(Event::Event(event))
    }

    fn spawn_boxed(&self, future: Pin<Box<dyn Future<Output = ()> + Send>>) {
        self.send(Event::Future(future));
    }
}
