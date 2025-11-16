pub struct CornerRadius {
    pub top_right: f32,
    pub top_left: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl CornerRadius {
    pub const fn all(radius: f32) -> Self {
        Self {
            top_right: radius,
            top_left: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }
}

pub struct BorderWidth {
    pub right: f32,
    pub top: f32,
    pub left: f32,
    pub bottom: f32,
}
