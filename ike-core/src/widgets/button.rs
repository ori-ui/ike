use std::time::Duration;

use crate::{
    AnyWidgetId, BorderWidth, Builder, Canvas, Color, CornerRadius, DrawCx, EventCx, Gesture, Key,
    KeyEvent, LayoutCx, NamedKey, Padding, Paint, PointerButton, PointerEvent, PointerPropagate,
    Propagate, Size, Space, TouchEvent, TouchPropagate, Transition, Transitioned, Widget,
    WidgetMut, context::UpdateCx, widget::Update,
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
    pub fn new(cx: &mut impl Builder, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        cx.build_widget(Button {
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
        })
        .with_child(child)
        .finish()
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

    pub fn set_border_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.border_color = color;
        this.cx.request_draw();
    }

    pub fn set_focus_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.focus_color = color;
        this.cx.request_draw();
    }

    pub fn set_idle_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.idle_color = color;
        Self::update_color(this);
    }

    pub fn set_hovered_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.hovered_color = color;
        Self::update_color(this);
    }

    pub fn set_active_color(this: &mut WidgetMut<Self>, color: Color) {
        this.widget.active_color = color;
        Self::update_color(this);
    }

    pub fn set_transition(this: &mut WidgetMut<Self>, transition: Transition) {
        this.widget.color.set_transition(transition);
    }

    fn update_color(this: &mut WidgetMut<Self>) {
        let color = if this.cx.is_active() {
            this.widget.active_color
        } else if this.cx.is_hovered() {
            this.widget.hovered_color
        } else {
            this.widget.idle_color
        };

        this.cx.request_draw();

        if this.widget.color.begin(color) {
            this.cx.request_animate();
        }
    }

    pub fn set_on_click(this: &mut WidgetMut<Self>, on_click: impl FnMut() + 'static) {
        this.widget.on_click = Box::new(on_click);
    }
}

impl Widget for Button {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let size = {
            let space = self.padding.layout_down(cx, space);
            let space = self.border_width.layout_down(cx, space);
            cx.layout_nth_child(0, space)
        };

        let offset = self.padding.aligned_offset(cx) + self.border_width.aligned_offset(cx);
        cx.place_nth_child(0, offset);

        let size = self.padding.layout_up(cx, size);
        let size = self.border_width.layout_up(cx, size);
        space.constrain(size)
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_rect(
            cx.rect(),
            self.corner_radius,
            &Paint::from(*self.color),
        );

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
                cx.rect(),
                BorderWidth::all(2.0),
                self.corner_radius,
                &Paint::from(self.focus_color),
            );
        }
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        match update {
            Update::Hovered(..) | Update::Active(..) | Update::Focused(..) => {
                cx.request_draw();

                let color = if cx.is_active() {
                    self.active_color
                } else if cx.is_hovered() {
                    self.hovered_color
                } else {
                    self.idle_color
                };

                if self.color.begin(color) {
                    cx.request_animate();
                }
            }

            _ => {}
        }
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        cx.request_draw();

        if self.color.animate(dt) {
            cx.request_animate();
        }
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        match event {
            PointerEvent::Down(event) if event.button == PointerButton::Primary => {
                PointerPropagate::Capture
            }

            PointerEvent::Up(..) if cx.is_hovered() && cx.is_active() => {
                (self.on_click)();
                PointerPropagate::Bubble
            }

            _ => PointerPropagate::Bubble,
        }
    }

    fn on_touch_event(&mut self, _cx: &mut EventCx<'_>, event: &TouchEvent) -> TouchPropagate {
        match event {
            TouchEvent::Gesture(Gesture::Tap(..)) => {
                (self.on_click)();
                TouchPropagate::Handled
            }

            _ => TouchPropagate::Bubble,
        }
    }

    fn on_key_event(&mut self, _cx: &mut EventCx<'_>, event: &KeyEvent) -> Propagate {
        match event {
            KeyEvent::Down(event)
                if matches!(event.key, Key::Character(ref c) if c == " ")
                    || event.key == Key::Named(NamedKey::Enter) =>
            {
                (self.on_click)();

                Propagate::Handled
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
