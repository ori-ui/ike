macro_rules! debug_panic {
    ($($tt:tt)*) => {{
        if cfg!(debug_assertions) {
            panic!($($tt)*);
        }
    }};
}

pub(crate) use debug_panic;

#[derive(Debug)]
pub struct DebugSettings {
    pub trace_widgets:    bool,
    pub recorder_overlay: bool,
}

impl Default for DebugSettings {
    fn default() -> Self {
        Self {
            trace_widgets:    cfg!(debug_assertions),
            recorder_overlay: false,
        }
    }
}
