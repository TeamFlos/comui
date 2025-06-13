use macroquad::prelude::Touch;

use crate::component::Component;
use crate::utils::Transform;
use crate::window::Window;

/// Notice that here the transform: tr: touch.position -> new_touch.position
fn transform_touch(touch: &Touch, tr: &Transform) -> Touch {
    let pos = nalgebra::Point2::new(touch.position.x, touch.position.y);
    let new_pos = tr.transform_point(&pos);
    Touch {
        id: touch.id,
        phase: touch.phase,
        position: macroquad::math::Vec2::new(new_pos.x, new_pos.y),
    }
}

#[derive(Default)]
pub struct LayoutBuilder {
    /// (Transform, Child)
    ///
    /// Transform is a 3x3 matrix, used for child_coord -> parent_coord
    inner: Vec<(Transform, Box<dyn Component>)>,
}

impl LayoutBuilder {
    #[must_use = "Call `build` to build the layout"]
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    #[must_use = "Call `build` to build the layout"]
    /// Add a child with a transform.
    pub fn at_transform(self, tr: Transform, child: Box<dyn Component>) -> Self {
        let mut new_inner = self.inner;
        new_inner.push((tr, child));
        Self { inner: new_inner }
    }

    #[must_use = "Call `build` to build the layout"]
    /// Rectangle representation:
    ///
    /// (center_x, center_y, width, height)
    ///
    /// The coordinate system in the rectangle is normalized, i.e. (0, 0) is the center,
    /// and (-0.5, -0.5) is the bottom-left corner, (0.5, 0.5) is the top-right corner.
    pub fn at_rect(self, rect: (f32, f32, f32, f32), child: Box<dyn Component>) -> Self {
        let (cx, cy, w, h) = rect;
        let tr = Transform::new_translation(&nalgebra::Vector2::new(cx, cy))
            * Transform::new_nonuniform_scaling(&nalgebra::Vector2::new(w, h));
        self.at_transform(tr, child)
    }

    #[must_use = "Call `render` to do the actual drawing"]
    pub fn build(self) -> Layout {
        Layout { inner: self.inner }
    }
}

// ! TODO: this should be a trait
pub struct Layout {
    inner: Vec<(Transform, Box<dyn Component>)>,
}

impl Component for Layout {
    fn touch(&mut self, touch: &Touch) -> anyhow::Result<bool> {
        for (child_tr, child) in self.inner.iter_mut() {
            if let Some(inv_tr) = child_tr.try_inverse() {
                if child.touch(&transform_touch(touch, &inv_tr))? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn render(&self, tr: &Transform, target: &mut Window) {
        for (child_tr, child) in &self.inner {
            let tr = tr * child_tr;
            child.render(&tr, target);
        }
    }
}
