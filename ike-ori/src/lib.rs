#![warn(clippy::unwrap_used)]

pub mod views {
    #[allow(clippy::module_inception)]
    #[path = "../views/mod.rs"]
    mod views;

    pub use ori::views::*;
    pub use views::*;
}

mod context;
mod palette;

pub use context::{Context, Effect, View};
pub use palette::Palette;

pub type AnyEffect<T> = Box<dyn ori::AnyView<Context, T, ori::NoElement>>;
pub type UiBuilder<T> = Box<dyn FnMut(&mut T) -> AnyEffect<T>>;
