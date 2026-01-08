use std::time::Instant;

use cursor_icon::CursorIcon;

use crate::{
    Color, Recorder, Signal, Window, WindowId, WindowSizing, WindowUpdate, debug::debug_panic,
    event::TouchSettings,
};

pub struct WorldState {
    signaller: Box<dyn Fn(Signal)>,

    pub(super) windows: Vec<Window>,

    pub should_trace:   bool,
    pub touch_settings: TouchSettings,
    pub recorder:       Recorder,
}

impl WorldState {
    pub(crate) fn new(signaller: Box<dyn Fn(Signal)>) -> Self {
        Self {
            signaller,

            windows: Vec::new(),

            should_trace: cfg!(debug_assertions),
            touch_settings: TouchSettings::default(),
            recorder: Recorder::new(),
        }
    }
}

impl WorldState {
    pub(crate) fn window(&self, id: WindowId) -> Option<&Window> {
        match self.get_window(id) {
            Some(window) => Some(window),
            None => {
                debug_panic!("invalid window `{id:?}`");
                None
            }
        }
    }

    pub(crate) fn window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        match self.get_window_mut(id) {
            Some(window) => Some(window),
            None => {
                debug_panic!("invalid window `{id:?}`");
                None
            }
        }
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    pub fn set_window_title(&self, window: WindowId, title: String) {
        self.emit_signal(Signal::UpdateWindow(
            window,
            WindowUpdate::Title(title),
        ));
    }

    pub fn set_window_sizing(&self, window: WindowId, sizing: WindowSizing) {
        self.emit_signal(Signal::UpdateWindow(
            window,
            WindowUpdate::Sizing(sizing),
        ));
    }

    pub fn set_window_visible(&self, window: WindowId, visible: bool) {
        self.emit_signal(Signal::UpdateWindow(
            window,
            WindowUpdate::Visible(visible),
        ));
    }

    pub fn set_window_decorated(&self, window: WindowId, decorated: bool) {
        self.emit_signal(Signal::UpdateWindow(
            window,
            WindowUpdate::Decorated(decorated),
        ));
    }

    pub fn set_window_color(&mut self, window: WindowId, color: Color) {
        if let Some(window) = self.window_mut(window) {
            window.color = color;
        }
    }

    pub fn set_window_cursor(&mut self, window: WindowId, cursor: CursorIcon) {
        if let Some(window) = self.window_mut(window) {
            window.cursor = cursor;
        }
    }
}

impl WorldState {
    pub(crate) fn emit_signal(&self, signal: Signal) {
        (self.signaller)(signal);
    }

    pub(crate) fn request_animate(&self, window: WindowId) {
        self.emit_signal(Signal::RequestAnimate {
            window,
            start: Instant::now(),
        });
    }

    pub(crate) fn request_redraw(&self, window: WindowId) {
        self.emit_signal(Signal::RequestRedraw { window });
    }
}
