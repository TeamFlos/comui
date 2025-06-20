use comui::{input::subscriber_id, utils::Transform};
use lyon::geom::traits::Transformation;
use macroquad::{
    input::{
        MouseButton, Touch, TouchPhase, is_mouse_button_down, mouse_position,
        utils::repeat_all_miniquad_input,
    },
    math::vec2,
    miniquad::EventHandler,
    window::screen_dpi_scale,
};

pub struct MyTransfrom(pub Transform);

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
pub struct Handler {
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
