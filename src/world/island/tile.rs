use crate::glob::types::*;
#[derive(Debug)]
pub struct Tile {
    pub pos: WorldCoordinate,
    pub height: f32,
}

impl Tile {
    pub fn new(pos: WorldCoordinate) -> Self {
        Tile {
            pos,
            height: 0.0,
        }
    }
}
