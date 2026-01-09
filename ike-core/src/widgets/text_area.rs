use std::time::Duration;

use keyboard_types::NamedKey;

use crate::{
    Affine, AnyWidgetId, Builder, Canvas, Color, ComposeCx, CornerRadius, CursorIcon, DrawCx,
    EventCx, Gesture, ImeEvent, Key, KeyEvent, LayoutCx, MutCx, Offset, Paint, Paragraph, Point,
    PointerEvent, PointerPropagate, Propagate, Rect, Size, Space, TextLayoutLine, TouchEvent,
    TouchPropagate, Update, UpdateCx, Widget, WidgetId, WidgetMut, event::TextEvent,
    window::LayerId,
};

/// When should newlines be inserted in a [`TextArea`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NewlineBehaviour {
    /// Insert a newline when [`NamedKey::Enter`] is pressed.
    ///
    /// When enabled, `on_submit` is never called.
    Enter,

    /// Insert a newline when [`NamedKey::Enter`] is pressed and
    /// [`Modifiers::shift`](crate::Modifiers) is held.
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
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SubmitBehaviour {
    pub keep_focus: bool,
    pub clear_text: bool,
}

pub struct TextArea<const EDITABLE: bool> {
    paragraph:         Paragraph,
    selection_color:   Color,
    cursor_color:      Color,
    handle_color:      Color,
    blink_rate:        f32,
    handle_size:       f32,
    handles_enabled:   bool,
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
    handle:        Option<(LayerId, WidgetId<Handle<EDITABLE>>)>,
}

impl<const EDITABLE: bool> TextArea<EDITABLE> {
    pub fn new(cx: &mut impl Builder, paragraph: Paragraph) -> WidgetMut<'_, Self> {
        let cursor = paragraph.text.len();

        cx.build_widget(Self {
            paragraph,
            selection_color: Color::BLUE,
            cursor_color: Color::BLACK,
            handle_color: Color::GREEN,
            blink_rate: 5.0,
            handle_size: 20.0,
            handles_enabled: cfg!(target_os = "android"),
            newline_behaviour: NewlineBehaviour::Enter,
            submit_behaviour: SubmitBehaviour::default(),

            on_change: None,
            on_submit: None,

            lines: Vec::new(),
            cursor,
            selection: None,
            blink: 0.0,
            cursor_anchor: None,
            handle: None,
        })
        .finish()
    }

    pub fn set_text(this: &mut WidgetMut<Self>, paragraph: Paragraph) {
        // the new text might place the cursor in the middle of a unicode character,
        // in which case we want to move the cursor to avoid crashes.

        this.widget.cursor = this.widget.cursor.min(paragraph.text.len());
        while !paragraph.text.is_char_boundary(this.widget.cursor) {
            this.widget.cursor -= 1;
        }

        if let Some(ref mut selection) = this.widget.selection {
            *selection = (*selection).min(paragraph.text.len());
            while !paragraph.text.is_char_boundary(*selection) {
                *selection -= 1;
            }
        }

        if this.cx.is_focused() {
            if this.widget.text() != paragraph.text {
                this.cx.restart_ime();
            }

            this.cx.set_ime_text(paragraph.text.clone());

            this.widget.set_selection_mut(&mut this.cx);
        }

        this.widget.paragraph = paragraph;

        this.cx.request_layout();
        this.cx.request_draw();
    }

    pub fn set_selection_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.selection_color = color;
        this.cx.request_draw();
    }

    pub fn set_cursor_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.cursor_color = color;
        this.cx.request_draw();
    }

    pub fn set_handle_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.handle_color = color;

        if let Some((_layer, handle)) = this.widget.handle
            && let Some(mut handle) = this.cx.get_widget_mut(handle)
        {
            handle.widget.color = color;
            handle.cx.request_draw();
        }
    }

    pub fn set_handle_size(this: &mut WidgetMut<Self>, size: f32) {
        this.widget.handle_size = size;
        this.cx.request_compose();

        if let Some((_layer, handle)) = this.widget.handle
            && let Some(mut handle) = this.cx.get_widget_mut(handle)
        {
            handle.widget.size = size;
            handle.cx.request_layout();
        }
    }

    pub fn set_handles_enabled(this: &mut WidgetMut<Self>, enabled: bool) {
        this.widget.handles_enabled = enabled;

        if !enabled
            && let Some(window) = this.cx.window()
            && let Some((layer, _handle)) = this.widget.handle.take()
        {
            this.cx.defer(move |world| {
                world.remove_layer(window, layer);
            });
        }
    }

    pub fn set_blink_rate(this: &mut WidgetMut<Self>, rate: f32) {
        this.widget.blink_rate = rate;
    }

    pub fn set_newline_behaviour(this: &mut WidgetMut<Self>, behaviour: NewlineBehaviour) {
        this.widget.newline_behaviour = behaviour;
        this.cx.request_draw();
    }

    pub fn set_submit_behaviour(this: &mut WidgetMut<Self>, behaviour: SubmitBehaviour) {
        this.widget.submit_behaviour = behaviour;
        this.cx.request_draw();
    }

    pub fn set_on_change(this: &mut WidgetMut<Self>, on_change: impl FnMut(&str) + 'static) {
        this.widget.on_change = Some(Box::new(on_change));
    }

    pub fn set_on_submit(this: &mut WidgetMut<Self>, on_submit: impl FnMut(&str) + 'static) {
        this.widget.on_submit = Some(Box::new(on_submit));
    }

    pub fn text(&self) -> &str {
        &self.paragraph.text
    }
}

