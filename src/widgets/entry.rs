use crate::{
    BorderWidth, BuildCx, Canvas, Color, CornerRadius, DrawCx, LayoutCx, Padding, Paint, Paragraph,
    Size, Space, Widget, WidgetId,
    widgets::{NewlineBehaviour, SubmitBehaviour, TextArea},
};

pub struct Entry {
    text_area: WidgetId<TextArea>,

    min_width:     f32,
    max_width:     f32,
    padding:       Padding,
    border_width:  BorderWidth,
    corner_radius: CornerRadius,
    background:    Color,
    border_color:  Color,
    focus_color:   Color,
}

impl Entry {
    pub fn new(cx: &mut impl BuildCx, paragraph: Paragraph) -> WidgetId<Self> {
        let text_area = TextArea::new(cx, paragraph, true);

        let this = cx.insert(Entry {
            text_area,

            min_width: 100.0,
            max_width: f32::INFINITY,
            padding: Padding::all(8.0),
            border_width: BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(8.0),
            background: Color::WHITE,
            border_color: Color::BLACK,
            focus_color: Color::BLUE,
        });

        cx.add_child(this, text_area);

        this
    }

    pub fn set_min_width(cx: &mut impl BuildCx, id: WidgetId<Self>, min_width: f32) {
        cx.get_mut(id).min_width = min_width;
        cx.request_layout(id);
    }

    pub fn set_max_width(cx: &mut impl BuildCx, id: WidgetId<Self>, max_width: f32) {
        cx.get_mut(id).max_width = max_width;
        cx.request_layout(id);
    }

    pub fn set_padding(cx: &mut impl BuildCx, id: WidgetId<Self>, padding: Padding) {
        cx.get_mut(id).padding = padding;
        cx.request_layout(id);
    }

    pub fn set_border_width(cx: &mut impl BuildCx, id: WidgetId<Self>, border_width: BorderWidth) {
        cx.get_mut(id).border_width = border_width;
        cx.request_layout(id);
    }

    pub fn set_corner_radius(
        cx: &mut impl BuildCx,
        id: WidgetId<Self>,
        corner_radius: CornerRadius,
    ) {
        cx.get_mut(id).corner_radius = corner_radius;
        cx.request_draw(id);
    }

    pub fn set_background_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        cx.get_mut(id).background = color;
        cx.request_draw(id);
    }

    pub fn set_border_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        cx.get_mut(id).border_color = color;
        cx.request_draw(id);
    }

    pub fn set_focus_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        cx.get_mut(id).focus_color = color;
        cx.request_draw(id);
    }

    pub fn set_paragraph(cx: &mut impl BuildCx, id: WidgetId<Self>, paragraph: Paragraph) {
        let text_area = cx.get(id).text_area;
        TextArea::set_paragraph(cx, text_area, paragraph);
    }

    pub fn set_selection_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        let text_area = cx.get(id).text_area;
        TextArea::set_selection_color(cx, text_area, color);
    }

    pub fn set_cursor_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        let text_area = cx.get(id).text_area;
        TextArea::set_cursor_color(cx, text_area, color);
    }

    pub fn set_blink_rate(cx: &mut impl BuildCx, id: WidgetId<Self>, rate: f32) {
        let text_area = cx.get(id).text_area;
        TextArea::set_blink_rate(cx, text_area, rate);
    }

    pub fn set_newline_behaviour(
        cx: &mut impl BuildCx,
        id: WidgetId<Self>,
        behaviour: NewlineBehaviour,
    ) {
        let text_area = cx.get(id).text_area;
        TextArea::set_newline_behaviour(cx, text_area, behaviour);
    }

    pub fn set_submit_behaviour(
        cx: &mut impl BuildCx,
        id: WidgetId<Self>,
        behaviour: SubmitBehaviour,
    ) {
        let text_area = cx.get(id).text_area;
        TextArea::set_submit_behaviour(cx, text_area, behaviour);
    }

    pub fn set_on_change(
        cx: &mut impl BuildCx,
        id: WidgetId<Self>,
        on_change: impl FnMut(&str) + 'static,
    ) {
        let text_area = cx.get(id).text_area;
        TextArea::set_on_change(cx, text_area, on_change);
    }

    pub fn set_on_submit(
        cx: &mut impl BuildCx,
        id: WidgetId<Self>,
        on_submit: impl FnMut(&str) + 'static,
    ) {
        let text_area = cx.get(id).text_area;
        TextArea::set_on_submit(cx, text_area, on_submit);
    }

    pub fn get_text(cx: &impl BuildCx, id: WidgetId<Self>) -> &str {
        let text_area = cx.get(id).text_area;
        TextArea::get_text(cx, text_area)
    }
}

impl Widget for Entry {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let mut space = space.shrink(self.padding.size() + self.border_width.size());
        space.min.width = space.min.width.max(self.min_width);
        space.max.width = space.max.width.max(self.max_width);
        space.min.width = space.min.width.min(space.max.width);

        let size = cx.layout_child(0, space);

        cx.place_child(0, self.padding.offset() + self.border_width.offset());

        size + self.padding.size() + self.border_width.size()
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_rect(cx.rect(), self.corner_radius, &Paint::from(self.background));
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
