use std::time::Duration;

use crate::{
    AnyWidgetId, Axis, BorderWidth, Builder, Canvas, Color, ComposeCx, CornerRadius, DrawCx,
    EventCx, Gesture, LayoutCx, Offset, Padding, Paint, Point, PointerEvent, PointerPropagate,
    Rect, ScrollDelta, Size, Space, TouchEvent, TouchPropagate, Transition, Transitioned, Update,
    UpdateCx, Widget, WidgetId, WidgetMut,
};

pub struct Scroll {
    portal:  WidgetId<Portal>,
    vbar:    WidgetId<ScrollBar>,
    hbar:    WidgetId<ScrollBar>,
    overlay: bool,

    scroll: Transitioned<Offset>,
}

impl Scroll {
    pub fn new(cx: &mut impl Builder, child: impl AnyWidgetId) -> WidgetMut<'_, Self> {
        let portal = cx
            .build_widget(Portal {
                overflow:      Size::ZERO,
                offset:        Offset::ZERO,
                loosen_width:  false,
                loosen_height: true,
            })
            .with_child(child)
            .finish()
            .id();

        let vbar = cx
            .build_widget(ScrollBar::new(Axis::Vertical, portal))
            .finish()
            .id();

        let hbar = cx
            .build_widget(ScrollBar::new(Axis::Horizontal, portal))
            .finish()
            .id();

        cx.set_stashed(hbar, true);

        cx.build_widget(Self {
            vbar,
            hbar,
            portal,
            overlay: false,

            scroll: Transitioned::new(Offset::ZERO, Transition::ease(0.25)),
        })
        .with_child(portal)
        .with_child(vbar)
        .with_child(hbar)
        .finish()
    }

    pub fn is_child(cx: &impl Builder, this: WidgetId<Self>, child: impl AnyWidgetId) -> bool {
        cx.get_widget(this)
            .is_ok_and(|this| cx.is_child(this.widget.portal, child))
    }

    pub fn set_child(cx: &mut impl Builder, this: WidgetId<Self>, child: impl AnyWidgetId) {
        if let Ok(portal) = cx.get_widget(this).map(|this| this.widget.portal) {
            cx.set_child(portal, 0, child);
        }
    }

    pub fn set_overlay(this: &mut WidgetMut<Self>, enabled: bool) {
        this.widget.overlay = enabled;
        this.cx.request_layout();
    }

    pub fn set_vertical(this: &mut WidgetMut<Self>, enabled: bool) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.set_stashed(!enabled);
        }
    }

    pub fn set_horizontal(this: &mut WidgetMut<Self>, enabled: bool) {
        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.set_stashed(!enabled);
        }
    }

    pub fn set_bar_thickness(this: &mut WidgetMut<Self>, thickness: f32) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.widget.thickness = thickness;
            vbar.cx.request_layout();
        }

        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.widget.thickness = thickness;
            hbar.cx.request_layout();
        }
    }

    pub fn set_bar_padding(this: &mut WidgetMut<Self>, padding: Padding) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.widget.padding = padding;
            vbar.cx.request_layout();
        }

        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.widget.padding = padding;
            hbar.cx.request_layout();
        }
    }

    pub fn set_bar_border_width(this: &mut WidgetMut<Self>, border_width: BorderWidth) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.widget.border_width = border_width;
            vbar.cx.request_layout();
        }

        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.widget.border_width = border_width;
            hbar.cx.request_layout();
        }
    }

    pub fn set_bar_corner_radius(this: &mut WidgetMut<Self>, corner_radius: CornerRadius) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.widget.corner_radius = corner_radius;
            vbar.cx.request_layout();
        }

        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.widget.corner_radius = corner_radius;
            hbar.cx.request_layout();
        }
    }

    pub fn set_knob_corner_radius(this: &mut WidgetMut<Self>, corner_radius: CornerRadius) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.widget.knob_corner_radius = corner_radius;
            vbar.cx.request_draw();
        }

        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.widget.knob_corner_radius = corner_radius;
            hbar.cx.request_draw();
        }
    }

    pub fn set_bar_border_paint(this: &mut WidgetMut<Self>, paint: Paint) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.widget.border_paint = paint.clone();
            vbar.cx.request_draw();
        }

        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.widget.border_paint = paint;
            hbar.cx.request_draw();
        }
    }

    pub fn set_bar_paint(this: &mut WidgetMut<Self>, paint: Paint) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.widget.background_paint = paint.clone();
            vbar.cx.request_draw();
        }

        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.widget.background_paint = paint;
            hbar.cx.request_draw();
        }
    }

    pub fn set_knob_paint(this: &mut WidgetMut<Self>, paint: Paint) {
        if let Ok(mut vbar) = this.cx.get_widget_mut(this.widget.vbar) {
            vbar.widget.knob_paint = paint.clone();
            vbar.cx.request_draw();
        }

        if let Ok(mut hbar) = this.cx.get_widget_mut(this.widget.hbar) {
            hbar.widget.knob_paint = paint;
            hbar.cx.request_draw();
        }
    }

    pub fn set_transition(this: &mut WidgetMut<Self>, transition: Transition) {
        this.widget.scroll.set_transition(transition);
    }
}

