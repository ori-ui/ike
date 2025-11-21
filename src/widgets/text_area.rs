use std::time::Duration;

use keyboard_types::NamedKey;

use crate::{
    BuildCx, Canvas, Color, CornerRadius, DrawCx, EventCx, Key, KeyEvent, LayoutCx, Offset, Paint,
    Paragraph, Point, PointerEvent, PointerPropagate, Propagate, Rect, Size, Space, TextLayoutLine,
    Update, UpdateCx, Widget, WidgetId,
};

pub struct TextArea {
    paragraph:       Paragraph,
    selection_color: Color,
    cursor_color:    Color,

    lines:     Vec<TextLayoutLine>,
    cursor:    usize,
    selection: Option<usize>,
    blink:     f32,
}

impl TextArea {
    pub fn new(cx: &mut impl BuildCx, paragraph: Paragraph) -> WidgetId<Self> {
        let cursor = paragraph.text.len();

        cx.insert(Self {
            paragraph,
            selection_color: Color::BLUE,
            cursor_color: Color::WHITE,

            lines: Vec::new(),
            cursor,
            selection: None,
            blink: 0.0,
        })
    }

    fn set_cursor(&mut self, cursor: usize, select: bool) {
        if !select {
            self.selection = None
        } else if self.selection.is_none() {
            self.selection = Some(self.cursor);
        }

        self.cursor = cursor;
        self.blink = 0.0;
    }

    fn find_point(&self, point: Point) -> usize {
        for line in &self.lines {
            if point.y <= line.bottom() {
                for glyph in &line.glyphs {
                    if point.x < glyph.bounds.center().x {
                        return glyph.start_index;
                    }
                }

                return line.end_index;
            }
        }

        self.paragraph.text().len()
    }

    fn move_forward(&mut self, select: bool) {
        if let Some(selection) = self.selection
            && !select
        {
            self.set_cursor(self.cursor.max(selection), false);
            return;
        }

        if self.cursor >= self.paragraph.text.len() {
            return;
        }

        if let Some(next_char) = self.paragraph.text[self.cursor..].chars().next() {
            self.set_cursor(self.cursor + next_char.len_utf8(), select);
        };
    }

    fn move_backward(&mut self, select: bool) {
        if let Some(selection) = self.selection
            && !select
        {
            self.set_cursor(self.cursor.min(selection), false);
            return;
        }

        if self.cursor == 0 {
            return;
        }

        if let Some(prev_char) = self.paragraph.text[..self.cursor].chars().next_back() {
            self.set_cursor(self.cursor - prev_char.len_utf8(), select);
        };
    }

    fn remove_selection(&mut self) -> bool {
        let Some(selection) = self.selection else {
            return false;
        };

        let start = usize::min(self.cursor, selection);
        let end = usize::max(self.cursor, selection);

        self.paragraph.text.drain(start..end);
        self.selection = None;
        self.set_cursor(start, false);

        true
    }

    fn insert_text(&mut self, text: &str) {
        self.remove_selection();
        self.paragraph.text.insert_str(self.cursor, text);
        self.set_cursor(self.cursor + text.len(), false);
    }

    fn draw_selection(&self, canvas: &mut dyn Canvas) {
        let Some(selection) = self.selection else {
            return;
        };

        let start = usize::min(self.cursor, selection);
        let end = usize::max(self.cursor, selection);

        for line in &self.lines {
            let top = line.top();
            let bottom = line.bottom();

            if line.glyphs.is_empty() && line.start_index >= start && line.end_index <= end {
                let rect = Rect {
                    min: Point::new(line.left(), top),
                    max: Point::new(line.right() + 2.0, bottom),
                };

                canvas.draw_rect(
                    rect,
                    CornerRadius::all(0.0),
                    &Paint::from(Color::BLUE.fade(0.5)),
                );

                continue;
            }

            let mut left = line.right();
            let mut right = line.left();

            for glyph in &line.glyphs {
                if start <= glyph.start_index && end >= glyph.start_index {
                    left = left.min(glyph.bounds.left());
                }

                if start <= glyph.end_index && end >= glyph.end_index {
                    right = right.max(glyph.bounds.right());
                }
            }

            if left >= right {
                continue;
            }

            let rect = Rect {
                min: Point::new(left, top),
                max: Point::new(right, bottom),
            };

            canvas.draw_rect(
                rect,
                CornerRadius::all(0.0),
                &Paint::from(self.selection_color.fade(0.6)),
            );
        }
    }

