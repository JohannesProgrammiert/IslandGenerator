use crate::glob::types::{ScreenCoordinate};

#[derive(Clone, Copy)]
pub struct Settings
{
    pub fps: f32,
    pub screen_size: ScreenCoordinate,
    pub scale: f32,
}
