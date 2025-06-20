use comui::{
    component::Component,
    components::button::QuadButton,
    layout::{Layout, LayoutBuilder},
    shading::IntoShading,
    utils::Transform,
    window::Window,
};
use lyon::{
    math::Box2D,
    path::{Path, Winding, builder::BorderRadii},
};
use macroquad::color::{self, Color};

use crate::utils::MyTransfrom;

#[derive(Default)]
pub enum Colors {
    #[default]
    Gray,
    Red,
    Green,
    Blue,
}

impl Colors {
    fn color(&self) -> Color {
        match self {
            Colors::Gray => color::GRAY,
            Colors::Red => color::RED,
            Colors::Green => color::GREEN,
            Colors::Blue => color::BLUE,
        }
    }

    pub fn next(&mut self) {
        *self = match self {
            Colors::Gray => Colors::Red,
            Colors::Red => Colors::Green,
            Colors::Green => Colors::Blue,
            Colors::Blue => Colors::Gray,
        }
    }
}

pub struct MyFancyBtn {
    pub inner: QuadButton,
    /// In local coord system
    radius: f32,

    pub color: Colors,
}

impl Default for MyFancyBtn {
    fn default() -> Self {
        Self {
            inner: QuadButton::default(),
            radius: 0.1,
            color: Colors::default(),
        }
    }
}

impl Layout for MyFancyBtn {
    fn components(&mut self) -> Vec<(Transform, &mut dyn Component)> {
        LayoutBuilder::new()
            .at_rect((0.0, 0.0, 1.0, 1.0), &mut self.inner)
            .build()
    }

    fn before_render(&mut self, tr: &Transform, target: &mut Window) {
        let size = 1.0
            - 0.04 * {
                let t = if self.inner.pressed {
                    self.inner.press_start_at.elapsed().as_secs_f32() / 0.15
                } else {
                    1.0 - self.inner.release_start_at.elapsed().as_secs_f32() / 0.1
                }
                .clamp(0.0, 1.0);
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
            };
        let path = {
            let mut builder = Path::builder();
            builder.add_rounded_rectangle(
                &Box2D::new(
                    lyon::math::point(-0.5 * size, -0.5 * size),
                    lyon::math::point(0.5 * size, 0.5 * size),
                ),
                &BorderRadii::new(self.radius),
                Winding::Positive,
            );
            builder.build()
        };
        target.fill_path(
            &path.transformed(&MyTransfrom(*tr)),
            self.color.color().into_shading(),
            1.0,
        );
    }
}
