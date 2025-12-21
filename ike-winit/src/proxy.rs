use std::{
    pin::Pin,
    sync::{Arc, mpsc::Sender},
};

use winit::event_loop::EventLoopProxy;

use crate::Event;

#[derive(Clone)]
pub struct Proxy {
    sender: Sender<Event>,
    proxy:  EventLoopProxy<()>,
}

impl Proxy {
    pub(crate) fn new(sender: Sender<Event>, proxy: EventLoopProxy<()>) -> Self {
        Self { sender, proxy }
    }
}

impl ori::Proxy for Proxy {
    fn cloned(&self) -> Arc<dyn ori::Proxy> {
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
