use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId {
    data: u64,
}

impl fmt::Debug for WindowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "w{:x}", self.data)
    }
}

#[derive(Debug)]
pub struct Window {
    id: WindowId,
    anchor: Option<WindowId>,
}

#[derive(Debug)]
pub struct Windows {
    windows: Vec<Window>,
}

impl Default for Windows {
    fn default() -> Self {
        Self::new()
    }
}

impl Windows {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }
}
