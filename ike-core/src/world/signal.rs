use std::{ops::Range, time::Instant};

use crate::{CursorIcon, Rect, WindowId, WindowSizing};

#[derive(Clone, Debug)]
pub enum Signal {
    RequestRedraw { window: WindowId },
    RequestAnimate { window: WindowId, start: Instant },

    ClipboardSet(String),

    CreateWindow(WindowId),
    RemoveWindow(WindowId),
    UpdateWindow(WindowId, WindowUpdate),

    Ime(ImeSignal),
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
