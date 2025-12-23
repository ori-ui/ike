use crate::{
    BorderWidth, BuildCx, Canvas, Color, CornerRadius, DrawCx, LayoutCx, Padding, Paint, Painter,
    Paragraph, Size, Space, TextAlign, TextWrap, Widget, WidgetId,
    arena::{WidgetMut, WidgetRef},
    widgets::{Label, NewlineBehaviour, SubmitBehaviour, TextArea},
};

pub struct Entry {
    text_area:   WidgetId<TextArea<true>>,
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
        let text_area = TextArea::new(cx, paragraph).id();
        let placeholder = Paragraph::new(16.0, TextAlign::Start, TextWrap::None);
        let placeholder = Label::new(cx, placeholder).id();

        let mut this = cx.insert_widget(Entry {
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
        if let Some(mut placeholder) = this.cx.get_mut(this.widget.placeholder) {
            Label::set_text(&mut placeholder, paragraph);
        }
    }

    pub fn set_min_width(this: &mut WidgetMut<Self>, min_width: f32) {
        this.widget.min_width = min_width;
        this.cx.request_layout();
    }

    pub fn set_max_width(this: &mut WidgetMut<Self>, max_width: f32) {
        this.widget.max_width = max_width;
        this.cx.request_layout();
    }

    pub fn set_padding(this: &mut WidgetMut<Self>, padding: Padding) {
        this.widget.padding = padding;
        this.cx.request_layout();
    }

    pub fn set_border_width(this: &mut WidgetMut<Self>, border_width: BorderWidth) {
        this.widget.border_width = border_width;
        this.cx.request_layout();
    }

    pub fn set_corner_radius(this: &mut WidgetMut<Self>, corner_radius: CornerRadius) {
        this.widget.corner_radius = corner_radius;
        this.cx.request_draw();
    }

    pub fn set_background_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.background_color = color;
        this.cx.request_draw();
    }

    pub fn set_border_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.border_color = color;
        this.cx.request_draw();
    }

    pub fn set_focus_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.focus_color = color;
        this.cx.request_draw();
    }

    pub fn set_text(this: &mut WidgetMut<Self>, paragraph: Paragraph) {
        if let Some(mut text_area) = this.cx.get_mut(this.widget.text_area) {
            TextArea::set_text(&mut text_area, paragraph);
        }
    }

    pub fn set_selection_color(this: &mut WidgetMut<Self>, color: Color) {
        if let Some(mut text_area) = this.cx.get_mut(this.widget.text_area) {
            TextArea::set_selection_color(&mut text_area, color);
        }
    }

    pub fn set_cursor_color(this: &mut WidgetMut<Self>, color: Color) {
        if let Some(mut text_area) = this.cx.get_mut(this.widget.text_area) {
            TextArea::set_cursor_color(&mut text_area, color);
        }
    }

    pub fn set_blink_rate(this: &mut WidgetMut<Self>, rate: f32) {
        if let Some(mut text_area) = this.cx.get_mut(this.widget.text_area) {
            TextArea::set_blink_rate(&mut text_area, rate);
        }
    }

    pub fn set_newline_behaviour(this: &mut WidgetMut<Self>, behaviour: NewlineBehaviour) {
        if let Some(mut text_area) = this.cx.get_mut(this.widget.text_area) {
            TextArea::set_newline_behaviour(&mut text_area, behaviour);
        }
    }

    pub fn set_submit_behaviour(this: &mut WidgetMut<Self>, behaviour: SubmitBehaviour) {
        if let Some(mut text_area) = this.cx.get_mut(this.widget.text_area) {
            TextArea::set_submit_behaviour(&mut text_area, behaviour);
        }
    }

    pub fn set_on_change(this: &mut WidgetMut<Self>, on_change: impl FnMut(&str) + 'static) {
        if let Some(mut text_area) = this.cx.get_mut(this.widget.text_area) {
            TextArea::set_on_change(&mut text_area, on_change);
        }
    }

    pub fn set_on_submit(this: &mut WidgetMut<Self>, on_submit: impl FnMut(&str) + 'static) {
        if let Some(mut text_area) = this.cx.get_mut(this.widget.text_area) {
            TextArea::set_on_submit(&mut text_area, on_submit);
        }
    }

    pub fn get_text_area<'a>(
        this: &'a WidgetRef<'_, Self>,
    ) -> Option<WidgetRef<'a, TextArea<true>>> {
        this.cx.get(this.widget.text_area)
    }

    pub fn get_text_area_mut<'a>(
        this: &'a mut WidgetMut<'a, Self>,
    ) -> Option<WidgetMut<'a, TextArea<true>>> {
        this.cx.get_mut(this.widget.text_area)
    }
}

impl Widget for Entry {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size {
        let mut space = space.shrink(self.padding.size() + self.border_width.size());
        space.min.width = space.min.width.max(self.min_width);
        space.max.width = space.max.width.min(self.max_width);
        space.min.width = space.min.width.min(space.max.width);

        let has_text = cx
            .get(self.text_area)
            .is_some_and(|w| w.widget.text().is_empty());

        cx.set_child_stashed(1, !has_text);

        let placeholder_size = cx.layout_child(1, painter, space);
        space.min.width = space.min.width.max(placeholder_size.width);
        let text_size = cx.layout_child(0, painter, space);

        cx.place_child(
            0,
            self.padding.offset() + self.border_width.offset(),
        );
        cx.place_child(
            1,
            self.padding.offset() + self.border_width.offset(),
        );

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
                cx.rect(),
                BorderWidth::all(2.0),
                self.corner_radius,
                &Paint::from(self.focus_color),
            );
        }
    }
}
