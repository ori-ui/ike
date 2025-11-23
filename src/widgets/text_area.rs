use std::time::Duration;

use keyboard_types::NamedKey;

use crate::{
    BuildCx, Canvas, Color, CornerRadius, DrawCx, EventCx, Key, KeyEvent, LayoutCx, Offset, Paint,
    Paragraph, Point, PointerEvent, PointerPropagate, Propagate, Rect, Size, Space, TextLayoutLine,
    Update, UpdateCx, Widget, tree::WidgetMut,
};

/// When should newlines be inserted in a [`TextArea`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NewlineBehaviour {
    /// Insert a newline when [`NamedKey::Enter`] is pressed.
    ///
    /// When enabled, `on_submit` is never called.
    Enter,

    /// Insert a newline when [`NamedKey::Enter`] is pressed and [`Modifiers::shift`] is held.
    ShiftEnter,

    /// Never insert newlines.
    Never,
}

impl NewlineBehaviour {
    fn insert_newline(&self, shift: bool) -> bool {
        match self {
            NewlineBehaviour::Enter => true,
            NewlineBehaviour::ShiftEnter => shift,
            NewlineBehaviour::Never => false,
        }
    }
}

/// What should happen when a `submit` action happens in a [`TextArea`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SubmitBehaviour {
    /// The [`TextArea`] will keep it's focus.
    KeepFocus,

    /// The focus will be given to the next widget.
    FocusNext,
}

pub struct TextArea {
    paragraph:         Paragraph,
    selection_color:   Color,
    cursor_color:      Color,
    blink_rate:        f32,
    is_editable:       bool,
    newline_behaviour: NewlineBehaviour,
    submit_behaviour:  SubmitBehaviour,

    #[allow(clippy::type_complexity)]
    on_change: Option<Box<dyn FnMut(&str)>>,
    #[allow(clippy::type_complexity)]
    on_submit: Option<Box<dyn FnMut(&str)>>,

    lines:         Vec<TextLayoutLine>,
    cursor:        usize,
    selection:     Option<usize>,
    blink:         f32,
    cursor_anchor: Option<f32>,
}

impl TextArea {
    pub fn new(
        cx: &mut impl BuildCx,
        paragraph: Paragraph,
        is_editable: bool,
    ) -> WidgetMut<'_, Self> {
        let cursor = paragraph.text.len();

        cx.insert(Self {
            paragraph,
            selection_color: Color::BLUE,
            cursor_color: Color::BLACK,
            blink_rate: 5.0,
            is_editable,
            newline_behaviour: NewlineBehaviour::Enter,
            submit_behaviour: SubmitBehaviour::FocusNext,

            on_change: None,
            on_submit: None,

            lines: Vec::new(),
            cursor,
            selection: None,
            blink: 0.0,
            cursor_anchor: None,
        })
    }

    pub fn set_text(this: &mut WidgetMut<Self>, paragraph: Paragraph) {
        // the new text might place the cursor in the middle of a unicode character,
        // in which case we want to move the cursor to avoid crashes.

        this.cursor = this.cursor.min(paragraph.text.len());
        while !paragraph.text.is_char_boundary(this.cursor) {
            this.cursor -= 1;
        }

        if let Some(ref mut selection) = this.selection {
            *selection = (*selection).min(paragraph.text.len());
            while !paragraph.text.is_char_boundary(*selection) {
                *selection -= 1;
            }
        }

        this.paragraph = paragraph;
        this.request_layout();
    }

    pub fn set_selection_color(this: &mut WidgetMut<Self>, color: Color) {
        this.selection_color = color;
        this.request_draw();
    }

    pub fn set_cursor_color(this: &mut WidgetMut<Self>, color: Color) {
        this.cursor_color = color;
        this.request_draw();
    }

    pub fn set_blink_rate(this: &mut WidgetMut<Self>, rate: f32) {
        this.blink_rate = rate;
    }

    pub fn set_newline_behaviour(this: &mut WidgetMut<Self>, behaviour: NewlineBehaviour) {
        this.newline_behaviour = behaviour;
        this.request_draw();
    }

    pub fn set_submit_behaviour(this: &mut WidgetMut<Self>, behaviour: SubmitBehaviour) {
        this.submit_behaviour = behaviour;
        this.request_draw();
    }

    pub fn set_on_change(this: &mut WidgetMut<Self>, on_change: impl FnMut(&str) + 'static) {
        this.on_change = Some(Box::new(on_change));
    }

    pub fn set_on_submit(this: &mut WidgetMut<Self>, on_submit: impl FnMut(&str) + 'static) {
        this.on_submit = Some(Box::new(on_submit));
    }

    pub fn text(&self) -> &str {
        &self.paragraph.text
    }
}

