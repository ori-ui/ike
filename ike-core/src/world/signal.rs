use std::{fmt, ops::Range, time::Instant};

use crate::{CursorIcon, Rect, WindowId, WindowSizing, World};

pub enum Signal {
    /// `window` needs to be redraw.
    RequestRedraw { window: WindowId },

    /// `window` needs to be animated.
    ///
    /// `start` is the time at which animate was requested.
    RequestAnimate { window: WindowId, start: Instant },

    /// Mutate the [`World`] through a closure.
    Mutate(Box<dyn FnOnce(&mut World) + Send>),

    /// Set the text contents of the clipboard.
    ClipboardSet(String),

    /// Create a window.
    CreateWindow(WindowId),

    /// Remove a window.
    RemoveWindow(WindowId),

    /// Update a window.
    UpdateWindow(WindowId, WindowUpdate),

    /// Perform an action pertaining to IME.
    Ime(ImeSignal),
}

impl fmt::Debug for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestRedraw { window } => f
                .debug_struct("RequestRedraw")
                .field("window", window)
                .finish(),

            Self::RequestAnimate { window, start } => f
                .debug_struct("RequestAnimate")
                .field("window", window)
                .field("start", start)
                .finish(),

            Self::Mutate(..) => f.debug_tuple("Deferred").finish_non_exhaustive(),

            Self::ClipboardSet(contents) => f.debug_tuple("ClipboardSet").field(contents).finish(),

            Self::CreateWindow(window) => f.debug_tuple("CreateWindow").field(window).finish(),

            Self::RemoveWindow(window) => f.debug_tuple("RemoveWindow").field(window).finish(),

            Self::UpdateWindow(window, update) => f
                .debug_tuple("UpdateWindow")
                .field(window)
                .field(update)
                .finish(),

            Self::Ime(ime) => f.debug_tuple("Ime").field(ime).finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum WindowUpdate {
    Title(String),
    Sizing(WindowSizing),
    Visible(bool),
    Decorated(bool),
    Cursor(CursorIcon),
}

#[derive(Clone, Debug)]
pub enum ImeSignal {
    Start,
    End,
    Area(Rect),
    Text(String),
    Selection {
        selection: Range<usize>,
        composing: Option<Range<usize>>,
    },
}
