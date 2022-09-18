use crate::glob::types::*;
use rand::distributions::Distribution;
const MAX_TEMPERATURE: f32 = 50.0;
const ZERO_TEMPERATURE_LAT: f32 = 1_000_000.0;
pub const CHUNK_SIZE: f32 = 128.0;
pub mod island;
use island::Island;

pub struct Chunk {
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
        }
    }
}
pub struct World {
    pub islands: Vec<Island>,
    pub clipping_rect: WorldRect,
    // chunks (indexed by upper left corner) mark world as 'generated' so areas that were visited once do not get re-generated
    pub chunks: std::collections::HashMap<Coord<isize>, Chunk>,
    /* Screen center world position */
    pub screen_pos: WorldCoordinate,
    pub map_needs_update: bool,
}
impl World {
    pub fn new(init_pos: WorldCoordinate) -> Self {
        World {
            islands: Vec::new(),
            clipping_rect: WorldRect::default(),
            chunks: std::collections::HashMap::new(),
            screen_pos: init_pos, // show on first tile
            map_needs_update: true,
        }
    }
    pub fn gen_chunk(&mut self, ind: Coord<isize>) {
        // generating this chunk may cause a snowball effect
        let chunk_pos = WorldCoordinate::new(ind.x() as f32 * CHUNK_SIZE, ind.y() as f32 * CHUNK_SIZE);
        log::debug!("Generating chunk {} {}", chunk_pos.x(), chunk_pos.y());
        let mut rng = rand::thread_rng();
        let die = rand::distributions::Bernoulli::new(0.5).unwrap();
        if die.sample(&mut rng) {
            // try to place island in middle of chunk
            if let Some(mut island) = Island::new(chunk_pos + WorldCoordinate::new(CHUNK_SIZE, CHUNK_SIZE)/2.0) {
                let mut fits = false;
                let mut intersects_none = true;
                for (index, _chunk) in &self.chunks {
                    let chunk = WorldRect::new(
                        WorldCoordinate::new(
                            index.x() as f32 * CHUNK_SIZE,
                            index.y() as f32 * CHUNK_SIZE,
                        ),
                        WorldCoordinate::new(
                            (index.x() + 1) as f32 * CHUNK_SIZE,
                            (index.y() + 1) as f32 * CHUNK_SIZE,
                        ));
                    if island.clipping_rect.intersects(&chunk) {
                        intersects_none = false;
                    }
                }
                if !intersects_none { // try shifting around
                    'outer: for x in -CHUNK_SIZE as isize..CHUNK_SIZE as isize +1 {
                        for y in -CHUNK_SIZE as isize..CHUNK_SIZE as isize + 1 {
                            let offset = WorldCoordinate::new(x as f32, y as f32);
                            // shift clipping rect in any direction until it works
                            let mut clipping_rect = island.clipping_rect;
                            clipping_rect.shift(offset);
                            let mut intersects_none = true;
                            for (index, _chunk) in &self.chunks {
                                let chunk = WorldRect::new(
                                    WorldCoordinate::new(
                                        index.x() as f32 * CHUNK_SIZE,
                                        index.y() as f32 * CHUNK_SIZE,
                                    ),
                                    WorldCoordinate::new(
                                        (index.x() + 1) as f32 * CHUNK_SIZE,
                                        (index.y() + 1) as f32 * CHUNK_SIZE,
                                    ));
                                if island.clipping_rect.intersects(&chunk) {
                                    intersects_none = false;
                                }
                            }
                            if intersects_none {
                                fits = true;
                                island.shift(offset);
                                break 'outer;
                            }
                        }
                    }
                }
                else {
                    fits = true;
                }
                if fits {
                    // calculate min chunk position
                    let chunk_min = Coord::new(
                        f32::floor(island.clipping_rect.upper_left().x() / CHUNK_SIZE) as isize,
                        f32::floor(island.clipping_rect.upper_left().y() / CHUNK_SIZE) as isize,
                    );
                    let chunk_max = Coord::new(
                        f32::floor(island.clipping_rect.lower_right().x() / CHUNK_SIZE) as isize,
                        f32::floor(island.clipping_rect.lower_right().y() / CHUNK_SIZE) as isize,
                    );
                    if chunk_min == chunk_max {
                        if self.chunks.contains_key(&chunk_min) {
                            log::debug!("Chunk {} {} Already exists", chunk_min.x(), chunk_min.y());
                        }
                        else {
                            log::debug!("Register chunk at {}, {}", chunk_min.x(), chunk_min.y());
                            self.chunks.insert(chunk_min, Chunk::new());
                        }
                    }
                    else {
                        for x in chunk_min.x()..chunk_max.x() {
                            for y in chunk_min.y()..chunk_max.y() {
                                let chunk_index = Coord::new(
                                    x,
                                    y,
                                );
                                if self.chunks.contains_key(&chunk_index) {
                                    log::debug!("Chunk {} {} Already exists", x, y);
                                    continue;
                                }
                                log::debug!("Register chunk at {}, {}", x, y);
                                self.chunks.insert(chunk_index, Chunk::new());
                            }
                        }
                    }
                    log::debug!("Inserting island with clipping rect {:?} - {:?}", island.clipping_rect.upper_left(), island.clipping_rect.lower_right());
                    self.islands.push(island);
                }
            }
        }
        if !self.chunks.contains_key(&ind) {
            log::debug!("Register chunk at {}, {}", ind.x(), ind.y());
            self.chunks.insert(ind, Chunk::new());
        }

        // re-generate clipping rect of world
        let mut min_pos = WorldCoordinate::new(f32::MAX, f32::MAX);
        let mut max_pos = WorldCoordinate::new(f32::MIN, f32::MIN);
        for (index, _chunk) in &self.chunks {
            if index.x() as f32 * CHUNK_SIZE < min_pos.x() {
                min_pos = WorldCoordinate::new(index.x() as f32 * CHUNK_SIZE as f32, min_pos.y());
            }
            if index.y() as f32 * CHUNK_SIZE < min_pos.y() {
                min_pos = WorldCoordinate::new(min_pos.x(), index.y() as f32 * CHUNK_SIZE);
            }
            if index.x() as f32 * CHUNK_SIZE + CHUNK_SIZE > max_pos.x() {
                max_pos = WorldCoordinate::new(index.x() as f32 * CHUNK_SIZE + CHUNK_SIZE, max_pos.y());
            }
            if index.y() as f32 * CHUNK_SIZE + CHUNK_SIZE > max_pos.y() {
                max_pos = WorldCoordinate::new(max_pos.x(), index.y() as f32 * CHUNK_SIZE + CHUNK_SIZE);
            }
        }
        self.clipping_rect = WorldRect::new(min_pos, max_pos);
        log::debug!("New world clipping rect: {:?}", self.clipping_rect);
        self.map_needs_update = true;
    }
}

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
