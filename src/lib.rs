mod app;
mod error;
mod key;
mod palette;
mod view;
mod winit;

#[doc(hidden)]
#[cfg(feature = "reload")]
pub mod reload;

pub mod views {
    #[allow(clippy::module_inception)]
    #[path = "../views/mod.rs"]
    mod views;

    pub use ori::views::*;
    pub use views::*;
}

pub use ike_core::*;
pub use ori::*;

pub use app::App;
pub use palette::Palette;
pub use view::{Effect, View};
pub use winit::Context;

pub mod prelude {
    pub use ike_core::{
        Axis, BorderWidth, Color, CornerRadius, FontStretch, FontStyle, FontWeight, Padding, Svg,
        TextAlign, TextWrap, Transition, include_svg,
        widgets::{Align, Fit, Justify, NewlineBehaviour, SubmitBehaviour},
    };

    pub use ori::{Action, Event, Proxy, ViewId};
    pub use tracing::{
        debug, debug_span, error, error_span, info, info_span, span, trace, trace_span, warn,
        warn_span,
    };

    pub use crate::{App, Effect, Palette, View, ViewSeq, WindowSizing, views::*};
}
