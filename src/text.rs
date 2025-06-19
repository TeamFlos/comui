//! [`Atlas`], [`Sprite`] and [`SpriteKey`] are copied from macroquad source code,
//! licensed under MIT OR APACHE-2.0.
use cosmic_text::{CacheKey, FontSystem, Placement, SwashCache};
use macroquad::{
    color::Color,
    math::Rect,
    miniquad::{self, TextureId},
    texture::Image,
    window::get_internal_gl,
};

use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct Sprite {
    pub rect: Rect,
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum SpriteKey {
    Texture(miniquad::TextureId),
    Id(u64),
}
pub struct Atlas {
    texture: miniquad::TextureId,
    pub(crate) image: Image,
    pub sprites: HashMap<SpriteKey, Sprite>,
    cursor_x: u16,
    cursor_y: u16,
    max_line_height: u16,

    pub dirty: bool,

    filter: miniquad::FilterMode,

    unique_id: u64,
}

impl Drop for Atlas {
    fn drop(&mut self) {
        let ctx = unsafe { get_internal_gl() }.quad_context;
        ctx.delete_texture(self.texture);
    }
}

impl Atlas {
    // pixel gap between glyphs in the atlas
    const GAP: u16 = 2;
    // well..
    const UNIQUENESS_OFFSET: u64 = 100000;

    pub fn new() -> Atlas {
        let ctx = unsafe { get_internal_gl() }.quad_context;
        let image = Image::gen_image_color(512, 512, Color::new(0.0, 0.0, 0.0, 0.0));
        let texture = ctx.new_texture_from_rgba8(image.width, image.height, &image.bytes);
        ctx.texture_set_filter(
            texture,
            miniquad::FilterMode::Nearest,
            miniquad::MipmapFilterMode::None,
        );

        Atlas {
            image,
            texture,
            cursor_x: 0,
            cursor_y: 0,
            dirty: false,
            max_line_height: 0,
            sprites: HashMap::new(),
            filter: miniquad::FilterMode::Nearest,
            unique_id: Self::UNIQUENESS_OFFSET,
        }
    }

    pub fn new_unique_id(&mut self) -> SpriteKey {
        self.unique_id += 1;

        SpriteKey::Id(self.unique_id)
    }

    pub fn set_filter(&mut self, filter_mode: miniquad::FilterMode) {
        let ctx = unsafe { get_internal_gl() }.quad_context;
        self.filter = filter_mode;
        ctx.texture_set_filter(self.texture, filter_mode, miniquad::MipmapFilterMode::None);
    }

    pub fn get(&self, key: SpriteKey) -> Option<Sprite> {
        self.sprites.get(&key).cloned()
    }

    pub const fn width(&self) -> u16 {
        self.image.width
    }

    pub const fn height(&self) -> u16 {
        self.image.height
    }

    pub fn texture(&mut self) -> miniquad::TextureId {
        let ctx = unsafe { get_internal_gl() }.quad_context;
        if self.dirty {
            self.dirty = false;
            let (texture_width, texture_height) = ctx.texture_size(self.texture);
            if texture_width != self.image.width as _ || texture_height != self.image.height as _ {
                ctx.delete_texture(self.texture);
                self.texture = ctx.new_texture_from_rgba8(
                    self.image.width,
                    self.image.height,
                    &self.image.bytes[..],
                );
                ctx.texture_set_filter(self.texture, self.filter, miniquad::MipmapFilterMode::None);
            }
            ctx.texture_update(self.texture, &self.image.bytes);
        }

        self.texture
    }

    pub fn get_uv_rect(&self, key: SpriteKey) -> Option<Rect> {
        let ctx = unsafe { get_internal_gl() }.quad_context;
        self.get(key).map(|sprite| {
            let (w, h) = ctx.texture_size(self.texture);

            Rect::new(
                sprite.rect.x / w as f32,
                sprite.rect.y / h as f32,
                sprite.rect.w / w as f32,
                sprite.rect.h / h as f32,
            )
        })
    }

    pub fn cache_sprite(&mut self, key: SpriteKey, sprite: Image) {
        let (width, height) = (sprite.width as usize, sprite.height as usize);

        let x = if self.cursor_x + (width as u16) < self.image.width {
            if height as u16 > self.max_line_height {
                self.max_line_height = height as u16;
            }
            let res = self.cursor_x + Self::GAP;
            self.cursor_x += width as u16 + Self::GAP * 2;
            res
        } else {
            self.cursor_y += self.max_line_height + Self::GAP * 2;
            self.cursor_x = width as u16 + Self::GAP;
            self.max_line_height = height as u16;
            Self::GAP
        };
        let y = self.cursor_y;

        // texture bounds exceeded
        if y + sprite.height > self.image.height || x + sprite.width > self.image.width {
            // reset glyph cache state
            let sprites = self.sprites.drain().collect::<Vec<_>>();
            self.cursor_x = 0;
            self.cursor_y = 0;
            self.max_line_height = 0;

            let old_image = self.image.clone();

            // increase font texture size
            // note: if we tried to fit gigantic texture into a small atlas,
            // new_width will still be not enough. But its fine, it will
            // be regenerated on the recursion call.
            let new_width = self.image.width * 2;
            let new_height = self.image.height * 2;

            self.image =
                Image::gen_image_color(new_width, new_height, Color::new(0.0, 0.0, 0.0, 0.0));

            // recache all previously cached symbols
            for (key, sprite) in sprites {
                let image = old_image.sub_image(sprite.rect);
                self.cache_sprite(key, image);
            }

            // cache the new sprite
            self.cache_sprite(key, sprite);
        } else {
            self.dirty = true;

            for j in 0..height {
                for i in 0..width {
                    self.image.set_pixel(
                        x as u32 + i as u32,
                        y as u32 + j as u32,
                        sprite.get_pixel(i as u32, j as u32),
                    );
                }
            }

            self.sprites.insert(
                key,
                Sprite {
                    rect: Rect::new(x as f32, y as f32, width as f32, height as f32),
                },
            );
        }
    }
}

impl Default for Atlas {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct FontAtlas {
    characters: HashMap<CacheKey, (SpriteKey, Placement)>,
    pub(crate) atlas: Atlas,
}

impl FontAtlas {
    pub fn cache_glyph(
        &mut self,
        key: CacheKey,
        cache: &mut SwashCache,
        font_system: &mut FontSystem,
    ) {
        if self.characters.contains_key(&key) {
            return;
        }
        let Some(image) = cache.get_image_uncached(font_system, key) else {
            return;
        };
        let cosmic_text::Placement {
            left,
            top,
            width,
            height,
        } = image.placement;

        // the following block is copied from bevy, licensed under MIT OR APACHE-2.0
        let data = match image.content {
            cosmic_text::SwashContent::Mask => image
                .data
                .iter()
                .flat_map(|a| [255, 255, 255, *a])
                .collect(),
            cosmic_text::SwashContent::Color => image.data,
            cosmic_text::SwashContent::SubpixelMask => {
                // TODO: implement
                todo!()
            }
        };
        let quad_image = Image {
            bytes: data,
            width: width as u16,
            height: height as u16,
        };
        let new_key = self.atlas.new_unique_id();
        self.atlas.cache_sprite(new_key, quad_image);
        self.characters.insert(
            key,
            (
                new_key,
                cosmic_text::Placement {
                    left,
                    top,
                    width,
                    height,
                },
            ),
        );
    }
    pub fn get_glyph(&self, key: CacheKey) -> Option<Rect> {
        self.characters
            .get(&key)
            .and_then(|(sprite_key, _)| self.atlas.get(*sprite_key).map(|s| s.rect))
    }
    pub fn get_placement(&self, key: CacheKey) -> Option<Placement> {
        self.characters.get(&key).map(|c| c.1)
    }
    pub fn texture(&mut self) -> TextureId {
        self.atlas.texture()
    }
}