    fn draw_cursor(&self, canvas: &mut dyn Canvas) {
        if self.selection.is_some() {
            return;
        }

        for line in &self.lines {
            if self.cursor < line.start_index || self.cursor > line.end_index {
                continue;
            }

            let blink = self.blink.cos().abs();

            for glyph in &line.glyphs {
                if self.cursor <= glyph.start_index {
                    let rect = Rect {
                        min: Point::new(glyph.bounds.left(), line.top()),
                        max: Point::new(glyph.bounds.left() + 1.0, line.bottom()),
                    };

                    canvas.draw_rect(
                        rect,
                        CornerRadius::all(0.0),
                        &Paint::from(self.cursor_color.fade(blink)),
                    );

                    return;
                }
            }

            let rect = Rect {
                min: Point::new(line.right(), line.top()),
                max: Point::new(line.right() + 1.0, line.bottom()),
            };

            canvas.draw_rect(
                rect,
                CornerRadius::all(0.0),
                &Paint::from(self.cursor_color.fade(blink)),
            );
        }
    }
}

impl Widget for TextArea {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        self.lines = cx.fonts().layout(&self.paragraph, space.max.width);

        cx.fonts()
            .measure(&self.paragraph, space.max.width)
            .min(space.max)
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_text(&self.paragraph, cx.width(), Offset::all(0.0));

        if !cx.is_focused() {
            return;
        }

        self.draw_selection(canvas);
        self.draw_cursor(canvas);
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        if let Update::Focused(focused) = update {
            cx.request_draw();

            match focused {
                true => cx.request_animate(),
                false => self.cursor = self.paragraph.text.len(),
            }
        }
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        if cx.is_focused() && self.selection.is_none() {
            self.blink += dt.as_secs_f32() * 5.0;

            cx.request_animate();
            cx.request_draw();
        }
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        match event {
            PointerEvent::Down(event) => {
                let local = cx.global_transform().inverse() * event.position;
                let cursor = self.find_point(local);
                self.set_cursor(cursor, false);

                cx.request_draw();
                cx.request_focus();
                PointerPropagate::Capture
            }

            PointerEvent::Move(event) if cx.is_active() => {
                let local = cx.global_transform().inverse() * event.position;
                let cursor = self.find_point(local);
                self.set_cursor(cursor, true);

                cx.request_draw();
                PointerPropagate::Bubble
            }

            _ => PointerPropagate::Bubble,
        }
    }

    fn on_key_event(&mut self, cx: &mut EventCx<'_>, event: &KeyEvent) -> Propagate {
        match event {
            KeyEvent::Down(event) => {
                if let Some(ref text) = event.text
                    && !text.chars().any(|c| c.is_ascii_control())
                {
                    self.insert_text(text);
                    cx.request_layout();

                    return Propagate::Stop;
                }

                if event.key == Key::Named(NamedKey::ArrowRight) {
                    self.move_forward(event.modifiers.shift());
                    cx.request_layout();

                    return Propagate::Stop;
                }

                if event.key == Key::Named(NamedKey::ArrowLeft) {
                    self.move_backward(event.modifiers.shift());
                    cx.request_layout();

                    return Propagate::Stop;
                }

                if event.key == Key::Named(NamedKey::Backspace) {
                    if self.selection.is_some() {
                        self.remove_selection();
                        cx.request_layout();
                    } else if self.cursor > 0 {
                        self.move_backward(false);
                        self.paragraph.text.remove(self.cursor);
                        cx.request_layout();
                    }

                    return Propagate::Stop;
                }

                if event.key == Key::Named(NamedKey::Enter) {
                    self.insert_text("\n");
                    cx.request_layout();

                    return Propagate::Stop;
                }

                Propagate::Bubble
            }

            _ => Propagate::Bubble,
        }
    }

    fn accepts_focus() -> bool {
        true
    }
}
