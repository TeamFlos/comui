use comui::{
    component::Component,
    components::button::QuadButton,
    input::subscriber_id,
    layout::{Layout, LayoutBuilder},
    utils::Transform,
    window::Window,
};
use macroquad::{
    color::WHITE,
    input::{
        MouseButton, Touch, TouchPhase, is_mouse_button_down, mouse_position,
        utils::repeat_all_miniquad_input,
    },
    math::vec2,
    miniquad::EventHandler,
    prelude::info,
    window::{clear_background, next_frame},
};
use nalgebra::Matrix3;

#[derive(Default)]
struct Main {
    clicked: bool,
    test_btn: QuadButton,
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
                &mut self.test_btn,
            )
            .build()
    }

    fn before_render(&mut self) {
        if self.test_btn.triggered {
            self.clicked = !self.clicked;
            self.test_btn.triggered = false;
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
            position: vec2(x, y),
        });
    }

    fn mouse_button_up_event(&mut self, button: MouseButton, x: f32, y: f32) {
        self.touches.push(Touch {
            id: button_to_id(button),
            phase: TouchPhase::Ended,
            position: vec2(x, y),
        });
    }
}

#[macroquad::main("Test")]
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