impl TextArea {
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
            if point.y >= line.top() && point.y <= line.bottom() {
                return Self::find_point_in_line(line, point.x);
            }
        }

        self.paragraph.text.len()
    }

    fn find_point_in_line(line: &TextLayoutLine, x: f32) -> usize {
        for glyph in &line.glyphs {
            if x < glyph.bounds.center().x {
                return glyph.start_index;
            }
        }

        line.end_index
    }

    fn move_forward(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            self.move_end();
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
        if !select && self.selection.is_some() {
            self.move_start();
            return;
        }

        if self.cursor == 0 {
            return;
        }

        if let Some(prev_char) = self.paragraph.text[..self.cursor].chars().next_back() {
            self.set_cursor(self.cursor - prev_char.len_utf8(), select);
        };
    }

    fn move_start(&mut self) {
        if let Some(selection) = self.selection {
            self.set_cursor(self.cursor.min(selection), false);
        }
    }

    fn move_end(&mut self) {
        if let Some(selection) = self.selection {
            self.set_cursor(self.cursor.max(selection), false);
        }
    }

    fn move_upward(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            self.move_start();
            return;
        }

        let Some(index) = self.current_line_index() else {
            return;
        };

        let next_index = index.saturating_sub(1);

        let anchor = *self.cursor_anchor.get_or_insert_with(|| {
            let line = &self.lines[index];
            Self::cursor_offset_in_line(self.cursor, line)
        });

        let line = &self.lines[next_index];
        let cursor = Self::find_point_in_line(line, anchor);
        self.set_cursor(cursor, select);
    }

    fn move_downward(&mut self, select: bool) {
        if !select && self.selection.is_some() {
            self.move_start();
            return;
        }

        let Some(index) = self.current_line_index() else {
            return;
        };

        let next_index = usize::min(index + 1, self.lines.len() - 1);

        let anchor = *self.cursor_anchor.get_or_insert_with(|| {
            let line = &self.lines[index];
            Self::cursor_offset_in_line(self.cursor, line)
        });

        let line = &self.lines[next_index];
        let cursor = Self::find_point_in_line(line, anchor);
        self.set_cursor(cursor, select);
    }

    fn current_line_index(&self) -> Option<usize> {
        self.lines
            .iter()
            .position(|l| self.cursor >= l.start_index && self.cursor <= l.end_index)
    }

    fn cursor_offset_in_line(cursor: usize, line: &TextLayoutLine) -> f32 {
        for glyph in &line.glyphs {
            if cursor >= glyph.start_index && cursor < glyph.end_index {
                return glyph.bounds.left();
            }
        }

        line.right()
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
                    &Paint::from(self.selection_color.fade(0.5)),
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

    fn draw_cursor(&self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        if self.selection.is_some() {
            return;
        }

        let blink = self.blink.cos().abs();

        if let Some(index) = self.current_line_index() {
            let line = &self.lines[index];
            let offset = Self::cursor_offset_in_line(self.cursor, line);

            let rect = Rect {
                min: Point::new(offset, line.top()),
                max: Point::new(offset + 1.0, line.bottom()),
            };

            canvas.draw_rect(
                rect,
                CornerRadius::all(0.0),
                &Paint::from(self.cursor_color.fade(blink)),
            );

            return;
        }

        // there are no layout lines yet we still want to draw the cursor at the start
        let rect = Rect {
            min: Point::new(cx.rect().left(), cx.rect().top()),
            max: Point::new(cx.rect().left() + 1.0, cx.rect().bottom()),
        };

        canvas.draw_rect(
            rect,
            CornerRadius::all(0.0),
            &Paint::from(self.cursor_color.fade(blink)),
        );
    }

    fn changed(&mut self) {
        if let Some(ref mut on_change) = self.on_change {
            on_change(&self.paragraph.text);
        }
    }
}

