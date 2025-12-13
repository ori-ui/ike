mod key;
mod pointer;
mod text;

pub use key::*;
pub use pointer::*;
pub use text::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Propagate {
    Bubble,
    Stop,
}
