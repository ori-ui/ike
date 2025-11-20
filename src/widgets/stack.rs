use core::f32;

use crate::{
    BuildCx, LayoutCx, Offset, Size, Space, Widget, WidgetId, context::UpdateCx, widget::Update,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn unpack_size(self, size: Size) -> (f32, f32) {
        match self {
            Axis::Horizontal => (size.width, size.height),
            Axis::Vertical => (size.height, size.width),
        }
    }

    fn pack_size(self, major: f32, minor: f32) -> Size {
        match self {
            Axis::Horizontal => Size::new(major, minor),
            Axis::Vertical => Size::new(minor, major),
        }
    }

    fn pack_offset(self, major: f32, minor: f32) -> Offset {
        match self {
            Axis::Horizontal => Offset::new(major, minor),
            Axis::Vertical => Offset::new(minor, major),
        }
    }
}

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
    pub fn new(cx: &mut impl BuildCx) -> WidgetId<Self> {
        cx.insert(Self {
            axis:    Axis::Vertical,
            justify: Justify::Start,
            align:   Align::Center,
            gap:     0.0,

            flex: Vec::new(),
        })
    }

    pub fn set_flex(
        cx: &mut impl BuildCx,
        id: WidgetId<Self>,
        index: usize,
        flex: f32,
        tight: bool,
    ) {
        cx.get_mut(id).flex[index] = (flex, tight);
        cx.request_layout(id);
    }

    pub fn get_flex(cx: &impl BuildCx, id: WidgetId<Self>, index: usize) -> (f32, bool) {
        cx.get(id).flex[index]
    }

    pub fn set_axis(cx: &mut impl BuildCx, id: WidgetId<Self>, axis: Axis) {
        cx.get_mut(id).axis = axis;
        cx.request_layout(id);
    }

    pub fn set_justify(cx: &mut impl BuildCx, id: WidgetId<Self>, justify: Justify) {
        cx.get_mut(id).justify = justify;
        cx.request_layout(id);
    }

    pub fn set_align(cx: &mut impl BuildCx, id: WidgetId<Self>, align: Align) {
        cx.get_mut(id).align = align;
        cx.request_layout(id);
    }

    pub fn set_gap(cx: &mut impl BuildCx, id: WidgetId<Self>, gap: f32) {
        cx.get_mut(id).gap = gap;
        cx.request_layout(id);
    }
}

impl Widget for Stack {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let (min_major, mut min_minor) = self.axis.unpack_size(space.min);
        let (max_major, max_minor) = self.axis.unpack_size(space.max);

        if let Align::Fill = self.align {
            min_minor = max_minor;
        }

        let total_gap = self.gap * (cx.children().len() - 1) as f32;

        let mut flex_sum = 0.0;
        let mut major_sum = 0.0;
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

            let size = cx.layout_child(i, space);
            let (major, minor) = self.axis.unpack_size(size);

            major_sum += major;
            minor_sum = minor_sum.max(minor);
        }

        // measure expanding children

        let remaining = f32::max(max_major - total_gap - major_sum, 0.0);
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

            let size = cx.layout_child(i, space);
            let (major, minor) = self.axis.unpack_size(size);

            major_sum += major;
            minor_sum = minor_sum.max(minor);
        }

        // measure tightly flexible children

        let remaining = f32::max(max_major - total_gap - major_sum, 0.0);
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

            let size = cx.layout_child(i, space);
            let (major, minor) = self.axis.unpack_size(size);

            major_sum += major;
            minor_sum = minor_sum.max(minor);
        }

        let major = f32::clamp(major_sum + total_gap, min_major, max_major);
        let minor = f32::clamp(minor_sum, min_minor, max_minor);

        let excess_major = major - major_sum + total_gap;

        let count = cx.children().len();
        let gap = match self.justify {
            Justify::Start | Justify::Center | Justify::End => self.gap,
            Justify::SpaceBetween => excess_major / (count - 1) as f32,
            Justify::SpaceAround => excess_major / count as f32,
            Justify::SpaceEvenly => excess_major / (count + 1) as f32,
        };

        let mut justify = match self.justify {
            Justify::Start | Justify::SpaceBetween => 0.0,
            Justify::Center => excess_major / 2.0,
            Justify::End => excess_major,
            Justify::SpaceAround => gap / 2.0,
            Justify::SpaceEvenly => gap,
        };

        for i in 0..count {
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
        match update {
            Update::ChildAdded(_) => {
                self.flex.push((0.0, false));
            }

            Update::ChildRemoved(i) => {
                self.flex.remove(i);
            }

            Update::ChildrenSwapped(a, b) => {
                self.flex.swap(a, b);
            }

            _ => {}
        }
    }

    fn accepts_pointer() -> bool {
        false
    }
}
