use lyon::tessellation::{
    FillVertex, FillVertexConstructor, StrokeVertex, StrokeVertexConstructor,
};
use macroquad::{texture::Texture2D, ui::Vertex};

use crate::utils::Point;

pub trait Shading {
    // TODO: why we need alpha here???
    /// `p`: (x, y) in final coordinate system
    fn new_vertex(&self, p: &Point, alpha: f32) -> Vertex;
    fn texture(&self) -> Option<Texture2D>;
}
pub trait IntoShading {
    type Target: Shading;

    fn into_shading(self) -> Self::Target;
}

pub struct ShadedConstructor<T: Shading> {
    pub shading: T,
    pub alpha: f32,
}
impl<T: Shading> FillVertexConstructor<Vertex> for ShadedConstructor<T> {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        let pos = vertex.position();
        self.shading
            .new_vertex(&Point::new(pos.x, pos.y), self.alpha)
    }
}
impl<T: Shading> StrokeVertexConstructor<Vertex> for ShadedConstructor<T> {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        let pos = vertex.position();
        self.shading
            .new_vertex(&Point::new(pos.x, pos.y), self.alpha)
    }
}

pub struct GradientShading {
    origin: (f32, f32),
    color: macroquad::color::Color,
    vector: (f32, f32),
    color_end: macroquad::color::Color,
}

impl Shading for GradientShading {
    fn new_vertex(&self, p: &Point, alpha: f32) -> Vertex {
        let mut color = {
            let (dx, dy) = (p.x - self.origin.0, p.y - self.origin.1);
            let t = dx * self.vector.0 + dy * self.vector.1;
            macroquad::color::Color {
                r: self.color.r * (1.0 - t) + self.color_end.r * t,
                g: self.color.g * (1.0 - t) + self.color_end.g * t,
                b: self.color.b * (1.0 - t) + self.color_end.b * t,
                a: self.color.a * (1.0 - t) + self.color_end.a * t,
            }
        };
        color.a *= alpha;

        Vertex::new(p.x, p.y, 0., 0., 0., color)
    }

    fn texture(&self) -> Option<Texture2D> {
        None
    }
}

impl IntoShading for macroquad::color::Color {
    type Target = GradientShading;

    fn into_shading(self) -> Self::Target {
        GradientShading {
            origin: (0., 0.),
            color: self,
            vector: (0., 0.),
            color_end: self,
        }
    }
}
