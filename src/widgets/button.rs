use std::time::Duration;

use crate::{
    AnyWidgetId, BorderWidth, BuildCx, Canvas, Color, CornerRadius, DrawCx, EventCx, Key, KeyEvent,
    LayoutCx, NamedKey, Padding, Paint, Painter, PointerEvent, PointerPropagate, Propagate, Size,
    Space, Transition, Transitioned, Widget, WidgetMut, context::UpdateCx, widget::Update,
};

pub struct Button {
    padding:       Padding,
    border_width:  BorderWidth,
    corner_radius: CornerRadius,
    color:         Transitioned<Color>,
    idle_color:    Color,
    hovered_color: Color,
    active_color:  Color,
    border_color:  Color,
    focus_color:   Color,
    on_click:      Box<dyn FnMut()>,
}

impl Button {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        let mut this = cx.insert(Button {
            padding:       Padding::all(8.0),
            border_width:  BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(8.0),
            color:         Transitioned::new(Color::GREEN, Transition::INSTANT),
            idle_color:    Color::GREEN,
            hovered_color: Color::RED,
            active_color:  Color::BLUE,
            border_color:  Color::BLACK,
            focus_color:   Color::BLUE,
            on_click:      Box::new(|| {}),
        });

        this.add_child(child);

        this
    }

    pub fn set_child(this: &mut WidgetMut<Self>, child: impl AnyWidgetId) {
        this.replace_child(0, child);
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

    pub fn set_border_color(this: &mut WidgetMut<Self>, color: Color) {
        this.border_color = color;
        this.request_draw();
    }

    pub fn set_focus_color(this: &mut WidgetMut<Self>, color: Color) {
        this.focus_color = color;
        this.request_draw();
    }

    pub fn set_idle_color(this: &mut WidgetMut<Self>, color: Color) {
        this.idle_color = color;
        Self::update_color(this);
    }

    pub fn set_hovered_color(this: &mut WidgetMut<Self>, color: Color) {
        this.hovered_color = color;
        Self::update_color(this);
    }

    pub fn set_active_color(this: &mut WidgetMut<Self>, color: Color) {
        this.active_color = color;
        Self::update_color(this);
    }

    pub fn set_transition(this: &mut WidgetMut<Self>, transition: Transition) {
        this.color.set_transition(transition);
    }

    fn update_color(this: &mut WidgetMut<Self>) {
        let color = if this.is_active() {
            this.active_color
        } else if this.is_hovered() {
            this.hovered_color
        } else {
            this.idle_color
        };

        this.request_draw();

        if this.color.begin(color) {
            this.request_animate();
        }
    }

    pub fn set_on_click(this: &mut WidgetMut<Self>, on_click: impl FnMut() + 'static) {
        this.on_click = Box::new(on_click);
    }
}

impl Widget for Button {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size {
        let space = space.shrink(self.padding.size() + self.border_width.size());
        let size = cx.layout_child(0, painter, space);

        cx.place_child(0, self.padding.offset() + self.border_width.offset());

        size + self.padding.size() + self.border_width.size()
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_rect(cx.rect(), self.corner_radius, &Paint::from(*self.color));
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
                let color = if cx.is_active() {
                    self.active_color
                } else if cx.is_hovered() {
                    self.hovered_color
                } else {
                    self.idle_color
                };

                cx.request_draw();

                if self.color.begin(color) {
                    cx.request_animate();
                }
            }

            _ => {}
        }
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        if self.color.animate(dt) {
            cx.request_animate();
            cx.request_draw();
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

    fn accepts_pointer() -> bool {
        true
    }

    fn accepts_focus() -> bool {
        true
    }
}
