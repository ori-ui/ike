use crate::{
    Affine, Blend, Builder, Canvas, Color, DrawCx, LayoutCx, Offset, Paint, Shader, Size, Space,
    Svg, SvgData, Widget, WidgetMut,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Picturable {
    Svg(Svg),
}

impl From<Svg> for Picturable {
    fn from(svg: Svg) -> Self {
        Picturable::Svg(svg)
    }
}

impl From<SvgData> for Picturable {
    fn from(data: SvgData) -> Self {
        Picturable::Svg(Svg::from(data))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fit {
    Contain,
    Cover,
    Fill,
    #[default]
    None,
}

pub struct Picture {
    contents: Picturable,
    fit:      Fit,
    color:    Option<Color>,
}

impl Picture {
    pub fn new(cx: &mut impl Builder, contents: Picturable) -> WidgetMut<'_, Self> {
        cx.build_widget(Self {
            contents,
            fit: Fit::None,
            color: None,
        })
        .finish()
    }

    pub fn set_contents(this: &mut WidgetMut<Self>, contents: Picturable) {
        this.widget.contents = contents;
        this.cx.request_layout();
    }

    pub fn set_fit(this: &mut WidgetMut<Self>, fit: Fit) {
        this.widget.fit = fit;
        this.cx.request_layout();
    }

    pub fn set_color(this: &mut WidgetMut<Self>, color: Option<Color>) {
        this.widget.color = color;
        this.cx.request_draw();
    }
}

impl Widget for Picture {
    fn layout(&mut self, cx: &mut LayoutCx<'_>, space: Space) -> Size {
        let size = match self.contents {
            Picturable::Svg(ref svg) => cx.measure_svg(svg),
        };

        if size.has_zero_area() {
            return space.min;
        }

        match self.fit {
            Fit::Contain => scale_to_fit(size, space),
            Fit::Cover => space.max,
            Fit::Fill => space.max,
            Fit::None => space.constrain(size),
        }
    }

    fn draw(&mut self, cx: &mut DrawCx<'_>, canvas: &mut dyn Canvas) {
        let size = match self.contents {
            Picturable::Svg(ref svg) => canvas.painter().measure_svg(svg),
        };

        let sx = cx.size().width / size.width;
        let sy = cx.size().height / size.height;

        let (sx, sy) = match self.fit {
            Fit::Fill => (sx, sy),
            Fit::None => (1.0, 1.0),
            Fit::Contain => {
                let s = f32::min(sx, sy);
                (s, s)
            }
            Fit::Cover => {
                let s = f32::max(sx, sy);
                (s, s)
            }
        };

        let offset = Offset {
            x: (cx.size().width - size.width * sx) / 2.0,
            y: (cx.size().height - size.height * sy) / 2.0,
        };

        match self.contents {
            Picturable::Svg(ref svg) => {
                canvas.layer(&mut |canvas| {
                    let transform = Affine::scale_translate(sx, sy, offset);
                    canvas.transform(transform, &mut |canvas| {
                        canvas.draw_svg(svg)
                    });

                    if let Some(color) = self.color {
                        let paint = Paint {
                            shader: Shader::Solid(color),
                            blend: Blend::SrcIn,
                            ..Default::default()
                        };

                        canvas.fill(&paint);
                    }
                });
            }
        }
    }
}

fn scale_to_fit(size: Size, space: Space) -> Size {
    if space.contains(size) {
        return size;
    }

    let aspect_ratio = f32::abs(size.width / size.height);

    let min_min = space.min.width / space.min.height;
    let max_min = space.max.width / space.min.height;
    let min_max = space.min.width / space.max.height;
    let max_max = space.max.width / space.max.height;

    if aspect_ratio < min_max {
        Size {
            width:  space.min.width,
            height: space.max.height,
        }
    } else if aspect_ratio > max_min {
        Size {
            width:  space.max.width,
            height: space.min.height,
        }
    } else if aspect_ratio < min_min {
        if size.width < space.min.width {
            Size {
                width:  space.min.width,
                height: space.min.width / aspect_ratio,
            }
        } else if aspect_ratio > max_max {
            Size {
                width:  space.max.width,
                height: space.max.width / aspect_ratio,
            }
        } else {
            Size {
                width:  space.max.height * aspect_ratio,
                height: space.max.height,
            }
        }
    } else if size.width < space.min.width {
        Size {
            width:  space.min.height * aspect_ratio,
            height: space.min.height,
        }
    } else if aspect_ratio < max_max {
        Size {
            width:  space.max.height * aspect_ratio,
            height: space.max.height,
        }
    } else {
        Size {
            width:  space.max.width,
            height: space.max.width / aspect_ratio,
        }
    }
}
