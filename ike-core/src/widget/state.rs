use std::{cell::Cell, rc::Rc};

use crate::{Affine, Clip, CursorIcon, Size, Space, Widget, WidgetId, WindowId, world::Widgets};

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct WidgetFlags: u16 {
        const IS_HOVERED = 1 << 0;
        const IS_FOCUSED = 1 << 1;
        const IS_ACTIVE  = 1 << 2;

        const HAS_HOVERED = 1 << 3;
        const HAS_FOCUSED = 1 << 4;
        const HAS_ACTIVE  = 1 << 5;

        const IS_STASHED  = 1 << 6;
        const IS_DISABLED = 1 << 7;

        const NEEDS_COMPOSE = 1 << 8;
        const NEEDS_ANIMATE = 1 << 9;
        const NEEDS_LAYOUT  = 1 << 10;
        const NEEDS_DRAW    = 1 << 11;

        const ACCEPTS_POINTER = 1 << 12;
        const ACCEPTS_FOCUS   = 1 << 13;
        const ACCEPTS_TEXT    = 1 << 14;
    }
}

impl WidgetFlags {
    pub fn new<T: Widget>() -> Self {
        let mut flags = Self::empty();

        flags.set(
            Self::ACCEPTS_POINTER,
            T::accepts_pointer(),
        );

        flags.set(Self::ACCEPTS_FOCUS, T::accepts_focus());
        flags.set(Self::ACCEPTS_TEXT, T::accepts_text());

        flags
    }

    pub fn reset(&mut self) {
        self.remove(Self::HAS_HOVERED | Self::HAS_FOCUSED | Self::HAS_ACTIVE);

        self.set(
            Self::HAS_HOVERED,
            self.contains(Self::IS_HOVERED),
        );

        self.set(
            Self::HAS_FOCUSED,
            self.contains(Self::IS_FOCUSED),
        );

        self.set(
            Self::HAS_ACTIVE,
            self.contains(Self::IS_ACTIVE),
        );
    }

    pub fn propagate_down(&mut self, child: Self) {
        if !child.contains(Self::IS_STASHED) {
            self.insert(child.intersection(
                Self::HAS_HOVERED
                    | Self::HAS_FOCUSED
                    | Self::HAS_ACTIVE
                    | Self::NEEDS_COMPOSE
                    | Self::NEEDS_ANIMATE
                    | Self::NEEDS_LAYOUT
                    | Self::NEEDS_DRAW,
            ));
        }
    }

    pub fn propagate_up(self, child: &mut Self) {
        child.insert(self.intersection(Self::IS_STASHED | Self::IS_DISABLED));
    }
}

#[derive(Debug)]
pub struct WidgetHierarchy {
    pub(crate) window:   Option<WindowId>,
    pub(crate) parent:   Option<WidgetId>,
    pub(crate) children: Rc<Vec<WidgetId>>,
    pub(crate) flags:    Cell<WidgetFlags>,
}

impl WidgetHierarchy {
    pub fn new<T: Widget>() -> Self {
        Self {
            window:   None,
            parent:   None,
            children: Rc::new(Vec::new()),
            flags:    Cell::new(WidgetFlags::new::<T>()),
        }
    }

    pub fn propagate_down(&self, widgets: &Widgets) {
        self.flags.update(|mut flags| {
            flags.reset();
            flags
        });

        for &child in self.children.iter() {
            let Some(child) = widgets.get_hierarchy(child) else {
                continue;
            };

            self.flags.update(|mut flags| {
                flags.propagate_down(child.flags.get());
                flags
            });
        }
    }

    pub fn children_mut(&mut self) -> &mut Vec<WidgetId> {
        Rc::make_mut(&mut self.children)
    }

    pub fn request_compose(&self) {
        self.flags.update(|flags| {
            // add the NEEDS_COMPOSE flag
            flags.union(WidgetFlags::NEEDS_COMPOSE)
        })
    }

    pub fn request_animate(&self) {
        self.flags.update(|flags| {
            // add the NEEDS_ANIMATE flag
            flags.union(WidgetFlags::NEEDS_ANIMATE)
        })
    }

    pub fn request_layout(&self) {
        self.flags.update(|flags| {
            // add the NEEDS_LAYOUT flag
            flags.union(WidgetFlags::NEEDS_LAYOUT)
        })
    }

    pub fn request_draw(&self) {
        self.flags.update(|flags| {
            // add the NEEDS_DRAW flag
            flags.union(WidgetFlags::NEEDS_DRAW)
        })
    }

    pub fn mark_composed(&self) {
        self.flags.update(|flags| {
            // remove the NEEDS_COMPOSE flags
            flags.difference(WidgetFlags::NEEDS_COMPOSE)
        })
    }

    pub fn mark_animated(&self) {
        self.flags.update(|flags| {
            // remove the NEEDS_ANIMATE flags
            flags.difference(WidgetFlags::NEEDS_ANIMATE)
        })
    }

    pub fn mark_laid_out(&self) {
        self.flags.update(|flags| {
            // remove the NEEDS_LAYOUT flags
            flags.difference(WidgetFlags::NEEDS_LAYOUT)
        })
    }

    pub fn mark_drawn(&self) {
        self.flags.update(|flags| {
            // remove the NEEDS_DRAW flags
            flags.difference(WidgetFlags::NEEDS_DRAW)
        })
    }

