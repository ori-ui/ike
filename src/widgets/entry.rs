use crate::{
    BorderWidth, BuildCx, Canvas, Color, CornerRadius, DrawCx, LayoutCx, Padding, Paint, Painter,
    Paragraph, Size, Space, TextAlign, TextWrap, Widget, WidgetId,
    tree::{WidgetMut, WidgetRef},
    widgets::{Label, NewlineBehaviour, SubmitBehaviour, TextArea},
};

pub struct Entry {
    text_area:   WidgetId<TextArea>,
    placeholder: WidgetId<Label>,

    min_width:        f32,
    max_width:        f32,
    padding:          Padding,
    border_width:     BorderWidth,
    corner_radius:    CornerRadius,
    background_color: Color,
    border_color:     Color,
    focus_color:      Color,
}

impl Entry {
    pub fn new(cx: &mut impl BuildCx, paragraph: Paragraph) -> WidgetMut<'_, Self> {
        let text_area = TextArea::new(cx, paragraph, true).id();
        let placeholder = Paragraph::new(16.0, TextAlign::Start, TextWrap::None);
        let placeholder = Label::new(cx, placeholder).id();

        let mut this = cx.insert(Entry {
            text_area,
            placeholder,

            min_width: 100.0,
            max_width: f32::INFINITY,
            padding: Padding::all(8.0),
            border_width: BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(8.0),
            background_color: Color::WHITE,
            border_color: Color::BLACK,
            focus_color: Color::BLUE,
        });
        this.add_child(text_area);
        this.add_child(placeholder);
        this
    }

    pub fn set_placeholder(this: &mut WidgetMut<Self>, paragraph: Paragraph) {
        Label::set_text(&mut this.get_mut(this.placeholder).unwrap(), paragraph)
    }

    pub fn set_min_width(this: &mut WidgetMut<Self>, min_width: f32) {
        this.min_width = min_width;
        this.request_layout();
    }

    pub fn set_max_width(this: &mut WidgetMut<Self>, max_width: f32) {
        this.max_width = max_width;
        this.request_layout();
    }

    pub fn set_padding(this: &mut WidgetMut<Self>, padding: Padding) {
        this.padding = padding;
        this.request_layout();
    }

    pub fn set_border_width(this: &mut WidgetMut<Self>, border_width: BorderWidth) {
        this.border_width = border_width;
        this.request_layout();
    }

    pub fn set_corner_radius(this: &mut WidgetMut<Self>, corner_radius: CornerRadius) {
        this.corner_radius = corner_radius;
        this.request_draw();
    }

    pub fn set_background_color(this: &mut WidgetMut<Self>, color: Color) {
        this.background_color = color;
        this.request_draw();
    }

    pub fn set_border_color(this: &mut WidgetMut<Self>, color: Color) {
        this.border_color = color;
        this.request_draw();
    }

    pub fn set_focus_color(this: &mut WidgetMut<Self>, color: Color) {
        this.focus_color = color;
        this.request_draw();
    }

    pub fn set_text(this: &mut WidgetMut<Self>, paragraph: Paragraph) {
        TextArea::set_text(&mut this.get_mut(this.text_area).unwrap(), paragraph);
    }

    pub fn set_selection_color(this: &mut WidgetMut<Self>, color: Color) {
        TextArea::set_selection_color(&mut this.get_mut(this.text_area).unwrap(), color);
    }

    pub fn set_cursor_color(this: &mut WidgetMut<Self>, color: Color) {
        TextArea::set_cursor_color(&mut this.get_mut(this.text_area).unwrap(), color);
    }

    pub fn set_blink_rate(this: &mut WidgetMut<Self>, rate: f32) {
        TextArea::set_blink_rate(&mut this.get_mut(this.text_area).unwrap(), rate);
    }

    pub fn set_newline_behaviour(this: &mut WidgetMut<Self>, behaviour: NewlineBehaviour) {
        TextArea::set_newline_behaviour(&mut this.get_mut(this.text_area).unwrap(), behaviour);
    }

    pub fn set_submit_behaviour(this: &mut WidgetMut<Self>, behaviour: SubmitBehaviour) {
        TextArea::set_submit_behaviour(&mut this.get_mut(this.text_area).unwrap(), behaviour);
    }

    pub fn set_on_change(this: &mut WidgetMut<Self>, on_change: impl FnMut(&str) + 'static) {
        TextArea::set_on_change(&mut this.get_mut(this.text_area).unwrap(), on_change);
    }

    pub fn set_on_submit(this: &mut WidgetMut<Self>, on_submit: impl FnMut(&str) + 'static) {
        TextArea::set_on_submit(&mut this.get_mut(this.text_area).unwrap(), on_submit);
    }

    pub fn text_area<'a>(this: &'a WidgetRef<'_, Self>) -> WidgetRef<'a, TextArea> {
        this.get(this.text_area).unwrap()
    }

    pub fn text_area_mut<'a>(this: &'a mut WidgetMut<'a, Self>) -> WidgetMut<'a, TextArea> {
        this.get_mut(this.text_area).unwrap()
    }
}

impl Widget for Entry {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size {
        let mut space = space.shrink(self.padding.size() + self.border_width.size());
        space.min.width = space.min.width.max(self.min_width);
        space.max.width = space.max.width.max(self.max_width);
        space.min.width = space.min.width.min(space.max.width);

        cx.set_child_stashed(1, !cx.get(self.text_area).text().is_empty());

        let placeholder_size = cx.layout_child(1, painter, space);
        space.min.width = space.min.width.max(placeholder_size.width);
        let text_size = cx.layout_child(0, painter, space);

        cx.place_child(0, self.padding.offset() + self.border_width.offset());
        cx.place_child(1, self.padding.offset() + self.border_width.offset());

        text_size.max(placeholder_size) + self.padding.size() + self.border_width.size()
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_rect(
            cx.rect(),
            self.corner_radius,
            &Paint::from(self.background_color),
        );
        canvas.draw_border(
            cx.rect(),
            self.border_width,
            self.corner_radius,
            &Paint::from(self.border_color),
        );
    }

    fn draw_over(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        if cx.has_focused() && cx.is_window_focused() {
            canvas.draw_border(
                cx.rect().expand(4.0),
                BorderWidth::all(2.0),
                self.corner_radius + 4.0,
                &Paint::from(self.focus_color),
            );
        }
    }
}
