use std::time::Duration;

use crate::{
    AnyWidgetId, Axis, BorderWidth, BuildCx, Canvas, Color, ComposeCx, CornerRadius, DrawCx,
    EventCx, Gesture, LayoutCx, Padding, Paint, Painter, Point, PointerEvent, PointerPropagate,
    Rect, ScrollDelta, Size, Space, TouchEvent, TouchPropagate, Transition, Transitioned, Update,
    UpdateCx, Widget, WidgetMut, build::SingleChild,
};

pub struct Scroll {
    axis:               Axis,
    bar_width:          f32,
    bar_padding:        Padding,
    bar_border_width:   BorderWidth,
    bar_corner_radius:  CornerRadius,
    knob_corner_radius: CornerRadius,
    bar_border_paint:   Paint,
    bar_paint:          Paint,
    knob_paint:         Paint,

    knob_length:  f32,
    scroll_major: f32,
    child_major:  f32,
    scroll:       Transitioned<f32>,

    drag_offset: f32,
}

impl Scroll {
    pub fn new(cx: &mut impl BuildCx, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        cx.insert_widget(
            Self {
                axis:               Axis::Vertical,
                bar_width:          16.0,
                bar_padding:        Padding::all(4.0),
                bar_border_width:   BorderWidth::all(1.0),
                bar_corner_radius:  CornerRadius::all(0.0),
                knob_corner_radius: CornerRadius::all(4.0),
                bar_paint:          Paint::from(Color::WHITE),
                bar_border_paint:   Paint::from(Color::BLACK),
                knob_paint:         Paint::from(Color::BLACK),

                knob_length:  0.0,
                scroll_major: 0.0,
                child_major:  0.0,
                scroll:       Transitioned::new(0.0, Transition::ease(0.25)),

                drag_offset: 0.0,
            },
            child,
        )
    }

    pub fn set_axis(this: &mut WidgetMut<Self>, axis: Axis) {
        this.widget.axis = axis;
        this.cx.request_layout();
    }

    pub fn set_bar_width(this: &mut WidgetMut<Self>, width: f32) {
        this.widget.bar_width = width;
        this.cx.request_layout();
    }

    pub fn set_bar_padding(this: &mut WidgetMut<Self>, padding: Padding) {
        this.widget.bar_padding = padding;
        this.cx.request_layout();
    }

    pub fn set_bar_border_width(this: &mut WidgetMut<Self>, border_width: BorderWidth) {
        this.widget.bar_border_width = border_width;
        this.cx.request_layout();
    }

    pub fn set_bar_corner_radius(this: &mut WidgetMut<Self>, corner_radius: CornerRadius) {
        this.widget.bar_corner_radius = corner_radius;
        this.cx.request_draw();
    }

    pub fn set_knob_corner_radius(this: &mut WidgetMut<Self>, corner_radius: CornerRadius) {
        this.widget.knob_corner_radius = corner_radius;
        this.cx.request_draw();
    }

    pub fn set_bar_border_paint(this: &mut WidgetMut<Self>, paint: Paint) {
        this.widget.bar_border_paint = paint;
        this.cx.request_draw();
    }

    pub fn set_bar_paint(this: &mut WidgetMut<Self>, paint: Paint) {
        this.widget.bar_paint = paint;
        this.cx.request_draw();
    }

    pub fn set_knob_paint(this: &mut WidgetMut<Self>, paint: Paint) {
        this.widget.knob_paint = paint;
        this.cx.request_draw();
    }

    pub fn set_transition(this: &mut WidgetMut<Self>, transition: Transition) {
        this.widget.scroll.set_transition(transition);
    }

    fn overflow(&self) -> f32 {
        f32::max(
            self.child_major - self.scroll_major,
            0.0,
        )
    }

    fn scroll_fraction(&self) -> f32 {
        if self.scroll_major >= self.child_major {
            return 0.0;
        }

        *self.scroll / self.overflow()
    }

    fn bar_rect(&self, size: Size) -> Rect {
        let (major, minor) = self.axis.unpack_size(size);

        Rect::min_size(
            self.axis.pack_point(0.0, minor - self.bar_width),
            self.axis.pack_size(major, self.bar_width),
        )
    }

    fn track_start(&self) -> f32 {
        let offset = self.bar_padding.offset() + self.bar_border_width.offset();
        self.axis.unpack_offset(offset).0
    }

    fn track_width(&self) -> f32 {
        let size = self.bar_padding.size() + self.bar_border_width.size();
        let (_, padding) = self.axis.unpack_size(size);
        self.bar_width - padding
    }

    fn track_length(&self) -> f32 {
        let size = self.bar_padding.size() + self.bar_border_width.size();
        let (padding, _) = self.axis.unpack_size(size);
        self.scroll_major - padding
    }

    fn knob_space(&self) -> f32 {
        let size = self.bar_padding.size() + self.bar_border_width.size();
        let (padding, _) = self.axis.unpack_size(size);
        self.scroll_major - padding - self.knob_length
    }

    fn knob_offset(&self) -> f32 {
        self.track_start() + self.knob_space() * self.scroll_fraction()
    }
}

impl SingleChild for Scroll {}

impl Widget for Scroll {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size {
        let (min_major, min_minor) = self.axis.unpack_size(space.min);
        let (_, max_minor) = self.axis.unpack_size(space.max);

        let child_space = Space {
            min: self.axis.pack_size(
                min_major, // retain minimum size
                f32::max(0.0, min_minor - self.bar_width),
            ),
            max: self.axis.pack_size(
                f32::INFINITY, // allow as much content as possible
                f32::max(0.0, max_minor - self.bar_width),
            ),
        };

