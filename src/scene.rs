use crate::{component::Component, layout::Layout, utils::Transform};

pub enum NextScene {
    Pop,
    Replace(Box<dyn Scene>),
    Push(Box<dyn Scene>),
}

pub trait Scene: Component {
    /// `None`: no change, `Some(NextScene)`: change scene
    fn next_scene(&mut self) -> Option<NextScene>;
}

pub struct SceneManager {
    pub scene_stack: Vec<Box<dyn Scene>>,
}

impl SceneManager {
    pub fn new(base: impl Scene + 'static) -> Self {
        Self {
            scene_stack: vec![Box::new(base)],
        }
    }

    pub fn current_scene(&mut self) -> &mut Box<dyn Scene> {
        self.scene_stack
            .last_mut()
            .expect("SceneManager requires at least one scene in the stack")
    }
}

impl Layout for SceneManager {
    fn components(&mut self) -> Vec<(Transform, &mut dyn Component)> {
        vec![(
            crate::utils::Transform::identity(),
            self.current_scene().as_mut(),
        )]
    }

    fn after_render(&mut self, _: &Transform, _: &mut crate::window::Window) {
        if let Some(next_scene) = self.current_scene().next_scene() {
            match next_scene {
                NextScene::Pop => {
                    self.scene_stack.pop();
                }
                NextScene::Replace(new_scene) => {
                    self.scene_stack.pop();
                    self.scene_stack.push(new_scene);
                }
                NextScene::Push(new_scene) => {
                    self.scene_stack.push(new_scene);
                }
            }
        }
    }
}
