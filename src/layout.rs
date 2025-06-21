use macroquad::prelude::Touch;

use crate::component::Component;
use crate::utils::Transform;
use crate::window::Window;

const LAYOUT_DEBUG: bool = option_env!("COMUI_LAYOUT_DEBUG").is_some();

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

#[must_use = "Call `build` to build the layout"]
#[derive(Default)]
pub struct LayoutBuilder<'a> {
    /// (Transform, Child)
    ///
    /// Transform is a 3x3 matrix, used for child_coord -> parent_coord
    inner: Vec<(Transform, &'a mut dyn Component)>,
}

impl<'a> LayoutBuilder<'a> {
    pub fn new() -> Self {
        Self { inner: vec![] }
    }

    /// Add a child with a transform.
    pub fn at_transform(self, tr: Transform, child: &'a mut dyn Component) -> Self {
        let mut new_inner = self.inner;
        new_inner.push((tr, child));
        Self { inner: new_inner }
    }

    /// Rectangle representation:
    ///
    /// (center_x, center_y, width, height)
    ///
    /// The coordinate system in the rectangle is normalized, i.e. (0, 0) is the center,
    /// and (-0.5, -0.5) is the bottom-left corner, (0.5, 0.5) is the top-right corner.
    pub fn at_rect(self, rect: (f32, f32, f32, f32), child: &'a mut dyn Component) -> Self {
        let (cx, cy, w, h) = rect;
        let tr = Transform::new_translation(&nalgebra::Vector2::new(cx, cy))
            * Transform::new_nonuniform_scaling(&nalgebra::Vector2::new(w, h));
        self.at_transform(tr, child)
    }

    #[must_use = "Call `render` to do the actual drawing"]
    pub fn build(self) -> Vec<(Transform, &'a mut dyn Component)> {
        self.inner
    }
}

pub trait Layout {
    fn components(&mut self) -> Vec<(Transform, &mut dyn Component)>;

    #[allow(unused_variables)]
    /// Called before rendering children.
    fn before_render(&mut self, tr: &Transform, target: &mut Window) {}
    #[allow(unused_variables)]
    /// Called after rendering children.
    fn after_render(&mut self, tr: &Transform, target: &mut Window) {}
}

impl<T: Layout> Component for T {
    fn render(&mut self, tr: &Transform, target: &mut Window) {
        self.before_render(tr, target);
        for (child_tr, child) in self.components() {
            let tr = tr * child_tr;
            child.render(&tr, target);
            #[cfg(feature = "layout-debug")]
            {
                // TODO: simplify this
                use lyon::{
                    geom::traits::Transformation,
                    math::Box2D,
                    path::{Builder, Winding},
                };

                use crate::shading::IntoShading;
                let mut builder = Builder::new();
                builder.add_rectangle(
                    &Box2D::new((-0.5, -0.5).into(), (0.5, 0.5).into()),
                    Winding::Positive,
                );

                struct MyTransform(Transform);

                impl Transformation<f32> for MyTransform {
                    fn transform_point(&self, p: lyon::geom::Point<f32>) -> lyon::geom::Point<f32> {
                        let point = nalgebra::Point2::new(p.x, p.y);
                        let new_point = self.0.transform_point(&point);
                        lyon::geom::Point::new(new_point.x, new_point.y)
                    }

                    fn transform_vector(
                        &self,
                        v: lyon::geom::Vector<f32>,
                    ) -> lyon::geom::Vector<f32> {
                        let vector = nalgebra::Vector2::new(v.x, v.y);
                        let new_vector = self.0.transform_vector(&vector);
                        lyon::geom::Vector::new(new_vector.x, new_vector.y)
                    }
                }
                if LAYOUT_DEBUG {
                    target.stroke_path(
                        &builder.build().transformed(&MyTransform(tr)),
                        1.0,
                        1.0,
                        macroquad::color::RED.into_shading(),
                    );
                }
            }
        }
        self.after_render(tr, target);
    }

    fn touch(&mut self, touch: &Touch) -> anyhow::Result<bool> {
        for (child_tr, child) in self.components() {
            if let Some(inv_tr) = child_tr.try_inverse() {
                if child.touch(&transform_touch(touch, &inv_tr))? {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
