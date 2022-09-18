use crate::glob::*;
const NUM_KEYS: usize = 119;
#[derive(Copy, Clone, PartialEq)]
pub enum KeyState {
    Pressed,
    Released,
}
pub struct RendererFeedback {
    pub mouse: MouseState,
    pub exit: bool,
    pub key_states: [KeyState; NUM_KEYS],
    pub loaded_world_area: types::WorldRect,
    pub update_necessary: bool,
}

impl RendererFeedback {
    pub fn new() -> Self {
        Self {
            mouse: MouseState::new(),
            exit: false,
            key_states: [KeyState::Released; NUM_KEYS],
            loaded_world_area: types::WorldRect::default(),
            update_necessary: false,
        }
    }
}

pub struct MouseState {
    pub left: bool,
    pub middle: bool,
    pub right: bool,
    pub pos: types::WorldCoordinate,
    pub pos_diff: types::WorldCoordinate,
}

impl MouseState {
    fn new() -> Self {
        MouseState {
            left: false,
            middle: false,
            right: false,
            pos: types::WorldCoordinate::new(0.0, 0.0),
            pos_diff: types::WorldCoordinate::new(0.0, 0.0),
        }
    }
}
