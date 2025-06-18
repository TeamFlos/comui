use cosmic_text::{Attrs, Buffer, Metrics, Shaping};
use macroquad::color::Color;
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
        let metrics = Metrics::new(self.font_size, self.line_height);
        let font_system = &mut target.font_system;
        let mut buffer = Buffer::new(font_system, metrics);
        // Borrow buffer together with the font system for more convenient method calls
        let mut buffer = buffer.borrow_with(font_system);
        // Set a size for the text buffer, in pixels
        buffer.set_size(self.area_width, self.area_height);
        // Attributes indicate what font to choose
        let attrs = Attrs::new();
        // Add some text!
        buffer.set_text(&self.text, &attrs, Shaping::Advanced);
        // Perform shaping as desired
        buffer.shape_until_scroll(true);
        let logical_ppi = target.logical_ppi;

        let span = span!(Level::DEBUG, "Draw buffers");
        let _enter = span.enter();
        // Draw the buffer
        buffer.draw(
            &mut target.swash_cache,
            macroquad_color_to_cosmic_color(self.color),
            move |x, y, w, h, color| {
                // We need this workaround to make the borrow checker happy,
                // as drawing and buffer.borrow_with cannot be used together.
                // They are borrowing different parts of `Window`.
                let mut x = x as f32;
                let mut y = y as f32;
                let mut w = w as f32;
                let mut h = h as f32;
                x += origin.x;
                y += origin.y;

                // high-dpi support
                // TODO: this syntax is too ugly.
                x *= logical_ppi;
                y *= logical_ppi;
                w *= logical_ppi;
                h *= logical_ppi;

                {
                    VertexBuilder::new(cosmic_color_to_macroquad_color(color).into_shading())
                        .add(x, y, 1.0)
                        .add(x, y + h, 1.0)
                        .add(x + w, y + h, 1.0)
                        .add(x + w, y, 1.0)
                        .triangle(2, 1, 0)
                        .triangle(0, 2, 3)
                        .commit();
                };
            },
        );
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
