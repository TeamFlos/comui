use comui::{
    component::Component,
    components::{button::QuadButton, label::Label},
    input::subscriber_id,
    layout::{Layout, LayoutBuilder},
    shading::IntoShading,
    utils::Transform,
    window::Window,
};
use lyon::{
    geom::traits::Transformation,
    math::Box2D,
    path::{Path, Winding, builder::BorderRadii},
};
use macroquad::{
    color::{self, Color, WHITE},
    input::{
        is_mouse_button_down, mouse_position, utils::repeat_all_miniquad_input, MouseButton, Touch, TouchPhase
    },
    math::vec2,
    miniquad::EventHandler,
    prelude::info,
    window::{clear_background, next_frame, screen_dpi_scale},
};
use nalgebra::Matrix3;

struct MyTransfrom(pub Transform);

impl Transformation<f32> for MyTransfrom {
    fn transform_point(&self, p: lyon::geom::Point<f32>) -> lyon::geom::Point<f32> {
        let p = nalgebra::Point2::new(p.x, p.y);
        let p = self.0.transform_point(&p);
        lyon::geom::Point::new(p.x, p.y)
    }

    fn transform_vector(&self, v: lyon::geom::Vector<f32>) -> lyon::geom::Vector<f32> {
        let v = nalgebra::Vector2::new(v.x, v.y);
        let v = self.0.transform_vector(&v);
        lyon::geom::Vector::new(v.x, v.y)
    }
}

#[derive(Default)]
enum Colors {
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

    fn next(&mut self) {
        *self = match self {
            Colors::Gray => Colors::Red,
            Colors::Red => Colors::Green,
            Colors::Green => Colors::Blue,
            Colors::Blue => Colors::Gray,
        }
    }
}

struct MyFancyBtn {
    pub inner: QuadButton,
    /// In local coord system
    radius: f32,
    color: Colors,
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

struct Main {
    clicked: bool,
    fancy_btn: MyFancyBtn,
    label: Label
}

impl Default for Main {
    fn default() -> Self {
        Self {
            clicked: false,
            fancy_btn: MyFancyBtn::default(),
            label: Label::new("A FANCY Button, 😲这么强？！")
                .with_font_size(20.0)
                .with_color(color::BLACK),
        }
    }
}

impl Layout for Main {
    fn components(&mut self) -> Vec<(Transform, &mut dyn Component)> {
        LayoutBuilder::new()
            .at_rect(
                (
                    100.0 + if self.clicked { 150.0 } else { 0.0 },
                    100.0,
                    100.0,
                    100.0,
                ),
                &mut self.fancy_btn,
            )
            .at_rect((100.0, 250.0, 100.0, 100.0), &mut self.label)
            .build()
    }

    fn before_render(&mut self, _tr: &Transform, _target: &mut Window) {
        if self.fancy_btn.inner.triggered {
            self.clicked = !self.clicked;
            self.fancy_btn.inner.triggered = false;
            self.fancy_btn.color.next();
        }
    }
}

fn button_to_id(button: MouseButton) -> u64 {
    u64::MAX
        - match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
            MouseButton::Unknown => 3,
        }
}

#[derive(Default)]
struct Handler {
    pub touches: Vec<Touch>,
}

impl EventHandler for Handler {
    fn draw(&mut self) {}

    fn update(&mut self) {
        repeat_all_miniquad_input(self, subscriber_id());
        if is_mouse_button_down(MouseButton::Left) {
            self.touches.push(Touch {
                id: button_to_id(MouseButton::Left),
                phase: TouchPhase::Moved,
                position: mouse_position().into(),
            });
        }
    }

    fn mouse_button_down_event(&mut self, button: MouseButton, x: f32, y: f32) {
        self.touches.push(Touch {
            id: button_to_id(button),
            phase: TouchPhase::Started,
            position: vec2(x, y) / screen_dpi_scale(),
        });
    }

    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        self.touches.push(Touch {
            id: button_to_id(button),
            phase: TouchPhase::Ended,
            position: vec2(x, y) / screen_dpi_scale(),
        });
    }
}

fn config() -> macroquad::prelude::Conf {
    macroquad::prelude::Conf {
        high_dpi: true,
        sample_count: 4,
        window_title: "A FANCY Button".to_string(),
        ..Default::default()
    }
}

#[macroquad::main(config)]
async fn main() {
    let mut handler = Handler::default();
    let mut main_view = Main::default();
    loop {
        clear_background(WHITE);
        handler.update();
        let touches = std::mem::take(&mut handler.touches);
        for touch in &touches {
            if let Err(e) = main_view.touch(touch) {
                info!("Error handling touch: {:?}", e);
            }
        }
        main_view.render(&Matrix3::identity(), &mut Window::default());
        next_frame().await
    }
}