impl<const EDITABLE: bool> TextArea<EDITABLE> {
    fn set_cursor(&mut self, cursor: usize, select: bool) {
        if !select {
            self.selection = None
        } else if self.selection.is_none() {
            self.selection = Some(self.cursor);
        }

        if self.selection == Some(cursor) {
            self.selection = None;
        }

        self.cursor = cursor;
        self.blink = 0.0;
    }

    fn find_point(&self, mut point: Point, constrain: bool) -> usize {
        if let Some(first) = self.lines.first()
            && let Some(last) = self.lines.last()
            && constrain
        {
            point.y = point.y.clamp(first.top(), last.bottom());
        }

        if let Some(first) = self.lines.first()
            && point.y < first.top()
        {
            return 0;
        }

        if let Some(last) = self.lines.last()
            && point.y > last.bottom()
        {
            return self.paragraph.text.len();
        }

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

    fn get_selection(&self) -> Option<&str> {
        match self.selection {
            Some(selection) => {
                let start = usize::min(self.cursor, selection);
                let end = usize::max(self.cursor, selection);

                Some(&self.text()[start..end])
            }

            None => None,
        }
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
            self.set_cursor(
                self.cursor + next_char.len_utf8(),
                select,
            );
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
            self.set_cursor(
                self.cursor - prev_char.len_utf8(),
                select,
            );
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

    fn current_line(&self) -> Option<&TextLayoutLine> {
        self.lines.get(self.current_line_index()?)
    }

    fn cursor_offset_in_line(cursor: usize, line: &TextLayoutLine) -> f32 {
        for glyph in &line.glyphs {
            if cursor >= glyph.start_index && cursor < glyph.end_index {
                // TODO: handle ligatures better
                let t = (cursor as f32 - glyph.start_index as f32)
                    / (glyph.end_index as f32 - glyph.start_index as f32);

                return glyph.bounds.left() * (1.0 - t) + glyph.bounds.right() * t;
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

        if let Some(line) = self.current_line() {
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

        let font_size = self
            .paragraph
            .sections()
            .next()
            .map_or(16.0, |(_, s)| s.font_size);

        // there are no layout lines yet we still want to draw the cursor at the start
        let rect = Rect {
            min: Point::new(cx.rect().left(), cx.rect().top()),
            max: Point::new(
                cx.rect().left() + 1.0,
                cx.rect().top() + font_size * 1.15,
            ),
        };

        canvas.draw_rect(
            rect,
            CornerRadius::all(0.0),
            &Paint::from(self.cursor_color.fade(blink)),
        );
    }

    fn set_selection_event(&self, cx: &mut EventCx<'_>) {
        if !cx.is_focused() || !EDITABLE {
            return;
        }

        let (start, end) = match self.selection {
            Some(selection) => (
                self.cursor.min(selection),
                self.cursor.max(selection),
            ),
            None => (self.cursor, self.cursor),
        };

        cx.set_ime_selection(start..end, None);
    }

    fn set_selection_mut(&self, cx: &mut MutCx<'_>) {
        if !cx.is_focused() || !EDITABLE {
            return;
        }

        let (start, end) = match self.selection {
            Some(selection) => (
                self.cursor.min(selection),
                self.cursor.max(selection),
            ),
            None => (self.cursor, self.cursor),
        };

        cx.set_ime_selection(start..end, None);
    }

    fn text_changed(&mut self, cx: &mut EventCx<'_>) {
        cx.set_ime_text(self.text().to_owned());

        if let Some(ref mut on_change) = self.on_change {
            on_change(&self.paragraph.text);
        }

        self.remove_handle_event(cx);
    }

    fn create_handle(&mut self, cx: &mut EventCx<'_>) {
        if let Some(window) = cx.window()
            && self.handles_enabled
            && self.handle.is_none()
        {
            let (position, offset) = self.handle_position(cx.rect(), self.cursor);
            let position = cx.global_transform() * position;

            let size = self.handle_size;
            let color = self.handle_color;
            let this = WidgetId::<Self>::downcast_unchecked(cx.id());

            cx.defer(move |world| {
                let handle = world
                    .build_widget(Handle {
                        size,
                        offset,
                        color,
                        text_area: this,
                    })
                    .finish()
                    .id();

                let layer = world.add_layer(window, position, handle);

                if let Some(text_area) = world.get_widget_mut(this) {
                    text_area.widget.handle = Some((layer, handle));
                }
            });
        }
    }

    fn remove_handle_event(&mut self, cx: &mut EventCx<'_>) {
        if let Some(window) = cx.get_window()
            && let Some((layer, _handle)) = self.handle.take()
        {
            let window = window.id();

            cx.defer(move |world| {
                world.remove_layer(window, layer);
            });
        }
    }

    fn remove_handle_update(&mut self, cx: &mut UpdateCx<'_>) {
        if let Some(window) = cx.get_window()
            && let Some((layer, _handle)) = self.handle.take()
        {
            let window = window.id();

            cx.defer(move |world| {
                world.remove_layer(window, layer);
            });
        }
    }

    fn handle_position(&mut self, rect: Rect, offset: usize) -> (Point, f32) {
        if let Some(line) = self
            .lines
            .iter()
            .find(|l| offset >= l.start_index && offset <= l.end_index)
        {
            let offset = Self::cursor_offset_in_line(self.cursor, line);
            let position = Point::new(
                offset - self.handle_size / 2.0,
                line.bottom(),
            );

            (
                position,
                position.y - line.rect().center().y + self.handle_size / 2.0,
            )
        } else {
            let position = Point::new(
                rect.left() - self.handle_size / 2.0,
                rect.bottom(),
            );

            (
                position,
                position.y - rect.center().y + self.handle_size / 2.0,
            )
        }
    }
}

impl<const EDITABLE: bool> Widget for TextArea<EDITABLE> {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        self.lines = cx.layout_text(&self.paragraph, space.max.width);
        let size = cx.measure_text(&self.paragraph, space.max.width);
        space.constrain(size)
    }

    fn compose(&mut self, cx: &mut ComposeCx<'_>) {
        if let Some(window) = cx.window()
            && let Some((layer, _handle)) = self.handle
        {
            let (position, _offset) = self.handle_position(cx.rect(), self.cursor);
            let position = cx.global_transform() * position;

            cx.defer(move |world| {
                world.set_layer_position(window, layer, position);
            });
        }
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_text(
            &self.paragraph,
            cx.width(),
            Offset::all(0.0),
        );

        if !cx.is_focused() {
            return;
        }

        self.draw_selection(canvas);

        if cx.is_window_focused() && EDITABLE {
            self.draw_cursor(cx, canvas);
        }
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        match update {
            Update::Focused(..) | Update::WindowFocused(..) => {
                // remove the handle
                if let Update::Focused(false) = update {
                    self.remove_handle_update(cx);
                }

                cx.request_draw();

                if cx.is_focused() {
                    cx.request_animate()
                }
            }

            Update::Hovered(true) => {
                cx.set_cursor(CursorIcon::Text);
            }

            Update::Removed => {
                self.remove_handle_update(cx);
            }

            _ => {}
        }
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        cx.request_draw();

        if cx.is_focused() && self.selection.is_none() && cx.is_window_focused() {
            self.blink += dt.as_secs_f32() * self.blink_rate;

            cx.request_animate();
        }
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        match event {
            PointerEvent::Down(event) => {
                let local = cx.global_transform().inverse() * event.position;
                let cursor = self.find_point(local, false);

                self.set_cursor(cursor, false);
                self.set_selection_event(cx);

                cx.request_compose();
                cx.request_draw();
                cx.request_focus();
                cx.request_animate();

                PointerPropagate::Capture
            }

            PointerEvent::Move(event) if cx.is_active() => {
                let local = cx.global_transform().inverse() * event.position;
                let cursor = self.find_point(local, false);

                self.set_cursor(cursor, true);
                self.set_selection_event(cx);

                cx.request_compose();
                cx.request_draw();
                cx.request_animate();

                PointerPropagate::Bubble
            }

            _ => PointerPropagate::Bubble,
        }
    }

    fn on_touch_event(&mut self, cx: &mut EventCx<'_>, event: &TouchEvent) -> TouchPropagate {
        match event {
            TouchEvent::Gesture(Gesture::Tap(event)) => {
                let local = cx.global_transform().inverse() * event.position;
                let cursor = self.find_point(local, true);

                self.set_cursor(cursor, false);
                self.set_selection_event(cx);

                cx.request_compose();
                cx.request_draw();
                cx.request_focus();
                cx.request_animate();

                if cx.is_focused() {
                    self.create_handle(cx);
                }

                TouchPropagate::Handled
            }

            _ => TouchPropagate::Bubble,
        }
    }

    fn on_key_event(&mut self, cx: &mut EventCx<'_>, event: &KeyEvent) -> Propagate {
        match event {
            KeyEvent::Down(event) => {
                let action_mod = match cfg!(target_os = "macos") {
                    true => event.modifiers.meta(),
                    false => event.modifiers.ctrl(),
                };

                match event.key {
                    Key::Character(ref c) if c == "c" && action_mod => {
                        if let Some(selection) = self.get_selection() {
                            cx.set_clipboard(selection.to_owned());
                        }

                        Propagate::Handled
                    }

                    Key::Character(ref c) if c == "x" && action_mod => {
                        if let Some(selection) = self.get_selection() {
                            cx.set_clipboard(selection.to_owned());
                        }

                        self.remove_selection();

                        self.text_changed(cx);
                        self.set_selection_event(cx);

                        cx.request_compose();
                        cx.request_layout();

                        Propagate::Handled
                    }

                    _ if matches!(event.text, Some(ref text) if !text.chars().any(|c| c.is_ascii_control()))
                        && EDITABLE =>
                    {
                        let text = event.text.as_ref().expect("checked by condition");
                        self.insert_text(text);

                        self.text_changed(cx);
                        self.set_selection_event(cx);

                        cx.request_compose();
                        cx.request_layout();

                        Propagate::Handled
                    }

                    Key::Named(NamedKey::ArrowRight) => {
                        self.move_forward(event.modifiers.shift());
                        self.set_selection_event(cx);

                        cx.request_compose();
                        cx.request_draw();
                        cx.request_animate();

                        Propagate::Handled
                    }

                    Key::Named(NamedKey::ArrowLeft) => {
                        self.move_backward(event.modifiers.shift());
                        self.set_selection_event(cx);

                        cx.request_compose();
                        cx.request_draw();
                        cx.request_animate();

                        Propagate::Handled
                    }

                    Key::Named(NamedKey::ArrowUp) => {
                        self.move_upward(event.modifiers.shift());
                        self.set_selection_event(cx);

                        cx.request_compose();
                        cx.request_draw();
                        cx.request_animate();

                        Propagate::Handled
                    }

                    Key::Named(NamedKey::ArrowDown) => {
                        self.move_downward(event.modifiers.shift());
                        self.set_selection_event(cx);

                        cx.request_compose();
                        cx.request_draw();
                        cx.request_animate();

                        Propagate::Handled
                    }

                    Key::Named(NamedKey::Delete) if EDITABLE => {
                        if self.selection.is_some() {
                            self.remove_selection();

                            self.text_changed(cx);
                            self.set_selection_event(cx);

                            cx.request_compose();
                            cx.request_layout();
                            cx.request_animate();
                        } else if self.cursor < self.paragraph.text.len() {
                            self.paragraph.text.remove(self.cursor);

                            self.text_changed(cx);
                            self.set_selection_event(cx);

                            cx.request_compose();
                            cx.request_layout();
                            cx.request_animate();
                        }

                        Propagate::Handled
                    }

                    Key::Named(NamedKey::Backspace) if EDITABLE => {
                        if self.selection.is_some() {
                            self.remove_selection();

                            self.text_changed(cx);
                            self.set_selection_event(cx);

                            cx.request_compose();
                            cx.request_layout();
                            cx.request_animate();
                        } else if self.cursor > 0 {
                            self.move_backward(false);
                            self.paragraph.text.remove(self.cursor);

                            self.text_changed(cx);
                            self.set_selection_event(cx);

                            cx.request_compose();
                            cx.request_layout();
                            cx.request_animate();
                        }

                        Propagate::Handled
                    }

                    Key::Named(NamedKey::Enter)
                        if self
                            .newline_behaviour
                            .insert_newline(event.modifiers.shift())
                            && EDITABLE =>
                    {
                        self.insert_text("\n");

                        self.text_changed(cx);
                        self.set_selection_event(cx);

                        cx.request_compose();
                        cx.request_layout();
                        cx.request_animate();

                        Propagate::Handled
                    }

                    Key::Named(NamedKey::Enter)
                        if !self
                            .newline_behaviour
                            .insert_newline(event.modifiers.shift())
                            && EDITABLE =>
                    {
                        if let Some(ref mut on_submit) = self.on_submit {
                            on_submit(&self.paragraph.text);
                        }

                        if !self.submit_behaviour.keep_focus {
                            cx.request_focus_next();
                        }

                        if self.submit_behaviour.clear_text {
                            self.paragraph.text.clear();
                            self.set_cursor(0, false);

                            cx.request_compose();
                            cx.request_layout();
                        }

                        Propagate::Handled
                    }

                    _ => Propagate::Bubble,
                }
            }

            _ => Propagate::Bubble,
        }
    }

    fn on_text_event(&mut self, cx: &mut EventCx<'_>, event: &TextEvent) -> Propagate {
        match event {
            TextEvent::Paste(event) if EDITABLE => {
                self.insert_text(&event.contents);

                self.text_changed(cx);
                self.set_selection_event(cx);

                cx.request_compose();
                cx.request_layout();

                Propagate::Handled
            }

            TextEvent::Ime(ime) => match ime {
                ImeEvent::Start => {
                    cx.set_ime_text(self.text().to_owned());
                    self.set_selection_event(cx);

                    Propagate::Handled
                }

                ImeEvent::Select(selection) => {
                    if selection.start == selection.end {
                        self.cursor = selection.start;
                    } else {
                        self.selection = Some(selection.start);
                        self.cursor = selection.end;

                        cx.request_compose();
                        cx.request_draw();
                    }

                    Propagate::Handled
                }

                ImeEvent::Commit(text) => {
                    self.insert_text(text);

                    self.text_changed(cx);
                    self.set_selection_event(cx);

                    cx.request_compose();
                    cx.request_layout();

                    Propagate::Handled
                }

                ImeEvent::End => Propagate::Handled,
            },

            _ => Propagate::Bubble,
        }
    }

    fn accepts_pointer() -> bool {
        true
    }

    fn accepts_text() -> bool {
        EDITABLE
    }

    fn accepts_focus() -> bool {
        EDITABLE
    }
}

struct Handle<const EDITABLE: bool> {
    size:      f32,
    offset:    f32,
    color:     Color,
    text_area: WidgetId<TextArea<EDITABLE>>,
}

impl<const EDITABLE: bool> Widget for Handle<EDITABLE> {
    fn layout(&mut self, _cx: &mut LayoutCx<'_>, _space: Space) -> Size {
        // take up double the space it make it easier to click
        Size::all(self.size * 2.0)
    }

    fn draw(&mut self, _cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.transform(
            Affine::translate(Offset::new(self.size / 2.0, 0.0))
                * Affine::rotate(std::f32::consts::PI / 4.0),
            &mut |canvas| {
                canvas.draw_rect(
                    Rect::min_size(Point::ORIGIN, Size::all(self.size)),
                    CornerRadius {
                        top_left:     self.size / 8.0,
                        top_right:    self.size / 2.0,
                        bottom_left:  self.size / 2.0,
                        bottom_right: self.size / 2.0,
                    },
                    &Paint::from(self.color),
                )
            },
        );
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        match event {
            PointerEvent::Down(..) => PointerPropagate::Capture,

            PointerEvent::Move(event) if cx.is_active() => {
                let mut position = event.position;
                position.y -= self.offset;

                let text_area = self.text_area;

                cx.defer(move |world| {
                    if let Some(mut widget) = world.get_widget_mut(text_area) {
                        let position = widget.cx.global_transform().inverse() * position;

                        let cursor = widget.widget.find_point(position, true);
                        widget.widget.set_cursor(cursor, false);

                        widget.cx.request_compose();
                        widget.cx.request_draw();
                    }
                });

                PointerPropagate::Bubble
            }

            _ => PointerPropagate::Bubble,
        }
    }

    fn on_touch_event(&mut self, cx: &mut EventCx<'_>, event: &TouchEvent) -> TouchPropagate {
        match event {
            TouchEvent::Down(..) => TouchPropagate::Capture,

            TouchEvent::Move(event) if cx.is_active() => {
                let mut position = event.position;
                position.y -= self.offset;

                let text_area = self.text_area;

                cx.defer(move |world| {
                    if let Some(mut widget) = world.get_widget_mut(text_area) {
                        let position = widget.cx.global_transform().inverse() * position;

                        let cursor = widget.widget.find_point(position, true);
                        widget.widget.set_cursor(cursor, false);
                        widget.widget.set_selection_mut(&mut widget.cx);

                        widget.cx.request_compose();
                        widget.cx.request_draw();
                    }
                });

                TouchPropagate::Bubble
            }

            TouchEvent::Up(..) if cx.is_active() => TouchPropagate::Handled,

            _ => TouchPropagate::Bubble,
        }
    }

    fn accepts_pointer() -> bool {
        true
    }
}
