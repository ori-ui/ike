use cursor_icon::CursorIcon;

use crate::{WindowId, WindowSizing};

#[derive(Clone, Debug)]
pub enum RootSignal {
    RequestRedraw(WindowId),
    RequestAnimate(WindowId),

    ClipboardSet(String),

    CreateWindow(WindowId),
    RemoveWindow(WindowId),
    UpdateWindow(WindowId, WindowUpdate),
}

#[derive(Clone, Debug)]
pub enum WindowUpdate {
    Title(String),
    Sizing(WindowSizing),
    Visible(bool),
    Decorated(bool),
    Cursor(CursorIcon),
}
