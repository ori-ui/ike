use core::f32;

use crate::{
    Axis, BuildCx, ChildUpdate, LayoutCx, Painter, Size, Space, Update, UpdateCx, Widget, WidgetMut,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Align {
    Start,
    Center,
    End,
    Fill,
}

pub struct Stack {
    axis:    Axis,
    justify: Justify,
    align:   Align,
    gap:     f32,

    flex: Vec<(f32, bool)>,
}

impl Stack {
    pub fn new(cx: &mut impl BuildCx) -> WidgetMut<'_, Self> {
        cx.insert_widget(Self {
            axis:    Axis::Vertical,
            justify: Justify::Start,
            align:   Align::Center,
            gap:     0.0,

            flex: Vec::new(),
        })
    }

    pub fn set_flex(this: &mut WidgetMut<Self>, index: usize, flex: f32, tight: bool) {
        this.widget.flex[index] = (flex, tight);
        this.cx.request_layout();
    }

    pub fn get_flex(&self, index: usize) -> (f32, bool) {
        self.flex[index]
    }

    pub fn set_axis(this: &mut WidgetMut<Self>, axis: Axis) {
        this.widget.axis = axis;
        this.cx.request_layout();
    }

    pub fn set_justify(this: &mut WidgetMut<Self>, justify: Justify) {
        this.widget.justify = justify;
        this.cx.request_layout();
    }

    pub fn set_align(this: &mut WidgetMut<Self>, align: Align) {
        this.widget.align = align;
        this.cx.request_layout();
    }

    pub fn set_gap(this: &mut WidgetMut<Self>, gap: f32) {
        this.widget.gap = gap;
        this.cx.request_layout();
    }
}

impl Widget for Stack {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, painter: &mut dyn Painter, space: Space) -> Size {
        let (min_major, mut min_minor) = self.axis.unpack_size(space.min);
        let (max_major, max_minor) = self.axis.unpack_size(space.max);

        if let Align::Fill = self.align {
            min_minor = max_minor;
        }

        let child_count = cx.children().len();
        let total_gap = self.gap * child_count.saturating_sub(1) as f32;

        let mut flex_sum = 0.0;
        let mut major_sum = total_gap;
        let mut minor_sum = min_minor;

        // measure inflexible children

        for (i, &(flex, _)) in self.flex.iter().enumerate() {
            if flex > 0.0 {
                flex_sum += flex;
                continue;
            };

            let space = Space::new(
                self.axis.pack_size(0.0, min_minor),
                self.axis.pack_size(f32::INFINITY, max_minor),
            );

            let size = cx.layout_child(i, painter, space);
            let (major, minor) = self.axis.unpack_size(size);

            major_sum += major;
            minor_sum = minor_sum.max(minor);
        }

        // measure expanding children

        let remaining = f32::max(max_major - major_sum, 0.0);
        let per_flex = remaining / flex_sum;

        for (i, &(flex, tight)) in self.flex.iter().enumerate() {
            if flex == 0.0 || tight {
                continue;
            }

            let max_major = per_flex * flex;

            let space = Space::new(
                self.axis.pack_size(0.0, min_minor),
                self.axis.pack_size(max_major, max_minor),
            );

            let size = cx.layout_child(i, painter, space);
            let (major, minor) = self.axis.unpack_size(size);

            major_sum += major;
            minor_sum = minor_sum.max(minor);
        }

        // measure tightly flexible children

        let remaining = f32::max(max_major - major_sum, 0.0);
        let per_flex = remaining / flex_sum;

        for (i, &(flex, tight)) in self.flex.iter().enumerate() {
            if flex == 0.0 || !tight {
                continue;
            }

            let major = per_flex * flex;

            let space = Space::new(
                self.axis.pack_size(major, min_minor),
                self.axis.pack_size(major, max_minor),
            );

            let size = cx.layout_child(i, painter, space);
            let (major, minor) = self.axis.unpack_size(size);

            major_sum += major;
            minor_sum = minor_sum.max(minor);
        }

        let major = f32::clamp(major_sum, min_major, max_major);
        let minor = f32::clamp(minor_sum, min_minor, max_minor);

        let excess_major = major - major_sum + total_gap;

        let gap = match self.justify {
            Justify::Start | Justify::Center | Justify::End => self.gap,
            Justify::SpaceBetween => excess_major / (child_count.saturating_sub(1)) as f32,
            Justify::SpaceAround => excess_major / child_count as f32,
            Justify::SpaceEvenly => excess_major / (child_count + 1) as f32,
        };

        let mut justify = match self.justify {
            Justify::Start | Justify::SpaceBetween => 0.0,
            Justify::Center => excess_major / 2.0,
            Justify::End => excess_major,
            Justify::SpaceAround => gap / 2.0,
            Justify::SpaceEvenly => gap,
        };

        for i in 0..child_count {
            let size = cx.child_size(i);
            let (child_major, child_minor) = self.axis.unpack_size(size);

            let excess_minor = minor - child_minor;

            let align = match self.align {
                Align::Start | Align::Fill => 0.0,
                Align::Center => excess_minor / 2.0,
                Align::End => excess_minor,
            };

            let offset = self.axis.pack_offset(justify, align);
            cx.place_child(i, offset);

            justify += child_major + gap;
        }

        self.axis.pack_size(major, minor)
    }

    fn update(&mut self, _cx: &mut UpdateCx<'_>, update: Update) {
        if let Update::Children(update) = update {
            match update {
                ChildUpdate::Added(..) => {
                    self.flex.push((0.0, false));
                }

                ChildUpdate::Removed(i) => {
                    self.flex.remove(i);
                }

                ChildUpdate::Swapped(a, b) => {
                    self.flex.swap(a, b);
                }

                ChildUpdate::Replaced(i) => {
                    self.flex[i] = (0.0, false);
                }
            }
        }
    }
}
