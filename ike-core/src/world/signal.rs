use std::{ops::Range, time::Instant};

use cursor_icon::CursorIcon;

use crate::{Rect, WindowId, WindowSizing};

#[derive(Clone, Debug)]
pub enum Signal {
    RequestRedraw(WindowId),
    RequestAnimate(WindowId, Instant),

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