impl Widget for Scroll {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let vbar_width = cx.get_child(self.vbar).map_or(0.0, |vbar| {
            vbar.widget.thickness
                + vbar.widget.padding.size().width
                + vbar.widget.border_width.size().height
        });

        let hbar_height = cx.get_child(self.hbar).map_or(0.0, |hbar| {
            hbar.widget.thickness
                + hbar.widget.padding.size().height
                + hbar.widget.border_width.size().height
        });

        let child_size = {
            let space = match self.overlay {
                false => space.shrink(Size::new(vbar_width, hbar_height)),
                true => space,
            };

            cx.layout_child(self.portal, space)
        };

        let vbar_size = cx.layout_child(
            self.vbar,
            Space {
                min: Size {
                    width:  0.0,
                    height: child_size.height.min(space.max.height),
                },
                max: Size {
                    width:  space.max.width,
                    height: child_size.height.min(space.max.height),
                },
            },
        );
        let hbar_size = cx.layout_child(
            self.hbar,
            Space {
                min: Size {
                    width:  child_size.width.min(space.max.width),
                    height: 0.0,
                },
                max: Size {
                    width:  child_size.width.min(space.max.width),
                    height: space.max.height,
                },
            },
        );

        if self.overlay {
            let vbar = Offset {
                x: child_size.width - vbar_size.width,
                y: 0.0,
            };
            let hbar = Offset {
                x: 0.0,
                y: child_size.height - hbar_size.height,
            };

            cx.place_child(self.vbar, vbar);
            cx.place_child(self.hbar, hbar);
        } else {
            cx.place_child(
                self.vbar,
                Offset::new(child_size.width, 0.0),
            );
            cx.place_child(
                self.hbar,
                Offset::new(0.0, child_size.height),
            );
        }

        let mut size = child_size;

        if !self.overlay {
            size.width += vbar_size.width;
            size.height += hbar_size.height;
        }

