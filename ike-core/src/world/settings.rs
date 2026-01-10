use crate::{DebugSettings, event::TouchSettings};

#[derive(Debug, Default)]
pub struct Settings {
    pub touch: TouchSettings,
    pub debug: DebugSettings,
}
