mod components;
mod scenes;
mod utils;

use comui::{
    component::Component,
    components::label::Label,
    layout::{Layout, LayoutBuilder},
    scene::SceneManager,
    utils::Transform,
    window::Window,
};
use macroquad::{
    color::{BLACK, WHITE},
    miniquad::EventHandler,
    prelude::info,
    window::{clear_background, next_frame},
};
use nalgebra::Matrix3;

use crate::{scenes::test_scene::TestScene, utils::Handler};

struct Main {
    counter: Label,
    scene_stack: SceneManager,
}

impl Default for Main {
    fn default() -> Self {
        Self {
            counter: Label::new("1").with_font_size(24.0).with_color(BLACK),
            scene_stack: SceneManager::new(TestScene::default()),
        }
    }
}

impl Layout for Main {
    fn components(&mut self) -> Vec<(Transform, &mut dyn Component)> {
        LayoutBuilder::new()
            .at_rect((200.0, 200.0, 300.0, 100.0), &mut self.scene_stack)
            .at_rect((150.0, 100.0, 100.0, 50.0), &mut self.counter)
            .build()
    }

    fn after_render(&mut self, _: &Transform, _: &mut Window) {
        self.counter.text = format!("{}", self.scene_stack.scene_stack.len());
    }
}

fn config() -> macroquad::prelude::Conf {
    macroquad::prelude::Conf {
        high_dpi: true,
        sample_count: 4,
        window_title: "Scenes".to_string(),
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
