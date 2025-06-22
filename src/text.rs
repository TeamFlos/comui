//! [`Atlas`], [`Sprite`] and [`SpriteKey`] are copied from macroquad source code,
//! licensed under MIT OR APACHE-2.0.
use cosmic_text::{CacheKey, FontSystem, Placement, SwashCache};
use guillotiere::{
    AllocId, Allocation, AtlasAllocator,
    euclid::{Box2D, Size2D, UnknownUnit},
    point2, size2,
};
use lru::LruCache;
use macroquad::{
    math::Rect,
    miniquad::native::gl,
    texture::{Image, Texture2D, render_target},
};
use tracing::trace;

/// For weird rect like 1x0
enum CAllocation {
    Real(Allocation),
    Fake,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CAllocId {
    Real(AllocId),
    Fake,
}

impl CAllocation {
    pub fn rect(&self) -> Box2D<i32, UnknownUnit> {
        match self {
            CAllocation::Real(alloc) => alloc.rectangle,
            CAllocation::Fake => Box2D {
                min: point2(0, 0),
                max: point2(0, 0),
            },
        }
    }

    pub fn id(&self) -> CAllocId {
        match self {
            CAllocation::Real(alloc) => CAllocId::Real(alloc.id),
            CAllocation::Fake => CAllocId::Fake,
        }
    }
}

fn alloc_or_evict(
    allocator: &mut AtlasAllocator,
    cache: &mut LruCache<CacheKey, (CAllocId, Placement)>,
    size: Size2D<i32, UnknownUnit>,
) -> CAllocation {
    if size.width <= 0 || size.height <= 0 {
        return CAllocation::Fake;
    }
    for _ in 0..Atlas::MAX_ALLOC_ATTEMPTS {
        if let Some(alloc) = allocator.allocate(size) {
            return CAllocation::Real(alloc);
        }
        trace!(
            "Failed to allocate space of {}x{} in the atlas, evicting one item",
            size.width, size.height
        );
        if let Some((_, (CAllocId::Real(id), _))) = cache.pop_lru() {
            allocator.deallocate(id);
        }
    }
    // TODO: handle this better
    println!("Current cache size: {}", cache.len());
    panic!(
        "Failed to allocate space of {}x{} in the atlas after {} attempts, maybe the atlas is too small?",
        size.width,
        size.height,
        Atlas::MAX_ALLOC_ATTEMPTS
    );
}

pub struct Atlas {
    allocator: AtlasAllocator,
    pub texture: Texture2D,
    cache: LruCache<CacheKey, (CAllocId, Placement)>,
}

impl Default for Atlas {
    fn default() -> Self {
        let mut length: i32 = 0;
        unsafe {
            gl::glGetIntegerv(gl::GL_MAX_TEXTURE_SIZE, &mut length);
        }
        let size = size2(length, length);
        let length = length as u32;
        println!("Creating a new atlas with size: {}x{}", length, length);
        let texture = render_target(length, length).texture;
        Self {
            allocator: AtlasAllocator::new(size),
            texture,
            cache: LruCache::unbounded(),
        }
    }
}

impl Atlas {
    const MAX_ALLOC_ATTEMPTS: usize = 32;
    const ALLOC_GAP: i32 = 1;

    // TODO: `SwashCache` here is not necessary, since we always use `get_image_uncached`
    pub fn cache_glyph(
        &mut self,
        key: CacheKey,
        cache: &mut SwashCache,
        font_system: &mut FontSystem,
    ) -> Option<CAllocId> {
        if let Some((alloc_id, _)) = self.cache.get(&key) {
            return Some(*alloc_id);
        }

        let image = cache.get_image_uncached(font_system, key)?;

        let cosmic_text::Placement {
            left,
            top,
            width,
            height,
        } = image.placement;

        let alloc = alloc_or_evict(
            &mut self.allocator,
            &mut self.cache,
            size2(
                width as i32 + 2 * Self::ALLOC_GAP,
                height as i32 + 2 * Self::ALLOC_GAP,
            ),
        );

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
        self.texture.update_part(
            &quad_image,
            alloc.rect().min.x + Self::ALLOC_GAP,
            alloc.rect().min.y + Self::ALLOC_GAP,
            alloc.rect().width() - 2 * Self::ALLOC_GAP,
            alloc.rect().height() - 2 * Self::ALLOC_GAP,
        );

        self.cache.push(
            key,
            (
                alloc.id(),
                Placement {
                    left,
                    top,
                    width,
                    height,
                },
            ),
        );

        Some(alloc.id())
    }

    pub fn get_glyph(&mut self, key: CacheKey) -> Option<Rect> {
        self.cache.get(&key).map(|(alloc_id, _)| {
            if let CAllocId::Real(alloc_id) = alloc_id {
                let box2d = self.allocator[*alloc_id].to_f32();
                Rect {
                    x: box2d.min.x + Self::ALLOC_GAP as f32,
                    y: box2d.min.y + Self::ALLOC_GAP as f32,
                    w: box2d.width() - 2.0 * Self::ALLOC_GAP as f32,
                    h: box2d.height() - 2.0 * Self::ALLOC_GAP as f32,
                }
            } else {
                Rect::new(0.0, 0.0, 0.0, 0.0)
            }
        })
    }

    pub fn get_placement(&mut self, key: CacheKey) -> Option<Placement> {
        self.cache.get(&key).map(|(_, placement)| *placement)
    }
}
