use std::hash::{DefaultHasher, Hash, Hasher};

use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use macroquad::{
    color::Color,
    math::vec2,
    texture::{DrawTextureParams, draw_texture_ex},
};
use tracing::{Level, instrument, span};

use crate::{utils::Point, window::Window};

pub use cosmic_text::Align;

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
    pub align: Align,
    cached_buffer: Option<(u64, Buffer)>,
    computed_height: f32,
}

#[derive(Hash)]
struct HashingKey {
    pub font_size: u32,
    pub line_height: u32,
    /// The width for the area to show the label.
    /// Set it to `None` for infinite size.
    pub area_width: Option<u32>,
    /// The height for the area to show the label.
    /// Set it to `None` for infinite size.
    pub area_height: Option<u32>,
    // pub color: Color,
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
            cached_buffer: None,
            computed_height: 0.0,
            align: Align::Left,
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
    pub fn with_align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    #[instrument(skip(self, target))]
    pub fn render_text(&mut self, target: &mut Window, origin: Point) {
        let (hash, buffer) = self
            .cached_buffer
            .take()
            .and_then(|(hash, buffer)| {
                if hash == self.state_hash() {
                    Some((hash, buffer))
                } else {
                    None
                }
            })
            .unwrap_or(self.layout_text(target));
        let span = span!(Level::DEBUG, "Draw buffers");
        let _enter = span.enter();
        let mut last = None;
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
                    (physical_glyph.x + placement.left) as f32 / target.logical_ppi + origin.x,
                    ((physical_glyph.y - placement.top) as f32 + run.line_y) / target.logical_ppi
                        + origin.y,
                    self.color,
                    DrawTextureParams {
                        dest_size: Some(
                            vec2(placement.width as f32, placement.height as f32)
                                / target.logical_ppi,
                        ),
                        source: Some(rect),
                        ..Default::default()
                    },
                );
            }
            last = Some(run)
        }
        self.computed_height = last
            .map(|run| run.line_top + run.line_height)
            .unwrap_or_default();
        self.cached_buffer = Some((hash, buffer));
    }

    pub fn latest_layout(&mut self, target: &mut Window) -> &mut Buffer {
        self.cached_buffer = Some(
            self.cached_buffer
                .take()
                .and_then(|(hash, buffer)| {
                    if hash == self.state_hash() {
                        Some((hash, buffer))
                    } else {
                        None
                    }
                })
                .unwrap_or(self.layout_text(target)),
        );
        &mut self.cached_buffer.as_mut().unwrap().1
    }

    pub fn computed_height(&self) -> f32 {
        self.computed_height
    }

    #[instrument(skip(self, target))]
    fn layout_text(&self, target: &mut Window) -> (u64, Buffer) {
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
        buffer_borrowed.set_rich_text(
            [(self.text.as_str(), attrs.clone())],
            &attrs,
            Shaping::Advanced,
            Some(self.align),
        );
        // Perform shaping as desired
        buffer_borrowed.shape_until_scroll(true);
        (self.state_hash(), buffer)
    }

    fn state_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        let hashing_key = HashingKey {
            font_size: self.font_size.to_bits(),
            line_height: self.line_height.to_bits(),
            area_height: self.area_height.map(|i| i.to_bits()),
            area_width: self.area_width.map(|i| i.to_bits()),
        };
        self.text.hash(&mut hasher);
        hashing_key.hash(&mut hasher);
        self.align.to_string().hash(&mut hasher);
        hasher.finish()
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
