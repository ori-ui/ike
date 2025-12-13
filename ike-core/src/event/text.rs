#[derive(Clone, Debug, PartialEq)]
pub enum TextEvent {
    Paste(TextPasteEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextPasteEvent {
    pub contents: String,
}
