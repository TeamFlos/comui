use std::hash::{DefaultHasher, Hash, Hasher};

use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use macroquad::{
    color::Color,
    math::vec2,
    prelude::warn,
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
    /// (0, 0) is the top-left corner of the texture,
    /// (0.5, 0.5) is the center of the texture,
    /// (1, 1) is the bottom-right corner of the texture.
    pub texture_align: (f32, f32),
    /// The alignment of the text.
    pub text_align: Align,
    cached_buffer: Option<(u64, Buffer, (f32, f32))>,
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
            text_align: Align::Left,
            texture_align: (0.5, 0.5),
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
        self.text_align = align;
        self
    }

    pub fn with_texture_align(mut self, align: (f32, f32)) -> Self {
        self.texture_align = align;
        self
    }

    #[instrument(skip(self, target))]
    pub fn render_text(&mut self, target: &mut Window, origin: Point) {
        let (hash, buffer, (text_block_w, text_block_h)) = self
            .cached_buffer
            .take()
            .and_then(|(hash, buffer, size)| {
                if hash == self.state_hash() {
                    Some((hash, buffer, size))
                } else {
                    None
                }
            })
            .unwrap_or(self.layout_text(target));
        let span = span!(Level::DEBUG, "Draw buffers");
        let _enter = span.enter();
        let text_block = {
            let (w, h) = buffer.size();
            vec2(
                w.unwrap_or(text_block_w) / target.logical_ppi,
                h.unwrap_or(text_block_h) / target.logical_ppi,
            )
        };
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
                let target_size =
                    vec2(placement.width as f32, placement.height as f32) / target.logical_ppi;
                draw_texture_ex(
                    &texture,
                    (physical_glyph.x + placement.left) as f32 / target.logical_ppi + origin.x
                        - self.texture_align.0 * text_block.x,
                    ((physical_glyph.y - placement.top) as f32 + run.line_y) / target.logical_ppi
                        + origin.y
                        - self.texture_align.1 * text_block.y,
                    self.color,
                    DrawTextureParams {
                        dest_size: Some(target_size),
                        source: Some(rect),
                        ..Default::default()
                    },
                );
            }
        }
        self.cached_buffer = Some((hash, buffer, (text_block_w, text_block_h)));
    }

    pub fn latest_layout(&mut self, target: &mut Window) -> &mut Buffer {
        self.cached_buffer = Some(
            self.cached_buffer
                .take()
                .and_then(|(hash, buffer, size)| {
                    if hash == self.state_hash() {
                        Some((hash, buffer, size))
                    } else {
                        None
                    }
                })
                .unwrap_or(self.layout_text(target)),
        );
        &mut self.cached_buffer.as_mut().unwrap().1
    }

    pub fn computed_height(&self) -> f32 {
        self.cached_buffer.as_ref().map_or(0.0, |(_, _, (_, h))| *h)
    }

    #[instrument(skip(self, target))]
    /// Returns:
    /// - `u64`: a hash of the current state of the label
    /// - `Buffer`: the cosmic text buffer containing the text layout
    /// - `(f32, f32)`: the width and height of the text block in pixels
    fn layout_text(&self, target: &mut Window) -> (u64, Buffer, (f32, f32)) {
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
            Some(self.text_align),
        );
        // Perform shaping as desired
        buffer_borrowed.shape_until_scroll(true);
        // Get the size of the text block in pixels
        let size = {
            let mut idx = 0;
            let mut max_w = 0.;
            while let Some(line) = buffer_borrowed.line_layout(idx) {
                idx += 1;
                line.iter().for_each(|l| {
                    max_w = f32::max(max_w, l.w);
                });
            }
            let max_h = buffer_borrowed
                .layout_runs()
                .last()
                .map_or(0.0, |run| run.line_y);
            (max_w, max_h)
        };

        (self.state_hash(), buffer, size)
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
        self.text_align.to_string().hash(&mut hasher);
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
