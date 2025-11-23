use core::f32;
use std::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use crate::{
    Canvas, Color, Modifiers, Size, Tree, Update, Widget, WidgetId, Window, WindowId,
    widget::WidgetState, window::WindowSizing,
};

mod animate;
mod draw;
mod focus;
mod key;
mod layout;
mod pointer;
mod update;

pub struct App {
    pub(crate) tree:    Tree,
    pub(crate) windows: Vec<Window>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self {
            tree:    Tree::new(),
            windows: Vec::new(),
        }
    }

    pub fn create_window(&mut self, content: WidgetId) -> &mut Window {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        let id = WindowId {
            data: NEXT_ID.fetch_add(1, Ordering::SeqCst),
        };

        self.windows.push(Window {
            id,
            anchor: None,
            scale: 1.0,
            pointers: Vec::new(),
            modifiers: Modifiers::empty(),
            is_focused: false,
            current_size: Size::new(800.0, 600.0),

            content,
            sizing: WindowSizing::Resizable {
                default_size: Size::new(800.0, 600.0),
                min_size:     Size::all(0.0),
                max_size:     Size::all(f32::INFINITY),
            },
            color: Color::WHITE,
        });

        self.windows.last_mut().unwrap()
    }

    pub fn remove_window(&mut self, window: WindowId) {
        self.windows.retain(|w| w.id() != window)
    }

    pub fn get_window(&self, id: WindowId) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn get_window_mut(&mut self, id: WindowId) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }

    pub fn windows(&self) -> impl Iterator<Item = &Window> {
        self.windows.iter()
    }

    #[must_use]
    pub fn draw(&mut self, window: WindowId, canvas: &mut dyn Canvas) -> Option<Size> {
        let window = self.windows.iter_mut().find(|w| w.id == window)?;

        let new_window_size = layout::layout_window(&mut self.tree, canvas.fonts(), window);
        draw::draw_widget(&mut self.tree, window, window.content, canvas);
        draw::draw_over_widget(&mut self.tree, window, window.content, canvas);

        new_window_size
    }

    pub fn animate(&mut self, window: WindowId, delta_time: Duration) {
        let Some(window) = self.windows.iter().find(|w| w.id == window) else {
            return;
        };

        animate::animate_widget(&mut self.tree, window.content, delta_time);
    }

    pub fn window_focused(&mut self, window: WindowId, is_focused: bool) {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return;
        };

        window.is_focused = is_focused;

        let update = Update::WindowFocused(is_focused);
        update::update_widget(&mut self.tree, window.content, &update);
    }

    pub fn window_resized(&mut self, window: WindowId, new_size: Size) {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return;
        };

        window.current_size = new_size;

        self.tree.request_layout(window.content);
    }

    pub fn window_scaled(&mut self, window: WindowId, new_scale: f32, new_size: Size) {
        let Some(window) = self.windows.iter_mut().find(|w| w.id == window) else {
            return;
        };

        window.scale = new_scale;
        window.current_size = new_size;

        self.tree.request_layout(window.content);
    }

    pub fn window_needs_animate(&self, window: WindowId) -> bool {
        match self.windows.iter().find(|w| w.id == window) {
            Some(window) => self.tree.needs_animate(window.content),
            None => false,
        }
    }

    pub fn window_needs_draw(&self, window: WindowId) -> bool {
        match self.windows.iter().find(|w| w.id == window) {
            Some(window) => self.tree.needs_draw(window.content),
            None => false,
        }
    }
}

impl App {
    pub(crate) fn with_entry<T, U>(
        &mut self,
        id: WidgetId<T>,
        f: impl FnOnce(&mut Self, &mut dyn Widget, &mut WidgetState) -> U,
    ) -> U
    where
        T: ?Sized,
    {
        let (mut widget, mut state) = self.tree.take_entry(id).unwrap();
        let output = { f(self, widget.as_mut(), &mut state) };
        self.tree.insert_entry(id, widget, state);
        output
    }
}
