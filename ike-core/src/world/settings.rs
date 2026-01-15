use crate::{DebugSettings, event::TouchSettings, record::RecordSettings};

#[derive(Debug, Default)]
pub struct Settings {
    pub touch:  TouchSettings,
    pub debug:  DebugSettings,
    pub record: RecordSettings,
    pub render: RenderSettings,
}

#[derive(Debug)]
pub struct RenderSettings {
    pub pixel_align: bool,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self { pixel_align: true }
    }
}
