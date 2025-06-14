use std::time::Instant;

use macroquad::input::TouchPhase;

use crate::{component::Component, utils::Transform, window::Window};

/// A Quadrilateral Button component.
///
/// Detects touch events from (-0.5, -0.5) to (0.5, 0.5) in its local coordinate system.
pub struct QuadButton {
    pub pressed: bool,
    pub touch_id: Option<u64>,
    pub press_start_at: Instant,
    pub release_start_at: Instant,
    /// A flag indicating whether the button was triggered.
    /// You will need to manually reset this flag, which means
    /// this button is ready to be triggered again.
    ///
    /// We don't use a callback here because it usually leads to
    /// self-referential problems.
    pub triggered: bool,
}

impl Default for QuadButton {
    fn default() -> Self {
        Self {
            pressed: false,
            touch_id: None,
            press_start_at: Instant::now(),
            release_start_at: Instant::now(),
            triggered: false,
        }
    }
}

impl Component for QuadButton {
    fn touch(&mut self, touch: &macroquad::prelude::Touch) -> anyhow::Result<bool> {
        let inside = touch.position.x >= -0.5
            && touch.position.x <= 0.5
            && touch.position.y >= -0.5
            && touch.position.y <= 0.5;
        let should_consume = match touch.phase {
            TouchPhase::Started => {
                if inside {
                    self.touch_id = Some(touch.id);
                }
                false
            }
            TouchPhase::Moved | TouchPhase::Stationary => {
                // TODO: Is this the expected behavior?
                if self.touch_id == Some(touch.id) && !inside {
                    self.touch_id = None;
                }
                false
            }
            TouchPhase::Cancelled => {
                self.touch_id = None;
                false
            }
            TouchPhase::Ended => self.touch_id == Some(touch.id) && inside,
        };
        if should_consume {
            self.triggered = true;
            self.touch_id = None;
        }
        Ok(should_consume)
    }

    fn render(&mut self, _tr: &Transform, _target: &mut Window) {
        let touching = self.touch_id.is_some();
        if self.pressed != touching {
            self.pressed = touching;
            if touching {
                self.press_start_at = Instant::now();
            } else {
                self.release_start_at = Instant::now();
            }
        }
    }
}