        space.constrain(size)
    }

    fn compose(&mut self, cx: &mut ComposeCx<'_>) {
        let Ok(overflow) = cx
            .get_child(self.portal)
            .map(|portal| portal.widget.overflow)
        else {
            return;
        };

        let mut scroll = *self.scroll;
        scroll.x = scroll.x.clamp(0.0, overflow.width);
        scroll.y = scroll.y.clamp(0.0, overflow.height);

        let mut vbar_scroll = scroll.y / overflow.height;
        let mut hbar_scroll = scroll.x / overflow.width;

        if !vbar_scroll.is_finite() {
            vbar_scroll = 0.0;
        }

        if !hbar_scroll.is_finite() {
            hbar_scroll = 0.0;
        }

        if let Ok(mut vbar) = cx.get_child_mut(self.vbar) {
            vbar.widget.scroll = vbar_scroll;
            vbar.cx.request_draw();
        }

        if let Ok(mut hbar) = cx.get_child_mut(self.hbar) {
            hbar.widget.scroll = hbar_scroll;
            hbar.cx.request_draw();
        }

        let vbar_knob_length = cx.height() / (cx.height() + overflow.height);
        if let Ok(mut vbar) = cx.get_child_mut(self.vbar) {
            vbar.widget.knob_length = vbar_knob_length;
            vbar.cx.request_draw();
        }

        let hbar_knob_length = cx.width() / (cx.width() + overflow.width);
        if let Ok(mut hbar) = cx.get_child_mut(self.hbar) {
            hbar.widget.knob_length = hbar_knob_length;
            hbar.cx.request_draw();
        }

        if let Ok(mut portal) = cx.get_child_mut(self.portal) {
            portal.widget.offset = self.scroll.get();
            portal.cx.request_compose();
        }

        if scroll != *self.scroll {
            self.scroll.set(scroll);
        }
    }

    fn animate(&mut self, cx: &mut UpdateCx<'_>, dt: Duration) {
        cx.request_compose();

        if self.scroll.animate(dt) {
            cx.request_animate();
            cx.set_subpixel(true);
        } else {
            cx.set_subpixel(false);
        }
    }

    fn update(&mut self, cx: &mut UpdateCx<'_>, update: Update) {
        if let Update::ScrollTo(mut target) = update
            && let Ok(size) = cx.get_child(self.portal).map(|portal| portal.cx.size())
            && let Ok(overflow) = cx
                .get_child(self.portal)
                .map(|portal| portal.widget.overflow)
        {
            target.min += Offset::all(-8.0);
            target.max += Offset::all(8.0);

            let mut x = if target.max.x - target.min.x > size.width {
                target.center().x - size.width / 2.0
            } else if target.min.x < 0.0 {
                target.min.x
            } else if target.max.x > size.width {
                target.max.x - size.width
            } else {
                0.0
            };

            let mut y = if target.max.y - target.min.y > size.height {
                target.center().y - size.height / 2.0
            } else if target.min.y < 0.0 {
                target.min.y
            } else if target.max.y > size.height {
                target.max.y - size.height
            } else {
                0.0
            };

            x += self.scroll.x;
            y += self.scroll.y;
            x = x.clamp(0.0, overflow.width);
            y = y.clamp(0.0, overflow.height);

            if self.scroll.begin(Offset::new(x, y)) {
                cx.request_animate();
            }

            cx.request_compose();
        }
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        let Ok(overflow) = cx
            .get_child(self.portal)
            .map(|portal| portal.widget.overflow)
        else {
            return PointerPropagate::Bubble;
        };

        match event {
            PointerEvent::Scroll(event) => {
                let mut scroll = self.scroll.end();
                scroll += match event.delta {
                    ScrollDelta::Line(offset) => -offset * 120.0,
                    ScrollDelta::Pixel(offset) => -offset,
                };

                scroll.x = scroll.x.clamp(0.0, overflow.width);
                scroll.y = scroll.y.clamp(0.0, overflow.height);

                if self.scroll.begin(scroll) {
                    cx.request_animate();
                }

                PointerPropagate::Handled
            }

            _ => PointerPropagate::Bubble,
        }
    }

    fn on_touch_event(&mut self, cx: &mut EventCx<'_>, event: &TouchEvent) -> TouchPropagate {
        let Ok(overflow) = cx
            .get_child(self.portal)
            .map(|portal| portal.widget.overflow)
        else {
            return TouchPropagate::Bubble;
        };

        match event {
            TouchEvent::Gesture(Gesture::Pan(event)) => {
                let mut scroll = self.scroll.end();
                scroll -= event.delta;

                scroll.x = scroll.x.clamp(0.0, overflow.width);
                scroll.y = scroll.y.clamp(0.0, overflow.height);

                self.scroll.set(scroll);
                cx.request_compose();

                TouchPropagate::Capture
            }

            _ => TouchPropagate::Bubble,
        }
    }

    fn accepts_pointer() -> bool {
        true
    }
}

struct Portal {
    overflow:      Size,
    offset:        Offset,
    loosen_width:  bool,
    loosen_height: bool,
}

impl Widget for Portal {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let child_size = {
            let mut space = space;

            if self.loosen_width {
                space.max.width = f32::INFINITY;
            }

            if self.loosen_height {
                space.max.height = f32::INFINITY;
            }

            cx.layout_nth_child(0, space)
        };

        let size = space.constrain(child_size);

        self.overflow = child_size - size;
        self.overflow = self.overflow.max(Size::ZERO);
        cx.set_clip(Rect::min_size(Point::ORIGIN, size));

        size
    }

    fn compose(&mut self, cx: &mut ComposeCx<'_>) {
        cx.place_nth_child(0, -self.offset);
    }
}

struct ScrollBar {
    axis:             Axis,
    thickness:        f32,
    padding:          Padding,
    border_width:     BorderWidth,
    corner_radius:    CornerRadius,
    border_paint:     Paint,
    background_paint: Paint,