impl Widget for TextArea {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        self.lines = cx.fonts().layout(&self.paragraph, space.max.width);
        let size = cx.fonts().measure(&self.paragraph, space.max.width);

        size.fit(space)
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_text(&self.paragraph, cx.width(), Offset::all(0.0));

        if !cx.is_focused() {
            return;
        }

        self.draw_selection(canvas);

        if cx.is_window_focused() && self.is_editable {
            self.draw_cursor(cx, canvas);
        }
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        match update {
            Update::Focused(..) | Update::WindowFocused(..) => {
                cx.request_draw();

                if cx.is_focused() {
                    cx.request_animate()
                }
            }

            _ => (),
        }
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        if cx.is_focused() && self.selection.is_none() {
            self.blink += dt.as_secs_f32() * self.blink_rate;

            cx.request_draw();
            cx.request_animate();
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
                cx.request_animate();
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
            KeyEvent::Down(event) if self.is_editable => {
                if let Some(ref text) = event.text
                    && !text.chars().any(|c| c.is_ascii_control())
                {
                    self.insert_text(text);

                    self.changed();
                    cx.request_layout();

                    return Propagate::Stop;
                }

                match event.key {
                    Key::Named(NamedKey::ArrowRight) => {
                        self.move_forward(event.modifiers.shift());
                        cx.request_draw();
                        cx.request_animate();

                        Propagate::Stop
                    }

                    Key::Named(NamedKey::ArrowLeft) => {
                        self.move_backward(event.modifiers.shift());
                        cx.request_draw();
                        cx.request_animate();

                        Propagate::Stop
                    }

                    Key::Named(NamedKey::ArrowUp) => {
                        self.move_upward(event.modifiers.shift());
                        cx.request_draw();
                        cx.request_animate();

                        Propagate::Stop
                    }

                    Key::Named(NamedKey::ArrowDown) => {
                        self.move_downward(event.modifiers.shift());
                        cx.request_draw();
                        cx.request_animate();

                        Propagate::Stop
                    }

                    Key::Named(NamedKey::Delete) => {
                        if self.selection.is_some() {
                            self.remove_selection();

                            self.changed();
                            cx.request_layout();
                            cx.request_animate();
                        } else if self.cursor <= self.paragraph.text.len() {
                            self.paragraph.text.remove(self.cursor);

                            self.changed();
                            cx.request_layout();
                            cx.request_animate();
                        }

                        Propagate::Stop
                    }

                    Key::Named(NamedKey::Backspace) => {
                        if self.selection.is_some() {
                            self.remove_selection();

                            self.changed();
                            cx.request_layout();
                            cx.request_animate();
                        } else if self.cursor > 0 {
                            self.move_backward(false);
                            self.paragraph.text.remove(self.cursor);

                            self.changed();
                            cx.request_layout();
                            cx.request_animate();
                        }

                        Propagate::Stop
                    }

                    Key::Named(NamedKey::Enter)
                        if self
                            .newline_behaviour
                            .insert_newline(event.modifiers.shift()) =>
                    {
                        self.insert_text("\n");

                        self.changed();
                        cx.request_layout();
                        cx.request_animate();

                        Propagate::Stop
                    }

                    Key::Named(NamedKey::Enter)
                        if !self
                            .newline_behaviour
                            .insert_newline(event.modifiers.shift()) =>
                    {
                        if let Some(ref mut on_submit) = self.on_submit {
                            on_submit(&self.paragraph.text);
                            cx.request_draw();
                        }

                        if let SubmitBehaviour::FocusNext = self.submit_behaviour {
                            cx.request_focus_next();
                        }

                        Propagate::Stop
                    }

                    _ => Propagate::Bubble,
                }
            }

            _ => Propagate::Bubble,
        }
    }

    fn accepts_focus() -> bool {
        true
    }
}
