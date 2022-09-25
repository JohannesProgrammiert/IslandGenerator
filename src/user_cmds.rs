use crate::glob::*;
pub const NUM_KEYS: usize = 119;
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

impl Default for RendererFeedback {
    fn default() -> Self {
        Self {
            mouse: MouseState::default(),
            exit: false,
            key_states: [KeyState::Released; NUM_KEYS],
            loaded_world_area: types::WorldRect::default(),
            update_necessary: false,
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub struct MouseState {
    pub left: bool,
    pub middle: bool,
    pub right: bool,
    pub pos: types::WorldCoordinate,
    pub pos_diff: types::WorldVector,
}

impl Default for MouseState {
    fn default() -> Self {
        MouseState {
            left: false,
            middle: false,
            right: false,
            pos: types::WorldCoordinate::new(0.0, 0.0),
            pos_diff: types::WorldVector::new(0.0, 0.0),
        }
    }
}
