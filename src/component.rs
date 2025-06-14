use crate::{utils::Transform, window::Window};

pub trait Component {
    /// Returns `Ok(true)` if the event was consumed, `Ok(false)` if not. `Err` is for errors.
    fn touch(&mut self, touch: &macroquad::prelude::Touch) -> anyhow::Result<bool>;

    /// `tr`: (x_comp, y_comp) -> (x_global, y_global)
    fn render(&mut self, tr: &Transform, target: &mut Window);
}
