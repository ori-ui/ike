use crate::{
    Axis, Builder, ChildUpdate, LayoutCx, Size, Space, Update, UpdateCx, Widget, WidgetMut, layout,
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
    pub fn new(cx: &mut impl Builder) -> WidgetMut<'_, Self> {
        cx.build_widget(Self {
            axis:    Axis::Vertical,
            justify: Justify::Start,
            align:   Align::Center,
            gap:     0.0,

            flex: Vec::new(),
        })
        .finish()
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
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let (min_major, min_minor) = self.axis.unpack_size(space.min);
        let (max_major, max_minor) = self.axis.unpack_size(space.max);

        let child_min_minor = match self.align {
            Align::Fill => max_minor,
            _ => 0.0,
        };

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
                self.axis.pack_size(0.0, child_min_minor),
                self.axis.pack_size(f32::INFINITY, max_minor),
            );

            let size = cx.layout_nth_child(i, space);
            let (major, minor) = self.axis.unpack_size(size);

            major_sum += major;
            minor_sum = minor_sum.max(minor);
        }

        // measure expanding children

        let mut remaining = f32::max(max_major - major_sum, 0.0);

        for (i, &(flex, tight)) in self.flex.iter().enumerate() {
            if flex == 0.0 || tight {
                continue;
            }

            let per_flex = remaining / flex_sum;
            let subpixel_max_major = per_flex * flex;

            let max_major = if cx.settings().render.pixel_align {
                // imagine three children with a flex of 1.0 and a remaining size of 20.0, each
                // child will is allotted 20.0 / 3.0 = 6.667.. pixels. since space is floored to
                // device pixels, each child will only get a max size of 6.0, which when summed up
                // to 3.0 * 6.0 = 18.0 leaves 2 pixels left over. to avoid this max_major is
                // rounded to the nearest pixel, and the difference is added to remaining.
                //
                // for the child #1, round(20.0 / 3.0 = 6.667..) = 7.0
                // remaining - 0.333.. = 19.667..
                //
                // for the child #2, round(19.667.. / 3.0 = 6.556..) = 7.0
                // remaining - 0.444.. = 19.222..
                //
                // for the child #3, round(19.222.. / 3.0 = 6.407..) = 6.0
                //
                // leaving us with 0 pixels in excess.
                let max_major = layout::pixel_round(subpixel_max_major, cx.scale());
                remaining += subpixel_max_major - max_major;
                max_major
            } else {
                subpixel_max_major
            };

            let space = Space::new(
                self.axis.pack_size(0.0, child_min_minor),
                self.axis.pack_size(max_major, max_minor),
            );

            let size = cx.layout_nth_child(i, space);
            let (major, minor) = self.axis.unpack_size(size);

            major_sum += major;
            minor_sum = minor_sum.max(minor);
        }

        // measure tightly flexible children

        let mut remaining = f32::max(max_major - major_sum, 0.0);

        for (i, &(flex, tight)) in self.flex.iter().enumerate() {
            if flex == 0.0 || !tight {
                continue;
            }

            let per_flex = remaining / flex_sum;
            let subpixel_major = per_flex * flex;

            let major = if cx.settings().render.pixel_align {
                let major = layout::pixel_round(subpixel_major, cx.scale());
                remaining += subpixel_major - major;
                major
            } else {
                subpixel_major
            };

            let space = Space::new(
                self.axis.pack_size(major, child_min_minor),
                self.axis.pack_size(major, max_minor),
            );

            let size = cx.layout_nth_child(i, space);
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
            let size = cx
                .get_nth_child(i)
                .map_or(Size::ZERO, |child| child.cx.size());
            let (child_major, child_minor) = self.axis.unpack_size(size);

            let excess_minor = minor - child_minor;

            let align = match self.align {
                Align::Start | Align::Fill => 0.0,
                Align::Center => excess_minor / 2.0,
                Align::End => excess_minor,
            };

            let offset = self.axis.pack_offset(justify, align);
            cx.place_nth_child(i, offset);

            justify += child_major + gap;
        }

        self.axis.pack_size(major, minor)
    }

    fn update(&mut self, _cx: &mut UpdateCx<'_>, update: Update) {
        if let Update::Children(update) = update {
            match update {
                ChildUpdate::Added(_) => {
                    self.flex.push((0.0, false));
                }

                ChildUpdate::Removed(index) => {
                    self.flex.remove(index);
                }

                ChildUpdate::Replaced(index) => {
                    self.flex[index] = (0.0, false);
                }

                ChildUpdate::Swapped(a, b) => {
                    self.flex.swap(a, b);
                }
            }
        }
    }
}
