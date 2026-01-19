#![warn(clippy::unwrap_used)]

mod axis;
mod build;
mod canvas;
mod color;
mod context;
mod curve;
mod debug;
mod event;
mod image;
mod layout;
mod math;
mod painter;
mod record;
mod svg;
mod text;
mod transition;
mod widget;
mod window;
mod world;

pub(crate) mod passes;

pub mod widgets;

pub use axis::Axis;
pub use build::Builder;
pub use canvas::{Blend, Canvas, Cap, Clip, Join, Paint, PixelRect, Shader, Stroke};
pub use color::Color;
pub use context::{ComposeCx, DrawCx, EventCx, LayoutCx, MutCx, RefCx, UpdateCx};
pub use curve::{Curve, CurveData, CurveSegment, CurveVerb, Fill, WeakCurve};
pub use debug::DebugSettings;
pub use event::{
    CursorIcon, Gesture, ImeEvent, Key, KeyEvent, KeyPressEvent, Modifiers, NamedKey, PanGesture,
    Pointer, PointerButton, PointerButtonEvent, PointerEvent, PointerId, PointerMoveEvent,
    PointerPropagate, PointerScrollEvent, Propagate, ScrollDelta, TapGesture, TextEvent,
    TextPasteEvent, Touch, TouchEvent, TouchId, TouchMoveEvent, TouchPressEvent, TouchPropagate,
    TouchSettings,
};
pub use layout::{BorderWidth, CornerRadius, Padding, pixel_ceil, pixel_floor, pixel_round};
pub use math::{Affine, Matrix, Offset, Point, Rect, Size, Space};
pub use painter::Painter;
pub use record::{RecordSettings, Recorder, Recording, RecordingData, WeakRecording};
pub use svg::{Svg, SvgData, WeakSvg};
pub use text::{
    FontStretch, FontStyle, FontWeight, GlyphCluster, Paragraph, TextAlign, TextDirection,
    TextLayoutLine, TextStyle, TextWrap, WeakParagraph,
};
pub use transition::{Interpolate, Transition, TransitionCurve, Transitioned};
pub use widget::{AnyWidgetId, ChildUpdate, Update, Widget, WidgetId, WidgetState};
pub use window::{Layer, LayerId, Window, WindowId, WindowSizing};
pub use world::{
    AnyWidget, GetError, ImeSignal, RenderSettings, Settings, Signal, WidgetMut, WidgetRef,
    WindowUpdate, World,
};
