use crate::glob::types::{ScreenCoordinate};

#[derive(Clone, Copy)]
pub struct Settings
{
    pub fps: f32,
    pub screen_size: ScreenCoordinate,
    pub scale: f32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: 60.0,
            screen_size: ScreenCoordinate::new(1280.0, 720.0),
            scale: 1.0,
        }
    }
}
