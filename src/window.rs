use lyon::{
    path::{Path, PathEvent},
    tessellation::{
        BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator,
        VertexBuffers,
    },
};
use macroquad::{
    camera::{Camera2D, set_camera},
    math::vec2,
    prelude::DrawMode,
    texture::Texture2D,
    ui::Vertex,
    window::{get_internal_gl, screen_dpi_scale, screen_height, screen_width},
};

use crate::{
    shading::{ShadedConstructor, Shading},
    utils::Point,
};

#[must_use = "Call `commit` to do the actual drawing"]
pub struct VertexBuilder<S> {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    shading: S,
}

impl<S: Shading> VertexBuilder<S> {
    pub fn new(shading: S) -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
            shading,
        }
    }

    pub fn add(mut self, x: f32, y: f32, alpha: f32) -> Self {
        self.vertices
            .push(self.shading.new_vertex(&Point::new(x, y), alpha));
        self
    }

    pub fn triangle(mut self, a: u16, b: u16, c: u16) -> Self {
        self.indices.push(a);
        self.indices.push(b);
        self.indices.push(c);
        self
    }

    pub fn commit(self) {
        let gl = unsafe { get_internal_gl() }.quad_gl;
        gl.texture(self.shading.texture().as_ref());
        gl.draw_mode(DrawMode::Triangles);
        gl.geometry(&self.vertices, &self.indices);
    }
}

pub struct Window {
    pub pixel_width: u32,
    pub pixel_height: u32,

    /// No need to care about physical PPI
    pub logical_ppi: f32,

    vertex_buffers: VertexBuffers<Vertex, u16>,
    fill_tessellator: FillTessellator,
    fill_options: FillOptions,
    stroke_tessellator: StrokeTessellator,
    stroke_options: StrokeOptions,

    pub(crate) font_system: cosmic_text::FontSystem,
    pub(crate) swash_cache: cosmic_text::SwashCache,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            pixel_width: screen_width() as u32,
            pixel_height: screen_height() as u32,
            logical_ppi: screen_dpi_scale(),

            vertex_buffers: VertexBuffers::new(),
            fill_tessellator: FillTessellator::new(),
            fill_options: FillOptions::tolerance(Self::DEFAULT_TOLERANCE),
            stroke_tessellator: StrokeTessellator::new(),
            stroke_options: StrokeOptions::default(),
            font_system: cosmic_text::FontSystem::new(),
            swash_cache: cosmic_text::SwashCache::new(),
        }
    }
}

impl Window {
    const DEFAULT_TOLERANCE: f32 = 0.3;

    fn set_tolerance(&mut self, tol: f32) {
        self.fill_options.tolerance = tol / (self.pixel_width as f32);
    }

    fn emit_lyon(&mut self, texture: Option<Texture2D>) {
        let gl = unsafe { get_internal_gl() }.quad_gl;
        gl.texture(texture.as_ref());
        gl.draw_mode(DrawMode::Triangles);
        gl.geometry(
            &std::mem::take(&mut self.vertex_buffers.vertices),
            &std::mem::take(&mut self.vertex_buffers.indices),
        );
    }

    /// `f` does the actual drawing
    fn draw_lyon<S: Shading>(
        &mut self,
        shading: S,
        alpha: f32,
        f: impl FnOnce(&mut Self, ShadedConstructor<S>),
    ) {
        self.set_tolerance(Self::DEFAULT_TOLERANCE);
        let tex = shading.texture();
        f(self, ShadedConstructor { shading, alpha });
        self.emit_lyon(tex);
    }

    pub fn set_camera(&self) {
        set_camera(&Camera2D {
            zoom: vec2(
                self.pixel_width as f32 / 2.0,
                -(self.pixel_height as f32 / 2.0),
            ),
            viewport: None,
            ..Default::default()
        });
    }

    pub fn vertex_builder<S: Shading>(&self, shading: S) -> VertexBuilder<S> {
        VertexBuilder::new(shading)
    }

    /// Drawing 2 triangles:
    ///   - a, b, c
    ///   - a, c, d
    ///
    ///  quad: [a, b, c, d]
    pub fn fill_quad(&self, quad: [Point; 4], shading: impl Shading) {
        let [a, b, c, d] = quad;
        // TODO: how to set alpha here????
        self.vertex_builder(shading)
            .add(a.x, a.y, 1.0)
            .add(b.x, b.y, 1.0)
            .add(c.x, c.y, 1.0)
            .add(d.x, d.y, 1.0)
            .triangle(0, 1, 2)
            .triangle(0, 2, 3)
            .commit();
    }

    pub fn fill_path(
        &mut self,
        path: impl IntoIterator<Item = PathEvent>,
        shading: impl Shading,
        alpha: f32,
    ) {
        self.draw_lyon(shading, alpha, |this, shading| {
            this.fill_tessellator
                .tessellate(
                    path,
                    &this.fill_options,
                    &mut BuffersBuilder::new(&mut this.vertex_buffers, shading),
                )
                .unwrap();
        });
    }

    pub fn stroke_path(&mut self, path: &Path, alpha: f32, thickness: f32, shading: impl Shading) {
        self.draw_lyon(shading, alpha, |this, shaded| {
            this.stroke_options.line_width = thickness;
            this.stroke_tessellator
                .tessellate_path(
                    path,
                    &this.stroke_options,
                    &mut BuffersBuilder::new(&mut this.vertex_buffers, shaded),
                )
                .unwrap();
        });
    }
}
