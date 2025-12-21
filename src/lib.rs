#![warn(clippy::unwrap_used)]

mod app;

pub use ike_macro::main;

pub use ike_core::*;
pub use ike_ori::*;

pub use app::{App, Error};

#[doc(hidden)]
#[cfg(backend = "android")]
pub use ike_android::android_main;

pub mod prelude {
    pub use crate::App;

    pub use ike_core::{
        Axis, BorderWidth, Color, CornerRadius, FontStretch, FontStyle, FontWeight, Padding, Svg,
        TextAlign, TextWrap, Transition, include_svg,
        widgets::{Align, Fit, Justify, NewlineBehaviour, SubmitBehaviour},
    };

    pub use ike_ori::{Effect, Palette, View, views::*};
    pub use ori::{Action, Event, Proxy, ViewId};

    pub use tracing::{
        debug, debug_span, error, error_span, info, info_span, span, trace, trace_span, warn,
        warn_span,
    };
}
