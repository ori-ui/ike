use std::ops::Range;

#[derive(Clone, Debug, PartialEq)]
pub enum TextEvent {
    Paste(TextPasteEvent),
    Ime(ImeEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextPasteEvent {
    pub contents: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ImeEvent {
    Enabled,
    Select(Range<usize>),
    Commit(String),
    Disabled,
}
