macro_rules! debug_panic {
    ($($tt:tt)*) => {{
        if cfg!(debug_assertions) {
            panic!($($tt)*);
        }
    }};
}

pub(crate) use debug_panic;
