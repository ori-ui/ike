#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TextWrap {
    None,
    Word,
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub line_height: f32,
    pub align: TextAlign,
    pub wrap: TextWrap,
    text: String,
    styles: Vec<(TextStyle, usize)>,
}

impl Paragraph {
    pub const fn new(line_height: f32, align: TextAlign, wrap: TextWrap) -> Self {
        Self {
            line_height,
            align,
            wrap,
            text: String::new(),
            styles: Vec::new(),
        }
    }

    pub fn push(&mut self, text: impl AsRef<str>, style: TextStyle) {
        self.text.push_str(text.as_ref());
        self.styles.push((style, text.as_ref().len()));
    }

    pub fn sections(&self) -> impl Iterator<Item = (&str, &TextStyle)> {
        let mut prev = 0;

        self.styles.iter().map(move |(style, len)| {
            let text = &self.text[prev..prev + len];
            prev += len;

            (text, style)
        })
    }
}

#[derive(Clone, Debug)]
pub struct TextStyle {
    pub font_size: f32,
    pub font_family: String,
}
