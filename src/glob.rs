pub mod types;
pub mod dbg;

const TILE_TEXTURE_LEN: f32 = 128.0;
const TILE_TEXTURE_CROP: f32 = 0.0;
pub const PERSPECTIVE_DISTORTION_Y: f32 = 0.5;
const TILE_LEN: f32 = TILE_TEXTURE_LEN / 2.0 - TILE_TEXTURE_CROP;

// const fn and const traits have a limited feature set so this is a bit manual
pub const TILE_TEXTURE_SIZE: types::ScreenCoordinate = types::ScreenCoordinate::new(TILE_TEXTURE_LEN, TILE_TEXTURE_LEN);
pub static TILE_SIZE: types::ScreenCoordinate = types::ScreenCoordinate::new(
    TILE_LEN,
    (TILE_LEN) * PERSPECTIVE_DISTORTION_Y);
const TILE_QUARTER: types::ScreenCoordinate = types::ScreenCoordinate::new(
    TILE_LEN / 2.0,
    ((TILE_LEN) * PERSPECTIVE_DISTORTION_Y) / 2.0,
);
