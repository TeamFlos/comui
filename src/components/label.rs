use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use macroquad::{
    color::Color, math::vec2, prelude::debug, text::draw_text, texture::{draw_texture_ex, DrawTextureParams}
};
use tracing::{Level, instrument, span};

use crate::{
    utils::Point,
    window::Window,
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

        let span = span!(Level::DEBUG, "Draw buffers");
        let _enter = span.enter();
        for run in buffer.layout_runs() {
            for glyph in run.glyphs.iter() {
                let physical_glyph = glyph.physical((0., 0.), 1.0);
                // cache if needed
                target.font_atlas.cache_glyph(
                    physical_glyph.cache_key,
                    &mut target.swash_cache,
                    &mut target.font_system,
                );
                let rect = target
                    .font_atlas
                    .get_glyph(physical_glyph.cache_key)
                    .unwrap();
                let placement = target
                    .font_atlas
                    .get_placement(physical_glyph.cache_key)
                    .unwrap();
                let texture = macroquad::texture::Texture2D::from_miniquad_texture(
                    target.font_atlas.texture(),
                );
                draw_texture_ex(
                    &texture,
                    (physical_glyph.x + placement.left) as f32,
                    (physical_glyph.y - placement.top) as f32 + run.line_y,
                    self.color,
                    DrawTextureParams {
                        dest_size: Some(vec2(placement.width as f32, placement.height as f32)),
                        source: Some(rect),
                        ..Default::default()
                    }
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