    knob_corner_radius: CornerRadius,
    knob_paint:         Paint,

    knob_length: f32,
    scroll:      f32,
    draw_offset: f32,

    portal: WidgetId<Portal>,
}

impl ScrollBar {
    fn new(axis: Axis, portal: WidgetId<Portal>) -> Self {
        Self {
            axis,
            thickness: 8.0,
            padding: Padding::all(4.0),
            border_width: BorderWidth::all(1.0),
            corner_radius: CornerRadius::all(0.0),
            border_paint: Paint::from(Color::BLACK),
            background_paint: Paint::from(Color::WHITE),

            knob_corner_radius: CornerRadius::all(4.0),
            knob_paint: Paint::from(Color::BLACK),

            knob_length: 0.5,
            scroll: 0.0,
            draw_offset: 0.0,

            portal,
        }
    }
}

impl Widget for ScrollBar {
    fn layout(&mut self, _cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let (max_major, _) = self.axis.unpack_size(space.max);
        let (_, padding_minor) = self.axis.unpack_size(self.padding.size());
        let (_, border_minor) = self.axis.unpack_size(self.border_width.size());

        let minor = self.thickness + padding_minor + border_minor;
        self.axis.pack_size(max_major, minor)
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        canvas.draw_rect(
            cx.rect(),
            self.corner_radius,
            &self.background_paint,
        );

        canvas.draw_border(
            cx.rect(),
            self.border_width,
            self.corner_radius,
            &self.border_paint,
        );

        let track_size = cx.size() - self.padding.size() - self.border_width.size();
        let (major, _) = self.axis.unpack_size(track_size);

        let length = self.knob_length * major;
        let excess = major - length;

        let knob = Rect::min_size(
            Point {
                x: self.padding.left + self.border_width.left,
                y: self.padding.top + self.border_width.top,
            } + self.axis.pack_offset(self.scroll * excess, 0.0),
            self.axis.pack_size(length, self.thickness),
        );

        canvas.draw_rect(
            knob,
            self.knob_corner_radius,
            &self.knob_paint,
        );
    }

    fn on_pointer_event(&mut self, cx: &mut EventCx<'_>, event: &PointerEvent) -> PointerPropagate {
        let (major, _) = self.axis.unpack_size(cx.size());

        let padding = self.border_width.size() + self.padding.size();
        let offset = self.border_width.offset() + self.padding.offset();

        let (padding, _) = self.axis.unpack_size(padding);
        let (offset, _) = self.axis.unpack_offset(offset);

        let length = major - padding;
        let space = length * (1.0 - self.knob_length);

        let start = offset + self.scroll * space;
        let end = start + length * self.knob_length;

        match event {
            PointerEvent::Down(event) => {
                let local = cx.global_transform().inverse() * event.position;
                let (local, _) = self.axis.unpack_point(local);

                if !(start..end).contains(&local) {
                    self.scroll = (local - offset - length * self.knob_length / 2.0) / space;
                    self.scroll = self.scroll.clamp(0.0, 1.0);
                    set_scroll(self, cx);
                }

                self.draw_offset = local - offset - self.scroll * space;
                cx.request_draw();

                PointerPropagate::Capture
            }

            PointerEvent::Move(event) if cx.is_active() => {
                let local = cx.global_transform().inverse() * event.position;
                let (local, _) = self.axis.unpack_point(local);

                let position = local - offset - self.draw_offset;
                self.scroll = position / space;
                self.scroll = self.scroll.clamp(0.0, 1.0);
                set_scroll(self, cx);

                PointerPropagate::Bubble
            }

            _ => PointerPropagate::Bubble,
        }
    }

    fn accepts_pointer() -> bool {
        true
    }
}

fn set_scroll(bar: &mut ScrollBar, cx: &mut EventCx<'_>) {
    if let Ok(overflow) = cx
        .get_widget(bar.portal)
        .map(|portal| portal.widget.overflow)
        && let Ok(parent) = cx.get_parent_mut()
        && let Some(mut scroll) = parent.downcast::<Scroll>()
    {
        let mut end = scroll.widget.scroll.end();

        match bar.axis {
            Axis::Vertical => {
                end.y = bar.scroll * overflow.height;
            }

            Axis::Horizontal => {
                end.x = bar.scroll * overflow.width;
            }
        }

        scroll.cx.request_compose();
        scroll.widget.scroll.set(end);
    }
}
