use comui::{
    layout::{Layout, LayoutBuilder},
    scene::{NextScene, Scene},
};

use crate::components::button::MyFancyBtn;

#[derive(Default)]
pub struct TestScene {
    pub back_btn: MyFancyBtn,
    pub create_new_scene_btn: MyFancyBtn,
    

    next_scene: Option<NextScene>,
}
impl Scene for TestScene {
    fn next_scene(&mut self) -> Option<NextScene> {
        self.next_scene.take()
    }
}

impl Layout for TestScene {
    fn components(
        &mut self,
    ) -> Vec<(
        comui::utils::Transform,
        &mut dyn comui::component::Component,
    )> {
        LayoutBuilder::new()
            .at_rect((-0.25, 0.0, 0.4, 0.8), &mut self.back_btn)
            .at_rect((0.25, 0.0, 0.4, 0.8), &mut self.create_new_scene_btn)
            .build()
    }

    fn after_render(&mut self, _: &comui::utils::Transform, _: &mut comui::window::Window) {
        if self.back_btn.inner.triggered {
            self.next_scene = Some(NextScene::Pop);
            self.back_btn.inner.triggered = false;
        }

        if self.create_new_scene_btn.inner.triggered {
            self.next_scene = Some(NextScene::Push(Box::new(TestScene::default())));
            self.create_new_scene_btn.inner.triggered = false;
        }
    }
}
