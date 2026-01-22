use std::hash::{Hash, Hasher};

use crate::Color;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub enum Shader {
    Solid(Color),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Blend {
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

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Join {
    Miter,
    Round,
    Bevel,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Stroke {
    pub width: f32,
    pub miter: f32,
    pub join:  Join,
    pub cap:   Cap,
}

impl Default for Stroke {
    fn default() -> Self {
        Self {
            width: 1.0,
            miter: 4.0,
            join:  Join::Miter,
            cap:   Cap::Butt,
        }
    }
}

impl Eq for Stroke {}

impl Hash for Stroke {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.width.to_bits().hash(state);
        self.miter.to_bits().hash(state);
        self.join.hash(state);
        self.cap.hash(state);
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct Paint {
    pub shader: Shader,
    pub blend:  Blend,
    pub stroke: Stroke,
}

impl Default for Paint {
    fn default() -> Self {
        Self {
            shader: Shader::Solid(Color::BLACK),
            blend:  Blend::SrcOver,
            stroke: Stroke::default(),
        }
    }
}

impl From<Color> for Paint {
    fn from(color: Color) -> Self {
        Self {
            shader: Shader::Solid(color),
            ..Default::default()
        }
    }
}
