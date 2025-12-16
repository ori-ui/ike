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
    Preedit(String, Option<(usize, usize)>),
    Commit(String),
    Disabled,
}
