pub use keyboard_types::{Key, Modifiers, NamedKey};

#[derive(Clone, Debug, PartialEq)]
pub enum KeyEvent {
    Down(KeyPressEvent),
    Up(KeyPressEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyPressEvent {
    pub key:       Key,
    pub modifiers: Modifiers,
    pub text:      Option<String>,
    pub repeat:    bool,
}