    pub fn set_hovered(&self, is_hovered: bool) {
        self.flags.update(|mut flags| {
            flags.set(WidgetFlags::IS_HOVERED, is_hovered);
            flags
        })
    }

    pub fn set_focused(&self, is_focused: bool) {
        self.flags.update(|mut flags| {
            flags.set(WidgetFlags::IS_FOCUSED, is_focused);
            flags
        })
    }

    pub fn set_active(&self, is_active: bool) {
        self.flags.update(|mut flags| {
            flags.set(WidgetFlags::IS_ACTIVE, is_active);
            flags
        })
    }

    pub fn set_stashed(&self, is_stashed: bool) {
        self.flags.update(|mut flags| {
            flags.set(WidgetFlags::IS_STASHED, is_stashed);
            flags
        })
    }

    pub fn set_disabled(&self, is_disabled: bool) {
        self.flags.update(|mut flags| {
            flags.set(WidgetFlags::IS_DISABLED, is_disabled);
            flags
        })
    }

    pub fn is_hovered(&self) -> bool {
        self.flags.get().contains(WidgetFlags::IS_HOVERED)
    }

    pub fn is_focused(&self) -> bool {
        self.flags.get().contains(WidgetFlags::IS_FOCUSED)
    }

    pub fn is_active(&self) -> bool {
        self.flags.get().contains(WidgetFlags::IS_ACTIVE)
    }

    pub fn is_stashed(&self) -> bool {
        self.flags.get().contains(WidgetFlags::IS_STASHED)
    }

    pub fn is_disabled(&self) -> bool {
        self.flags.get().contains(WidgetFlags::IS_DISABLED)
    }

    pub fn has_hovered(&self) -> bool {
        self.flags.get().contains(WidgetFlags::HAS_HOVERED)
    }

    pub fn has_focused(&self) -> bool {
        self.flags.get().contains(WidgetFlags::HAS_FOCUSED)
    }

    pub fn has_active(&self) -> bool {
        self.flags.get().contains(WidgetFlags::HAS_ACTIVE)
    }

    pub fn needs_compose(&self) -> bool {
        self.flags.get().contains(WidgetFlags::NEEDS_COMPOSE)
    }

    pub fn needs_animate(&self) -> bool {
        self.flags.get().contains(WidgetFlags::NEEDS_ANIMATE)
    }

    pub fn needs_layout(&self) -> bool {
        self.flags.get().contains(WidgetFlags::NEEDS_LAYOUT)
    }

    pub fn needs_draw(&self) -> bool {
        self.flags.get().contains(WidgetFlags::NEEDS_DRAW)
    }

    /// Get whether the widget is currently accepting pointer hover.
    ///
    /// This is true if both the [`Widget`] accepts pointer and it isn't `stashed`.
    pub fn accepts_pointer(&self) -> bool {
        self.flags.get().contains(WidgetFlags::ACCEPTS_POINTER) && !self.is_stashed()
    }

    /// Get whether the widget is currently accepting focus.
    ///
    /// This is true if the [`Widget`] accepts focus and is not `stashed` or `disabled`.
    pub fn accepts_focus(&self) -> bool {
        self.flags.get().contains(WidgetFlags::ACCEPTS_FOCUS)
            && !self.is_stashed()
            && !self.is_disabled()
    }

    /// Get whether the widget is currently accepting text input.
    ///
    /// This is true if the [`Widget`] accepts focus and is not `stashed` or `disabled`.
    pub fn accepts_text(&self) -> bool {
        self.flags.get().contains(WidgetFlags::ACCEPTS_TEXT)
            && !self.is_stashed()
            && !self.is_disabled()
    }
}

#[derive(Debug)]
pub struct WidgetState {
    pub(crate) id:               WidgetId,
    pub(crate) transform:        Affine,
    pub(crate) global_transform: Affine,
    pub(crate) size:             Size,
    pub(crate) previous_space:   Option<Space>,
    pub(crate) cursor:           CursorIcon,

    pub(crate) is_subpixel:  bool,
    pub(crate) stable_draws: u32,
    pub(crate) clip:         Option<Clip>,

    pub(crate) tracing_span: tracing::Span,
    #[allow(dead_code, reason = "used for debug purposes")]
    pub(crate) short_name:   &'static str,
    #[allow(dead_code, reason = "used for debug purposes")]
    pub(crate) type_name:    &'static str,
}

impl WidgetState {
    pub(crate) fn new<T: Widget>(id: WidgetId) -> Self {
        #[allow(clippy::redundant_field_names)]
        Self {
            id: id,

            transform:        Affine::IDENTITY,
            global_transform: Affine::IDENTITY,
            size:             Size::new(0.0, 0.0),
            previous_space:   None,
            cursor:           CursorIcon::Default,

            is_subpixel:  false,
            stable_draws: 0,
            clip:         None,

            tracing_span: tracing::error_span!(
                "Widget",
                r#type = Self::short_type_name::<T>()
            ),
            short_name:   Self::short_type_name::<T>(),
            type_name:    std::any::type_name::<T>(),
        }
    }

    fn short_type_name<T>() -> &'static str {
        let name = std::any::type_name::<T>();
        name.split('<')
            .next()
            .unwrap_or(name)
            .split("::")
            .last()
            .unwrap_or(name)
    }
}
