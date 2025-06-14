use std::sync::OnceLock;

use macroquad::input::utils::register_input_subscriber;

static SUBSCRIBER_ID: OnceLock<usize> = OnceLock::new();

// TODO: Should this be part of this crate?
pub fn subscriber_id() -> usize {
    *SUBSCRIBER_ID.get_or_init(register_input_subscriber)
}
