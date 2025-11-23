use crate::{
    AnyWidgetId, BorderWidth, BuildCx, Canvas, Color, CornerRadius, DrawCx, EventCx, Key, KeyEvent,
    LayoutCx, NamedKey, Padding, Paint, PointerEvent, PointerPropagate, Propagate, Size, Space,
    Widget, WidgetId, context::UpdateCx, widget::Update,
};

pub struct Button {
    padding:       Padding,
    border_width:  BorderWidth,
    corner_radius: CornerRadius,
    color:         Color,
    hovered_color: Color,
    active_color:  Color,
    border_color:  Color,
    focus_color:   Color,
    on_click:      Box<dyn FnMut()>,
}

impl Button {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetId<Self> {
        let this = cx.insert(Button {
            padding:       Padding::all(8.0),
            border_width:  BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(8.0),
            color:         Color::GREEN,
            hovered_color: Color::RED,
            active_color:  Color::BLUE,
            border_color:  Color::BLACK,
            focus_color:   Color::BLUE,
            on_click:      Box::new(|| {}),
        });

        cx.add_child(this, child);

        this
    }

    pub fn set_child(cx: &mut impl BuildCx, id: WidgetId<Self>, child: impl AnyWidgetId) {
        cx.replace_child(id, 0, child);
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

    pub fn set_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        cx.get_mut(id).color = color;
        cx.request_draw(id);
    }

    pub fn set_hovered_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        cx.get_mut(id).hovered_color = color;
        cx.request_draw(id);
    }

    pub fn set_active_color(cx: &mut impl BuildCx, id: WidgetId<Self>, color: Color) {
        cx.get_mut(id).active_color = color;
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

    pub fn set_on_click(
        cx: &mut impl BuildCx,
        id: WidgetId<Self>,
        on_click: impl FnMut() + 'static,
    ) {
        cx.get_mut(id).on_click = Box::new(on_click);
    }
}

impl Widget for Button {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let space = space.shrink(self.padding.size() + self.border_width.size());
        let size = cx.layout_child(0, space);

        cx.place_child(0, self.padding.offset() + self.border_width.offset());

        size + self.padding.size() + self.border_width.size()
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        let color = if cx.is_active() {
            self.active_color
        } else if cx.is_hovered() {
            self.hovered_color
        } else {
            self.color
        };

        canvas.draw_rect(cx.rect(), self.corner_radius, &Paint::from(color));
        canvas.draw_border(
            cx.rect(),
            self.border_width,
            self.corner_radius,
            &Paint::from(self.border_color),
        );
    }

    fn draw_over(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        if cx.is_focused() && cx.is_window_focused() {
            canvas.draw_border(
                cx.rect().expand(4.0),
                BorderWidth::all(2.0),
                self.corner_radius + 4.0,
                &Paint::from(self.focus_color),
            );
        }
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        match update {
            Update::Hovered(..) | Update::Active(..) | Update::Focused(..) => {
                cx.request_draw();
            }

            _ => {}
        }
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        match event {
            PointerEvent::Down(..) => PointerPropagate::Capture,

            PointerEvent::Up(..) if cx.is_hovered() && cx.is_active() => {
                (self.on_click)();
                PointerPropagate::Bubble
            }

            _ => PointerPropagate::Bubble,
        }
    }

    fn on_key_event(&mut self, _cx: &mut EventCx<'_>, event: &KeyEvent) -> Propagate {
        match event {
            KeyEvent::Up(event)
                if matches!(event.key, Key::Character(ref c) if c == " ")
                    || event.key == Key::Named(NamedKey::Enter) =>
            {
                (self.on_click)();

                Propagate::Stop
            }

            _ => Propagate::Bubble,
        }
    }

    fn accepts_focus() -> bool {
        true
    }
}
