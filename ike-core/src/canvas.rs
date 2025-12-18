use std::{
    hash::{Hash, Hasher},
    sync::{Arc, Weak},
};

use crate::{
    Affine, BorderWidth, Color, CornerRadius, Offset, Painter, Paragraph, Rect, Size, Svg,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum Shader {
    Solid(Color),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BlendMode {
    Clear,
    Src,
    Dst,
    SrcOver,
    DstOver,
    SrcIn,
    DstIn,
    SrcATop,
    DstATop,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct Paint {
    pub shader: Shader,
    pub blend:  BlendMode,
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self {
            shader: Shader::Solid(color),
            blend:  BlendMode::SrcOver,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Clip {
    Rect(Rect, CornerRadius),
}

impl From<Rect> for Clip {
    fn from(rect: Rect) -> Self {
        Self::Rect(rect, CornerRadius::all(0.0))
    }
}

impl From<Rect> for Option<Clip> {
    fn from(rect: Rect) -> Self {
        Some(Clip::from(rect))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Recording {
    data: Arc<()>,
}

impl Default for Recording {
    fn default() -> Self {
        Self::new()
    }
}

impl Recording {
    pub fn new() -> Self {
        Self { data: Arc::new(()) }
    }

    pub fn downgrade(this: &Self) -> WeakRecording {
        WeakRecording {
            data: Arc::downgrade(&this.data),
        }
    }
}

#[derive(Clone, Debug)]
pub struct WeakRecording {
    data: Weak<()>,
}

impl WeakRecording {
    pub fn upgrade(&self) -> Option<Recording> {
        Some(Recording {
            data: self.data.upgrade()?,
        })
    }

    pub fn strong_count(&self) -> usize {
        self.data.strong_count()
    }
}

impl PartialEq for WeakRecording {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.data, &other.data)
    }
}

impl Eq for WeakRecording {}

impl Hash for WeakRecording {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.as_ptr().hash(state);
    }
}

pub trait Canvas {
    fn painter(&mut self) -> &mut dyn Painter;

    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas));

    fn layer(&mut self, f: &mut dyn FnMut(&mut dyn Canvas));

    fn record(&mut self, size: Size, f: &mut dyn FnMut(&mut dyn Canvas)) -> Recording;

    fn clip(&mut self, clip: &Clip, f: &mut dyn FnMut(&mut dyn Canvas));

    fn fill(&mut self, paint: &Paint);

    fn draw_rect(&mut self, rect: Rect, corners: CornerRadius, paint: &Paint);

    fn draw_border(&mut self, rect: Rect, width: BorderWidth, radius: CornerRadius, paint: &Paint);

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset);

    fn draw_svg(&mut self, svg: &Svg);

    fn draw_recording(&mut self, recording: &Recording);
}

pub struct TrackedCanvas<'a> {
    pub canvas:     &'a mut dyn Canvas,
    pub layers:     u32,
    pub clips:      u32,
    pub fills:      u32,
    pub rects:      u32,
    pub borders:    u32,
    pub texts:      u32,
    pub svgs:       u32,
    pub recordings: u32,
}

impl<'a> TrackedCanvas<'a> {
    pub fn new(canvas: &'a mut dyn Canvas) -> Self {
        Self {
            canvas,
            layers: 0,
            clips: 0,
            fills: 0,
            rects: 0,
            borders: 0,
            texts: 0,
            svgs: 0,
            recordings: 0,
        }
    }

    fn wrap(
        &mut self,
        f: &mut dyn FnMut(&mut dyn Canvas),
        g: impl FnOnce(&mut dyn Canvas, &mut dyn FnMut(&mut dyn Canvas)),
    ) {
        g(self.canvas, &mut |canvas| {
            let mut tracked = TrackedCanvas::new(canvas);
            f(&mut tracked);
            self.layers += tracked.layers;
            self.clips += tracked.clips;
            self.fills += tracked.fills;
            self.rects += tracked.rects;
            self.borders += tracked.borders;
            self.texts += tracked.texts;
            self.svgs += tracked.svgs;
            self.recordings += tracked.recordings;
        })
    }
}

impl Canvas for TrackedCanvas<'_> {
    fn painter(&mut self) -> &mut dyn Painter {
        self.canvas.painter()
    }

    fn transform(&mut self, affine: Affine, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.wrap(f, |canvas, f| {
            canvas.transform(affine, f)
        })
    }

    fn layer(&mut self, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.wrap(f, |canvas, f| canvas.layer(f))
    }

    fn record(&mut self, size: Size, f: &mut dyn FnMut(&mut dyn Canvas)) -> Recording {
        self.canvas.record(size, f)
    }

    fn clip(&mut self, clip: &Clip, f: &mut dyn FnMut(&mut dyn Canvas)) {
        self.wrap(f, |canvas, f| canvas.clip(clip, f))
    }

    fn fill(&mut self, paint: &Paint) {
        self.fills += 1;
        self.canvas.fill(paint);
    }

    fn draw_rect(&mut self, rect: Rect, corners: CornerRadius, paint: &Paint) {
        self.rects += 1;
        self.canvas.draw_rect(rect, corners, paint);
    }

    fn draw_border(&mut self, rect: Rect, width: BorderWidth, radius: CornerRadius, paint: &Paint) {
        self.borders += 1;
        self.canvas.draw_border(rect, width, radius, paint);
    }

    fn draw_text(&mut self, paragraph: &Paragraph, max_width: f32, offset: Offset) {
        self.texts += 1;
        self.canvas.draw_text(paragraph, max_width, offset);
    }

    fn draw_svg(&mut self, svg: &Svg) {
        self.svgs += 1;
        self.canvas.draw_svg(svg);
    }

    fn draw_recording(&mut self, recording: &Recording) {
        self.recordings += 1;
        self.canvas.draw_recording(recording);
    }
}
