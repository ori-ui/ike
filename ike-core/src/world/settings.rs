use crate::{DebugSettings, event::TouchSettings, record::RecordSettings};

#[derive(Debug, Default)]
pub struct Settings {
    pub touch:  TouchSettings,
    pub debug:  DebugSettings,
    pub record: RecordSettings,
}
