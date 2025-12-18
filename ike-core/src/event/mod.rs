mod key;
mod pointer;
mod text;
mod touch;

pub use key::*;
pub use pointer::*;
pub use text::*;
pub use touch::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Propagate {
    Bubble,
    Handled,
}