        let child_size = cx.layout_child(0, painter, child_space);
        let size = space.constrain(child_size + self.axis.pack_size(0.0, self.bar_width));

        let (child_major, _) = self.axis.unpack_size(child_size);
        let (major, _) = self.axis.unpack_size(size);

        self.child_major = child_major;
        self.scroll_major = major;
        self.knob_length = major / child_major * self.track_length();
        self.knob_length = self.knob_length.max(24.0);

        let clamped_scroll = self.scroll.clamp(0.0, self.overflow());
        if *self.scroll != clamped_scroll {
            self.scroll.set(clamped_scroll);
        }

        let offset = self.axis.pack_offset(-*self.scroll, 0.0);
        cx.place_child(0, offset);
        cx.set_clip(Rect::min_size(Point::ORIGIN, size));

        size
    }

    fn compose(&mut self, cx: &mut ComposeCx<'_>) {
        let offset = self.axis.pack_offset(-*self.scroll, 0.0);
        cx.place_child(0, offset);
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        let (_, minor) = self.axis.unpack_size(cx.size());

        canvas.draw_rect(
            self.bar_rect(cx.size()),
            self.bar_corner_radius,
            &self.bar_paint,
        );

        if self.child_major == 0.0 {
            return;
        }

        let offset = self.bar_padding.offset() + self.bar_border_width.offset();
        let (_, padding) = self.axis.unpack_offset(offset);
        let knob_minor = minor - self.bar_width + padding;
        let knob = Rect::min_size(
            self.axis.pack_point(self.knob_offset(), knob_minor),
            self.axis.pack_size(self.knob_length, self.track_width()),
        );

        canvas.draw_rect(
            knob,
            self.knob_corner_radius,
            &self.knob_paint,
        );

        canvas.draw_border(
            self.bar_rect(cx.size()),
            self.bar_border_width,
            self.bar_corner_radius,
            &self.bar_border_paint,
        );
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        if let Update::ScrollTo(target) = update {
            let (mut min, _) = self.axis.unpack_point(target.min);
            let (mut max, _) = self.axis.unpack_point(target.max);

            min -= 8.0;
            max += 8.0;

            let mut scroll = if max - min > self.scroll_major {
                let center = (min + max) / 2.0;

                *self.scroll + center - self.scroll_major / 2.0
            } else if min < 0.0 {
                *self.scroll + min
            } else if max > self.scroll_major {
                *self.scroll + max - self.scroll_major
            } else {
                return;
            };

            scroll = scroll.clamp(0.0, self.overflow());

            if self.scroll.begin(scroll) {
                cx.request_animate();
            }

            cx.request_compose();
        }
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        cx.request_compose();

        if self.scroll.animate(dt) {
            cx.request_animate();
            cx.set_pixel_perfect(false);
        } else {
            cx.set_pixel_perfect(true);
        }
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        match event {
            PointerEvent::Down(event) => {
                let local = cx.global_transform().inverse() * event.position;

                if self.bar_rect(cx.size()).contains(local) {
                    let (major, _) = self.axis.unpack_point(local);
                    let knob_end = self.knob_offset() + self.knob_length;

                    if major < self.knob_offset() || major > knob_end {
                        let point = major - self.track_start() - self.knob_length / 2.0;
                        let fraction = point.clamp(0.0, self.knob_space()) / self.knob_space();
                        self.scroll.set(self.overflow() * fraction);
                    }

                    self.drag_offset = major - self.knob_offset();
                    cx.request_compose();

                    PointerPropagate::Capture
                } else {
                    PointerPropagate::Bubble
                }
            }

            PointerEvent::Move(event) if cx.is_active() => {
                if self.knob_space() <= 0.0 {
                    return PointerPropagate::Bubble;
                }

                let local = cx.global_transform().inverse() * event.position;

                let (major, _) = self.axis.unpack_point(local);
                let point = major - self.track_start() - self.drag_offset;
                let fraction = point.clamp(0.0, self.knob_space()) / self.knob_space();
                let scroll = self.overflow() * fraction;

                if *self.scroll != scroll {
                    self.scroll.set(scroll);
                    cx.request_compose();
                }

                PointerPropagate::Bubble
            }

            PointerEvent::Scroll(event) if cx.has_hovered() => {
                let delta = match event.delta {
                    ScrollDelta::Line(offset) => offset * 120.0,
                    ScrollDelta::Pixel(offset) => offset,
                };

                let mut scroll = self.scroll.to() - self.axis.unpack_offset(delta).0;
                scroll = scroll.clamp(0.0, self.overflow());

                match event.delta {
                    ScrollDelta::Line(..) => {
                        if self.scroll.begin(scroll) {
                            cx.request_animate();
                        }
                    }

                    ScrollDelta::Pixel(..) => {
                        if *self.scroll != scroll {
                            self.scroll.set(scroll);
                            cx.request_compose();
                        }
                    }
                }

                PointerPropagate::Handled
            }

            _ => PointerPropagate::Bubble,
        }
    }

    fn on_touch_event(&mut self, cx: &mut EventCx<'_>, event: &TouchEvent) -> TouchPropagate {
        match event {
            TouchEvent::Gesture(Gesture::Pan(event)) => {
                let (delta, _) = self.axis.unpack_offset(event.delta);
                let mut scroll = self.scroll.to() - delta;
                scroll = scroll.clamp(0.0, self.overflow());

                if *self.scroll != scroll {
                    self.scroll.set(scroll);
                    cx.request_compose();
                }

                TouchPropagate::Handled
            }

            _ => TouchPropagate::Bubble,
        }
    }

    fn accepts_pointer() -> bool {
        true
    }
}
