use comui::{
    component::Component,
    components::button::QuadButton,
    layout::{Layout, LayoutBuilder},
    window::Window,
};
use macroquad::{
    color::WHITE,
    window::{clear_background, next_frame},
};
use nalgebra::Matrix3;

struct Main {
    window: Window,
    main: Layout,
}

impl Default for Main {
    fn default() -> Self {
        let window = Window::default();
        Self {
            window,
            main: LayoutBuilder::new()
                .at_rect((0.0, 0.0, 100.0, 100.0), Box::new(QuadButton::default()))
                .build(),
        }
    }
}

#[macroquad::main("Underflow")]
async fn main() {
    loop {
        clear_background(WHITE);
        let mut main_view = Main::default();
        main_view
            .main
            .render(&Matrix3::identity(), &mut main_view.window);
        next_frame().await
    }
}
