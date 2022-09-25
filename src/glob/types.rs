use crate::glob::*;
#[derive(Clone, Copy, PartialEq)]
#[allow(unused)]
pub enum Direction {
    NoDirection,
    North,
    South,
    West,
    East,
    NorthWest,
    SouthWest,
    NorthEast,
    SouthEast,
}

/// Unit tag for world space
pub struct WorldSpace;

/// Unit tag for screen space
pub struct ScreenSpace;

/// Point2D of f32 in world space
pub type WorldCoordinate = euclid::Point2D<f32, WorldSpace>;
pub type WorldVector = euclid::Vector2D<f32, WorldSpace>;

/// Point2D of f32 in screen space
pub type ScreenCoordinate = euclid::Point2D<f32, ScreenSpace>;
pub type ScreenVector = euclid::Vector2D<f32, ScreenSpace>;

pub type WorldRect = euclid::Rect<f32, WorldSpace>;
pub type ScreenRect = euclid::Rect<f32, ScreenSpace>;

/// Returns the smallest rectangular area of the world that contains the entire screen.
pub fn visible_world_rect(screen: ScreenRect, s2w: euclid::Transform2D<f32, ScreenSpace, WorldSpace>) -> WorldRect {
    let upper_left_world = s2w.transform_point(
        ScreenCoordinate::new(
            - screen.max_x() / 2.0,
            - screen.max_y() / 2.0,
        ));
    let lower_right_world = s2w.transform_point(
        ScreenCoordinate::new(
            screen.max_x() / 2.0,
            screen.max_y() / 2.0,
        ));
    let points = vec![
        upper_left_world,
        lower_right_world,
    ];
    WorldRect::from_points(points.into_iter())
}

pub fn gen_s2w_matrix(zoom: f32, camera: WorldCoordinate) -> euclid::Transform2D<f32, ScreenSpace, WorldSpace> {
    euclid::Transform2D::<f32, ScreenSpace, WorldSpace>::new(
        1.0 / (zoom * TILE_SIZE.x), -1.0/(zoom * TILE_SIZE.x),
        1.0 / (zoom * TILE_SIZE.y), 1.0/(zoom * TILE_SIZE.y),
        camera.x, camera.y,
    )
}

pub fn gen_w2s_matrix(zoom: f32, camera: WorldCoordinate) -> euclid::Transform2D<f32, WorldSpace, ScreenSpace> {
    euclid::Transform2D::<f32, WorldSpace, ScreenSpace>::new(
        zoom * TILE_SIZE.x / 2.0, zoom * TILE_SIZE.y / 2.0,
        - zoom * TILE_SIZE.x / 2.0, zoom * TILE_SIZE.y / 2.0,
        - camera.x - camera.y, - camera.x - camera.y,
    )
}
