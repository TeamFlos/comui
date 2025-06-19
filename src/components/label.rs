use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use macroquad::{
    color::Color,
    math::vec2,
    miniquad::{MipmapFilterMode, TextureFormat, TextureKind, TextureParams, TextureWrap},
    texture::{FilterMode, Image, draw_texture_ex},
    window::get_internal_gl,
};
use tracing::{Level, instrument, span};

use crate::{
    shading::IntoShading,
    utils::{Point, cosmic_color_to_macroquad_color, macroquad_color_to_cosmic_color},
    window::{VertexBuilder, Window},
};

pub struct Label {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    /// The width for the area to show the label.
    /// Set it to `None` for infinite size.
    pub area_width: Option<f32>,
    /// The height for the area to show the label.
    /// Set it to `None` for infinite size.
    pub area_height: Option<f32>,
    pub color: Color,
}
impl Default for Label {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_size: 16.,
            line_height: 20.,
            area_height: None,
            area_width: None,
            color: Color::from_rgba(255, 255, 255, 255), // Default white color
        }
    }
}
impl Label {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_string(),
            ..Default::default()
        }
    }

    pub fn with_font_size(mut self, font_size: f32) -> Self {
        self.font_size = font_size;
        self
    }

    pub fn with_line_height(mut self, line_height: f32) -> Self {
        self.line_height = line_height;
        self
    }

    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_preferred_width(mut self, width: f32) -> Self {
        self.area_width = Some(width);
        self
    }
    pub fn with_preferred_height(mut self, height: f32) -> Self {
        self.area_height = Some(height);
        self
    }
    #[instrument(skip(self, target))]
    pub fn render_text(&self, target: &mut Window, origin: Point) {
        let metrics = Metrics::relative(
            self.font_size * target.logical_ppi,
            self.line_height / self.font_size,
        );
        let font_system = &mut target.font_system;
        let mut buffer = Buffer::new(font_system, metrics);
        // Borrow buffer together with the font system for more convenient method calls
        let mut buffer_borrowed = buffer.borrow_with(font_system);
        // Set a size for the text buffer, in pixels
        buffer_borrowed.set_size(
            self.area_width.map(|w| w * target.logical_ppi),
            self.area_height.map(|h| h * target.logical_ppi),
        );
        // Attributes indicate what font to choose
        let attrs = Attrs::new();
        // Add some text!
        buffer_borrowed.set_text(&self.text, &attrs, Shaping::Advanced);
        // Perform shaping as desired
        buffer_borrowed.shape_until_scroll(true);
        let logical_ppi = target.logical_ppi;

        let span = span!(Level::DEBUG, "Draw buffers");
        let _enter = span.enter();
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), 1.0);

                let glyph_color = match glyph.color_opt {
                    Some(some) => some,
                    None => macroquad_color_to_cosmic_color(self.color),
                };
                let img = target
                    .swash_cache
                    .get_image(&mut target.font_system, physical_glyph.cache_key)
                    .as_ref()
                    .expect("no target glyph");

                let texture = match img.content {
                    cosmic_text::SwashContent::Color => macroquad::texture::Texture2D::from_rgba8(
                        img.placement.width as u16,
                        img.placement.height as u16,
                        &img.data,
                    ),
                    cosmic_text::SwashContent::Mask => {
                        let ctx = unsafe { get_internal_gl() }.quad_context;
                        let id = ctx.new_texture_from_data_and_format(
                            &img.data,
                            TextureParams {
                                kind: TextureKind::Texture2D,
                                width: img.placement.width,
                                height: img.placement.height,
                                format: TextureFormat::Alpha,
                                wrap: TextureWrap::Clamp,
                                min_filter: FilterMode::Linear,
                                mag_filter: FilterMode::Linear,
                                mipmap_filter: MipmapFilterMode::None,
                                allocate_mipmaps: false,
                                sample_count: 1,
                            },
                        );
                        macroquad::texture::Texture2D::from_miniquad_texture(id)
                    }
                    _ => todo!(),
                };
                draw_texture_ex(
                    &texture,
                    physical_glyph.x as f32 + img.placement.left as f32,
                    physical_glyph.y as f32 + run.line_y - img.placement.top as f32,
                    cosmic_color_to_macroquad_color(glyph_color),
                    macroquad::prelude::DrawTextureParams {
                        dest_size: Some(vec2(
                            img.placement.width as f32,
                            img.placement.height as f32,
                        )),
                        ..Default::default()
                    },
                );
            }
        }
    }
}
impl crate::component::Component for Label {
    fn render(&mut self, tr: &crate::utils::Transform, target: &mut Window) {
        self.render_text(target, tr.transform_point(&Point::origin()));
    }
    fn touch(&mut self, _touch: &macroquad::prelude::Touch) -> anyhow::Result<bool> {
        Ok(false)
    }
}
