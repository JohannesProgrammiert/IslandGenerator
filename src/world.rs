use crate::glob::types::*;
use rand::distributions::Distribution;
pub const CHUNK_SIZE: f32 = 128.0;
pub mod island;
use island::Island;

/// A `Chunk` is currently a placeholder struct to mark a certain world chunk as 'occupied'
pub struct Chunk {}

impl Chunk {
    pub fn new() -> Self {
        Chunk {}
    }
}

pub type ChunkIndex = euclid::default::Point2D<isize>;

/// Game world. Is made out of islands.
pub struct World {
    /// islands in this world
    pub islands: Vec<Island>,
    /// minimum rectangle in world coordinates that contains all islands
    pub clipping_rect: WorldRect,
    /// chunks (indexed by upper left corner) mark world as 'generated' so areas that were visited once do not get re-generated
    pub chunks: std::collections::HashMap<ChunkIndex, Chunk>,
    /// Screen center world position
    pub screen_pos: WorldCoordinate,
}

impl Default for World {
    fn default() -> Self {
        World {
            islands: Vec::new(),
            clipping_rect: WorldRect::default(),
            chunks: std::collections::HashMap::new(),
            screen_pos: WorldCoordinate::new(0.0, 0.0),
        }
    }
}
impl World {
    /// Generate a new chunk with index `ind`
    pub fn gen_chunk(&mut self, ind: ChunkIndex) {
        // generating this chunk may cause a snowball effect
        let chunk_pos = WorldCoordinate::new(ind.x as f32 * CHUNK_SIZE, ind.y as f32 * CHUNK_SIZE);
        log::debug!("Generating chunk {} {}", chunk_pos.x, chunk_pos.y);
        let mut rng = rand::thread_rng();
        let die = rand::distributions::Bernoulli::new(0.5).unwrap();
        if die.sample(&mut rng) {
            // try to place island in middle of chunk
            if let Some(mut island) = Island::new(chunk_pos + WorldVector::new(CHUNK_SIZE, CHUNK_SIZE)/2.0) {
                let mut fits = false;
                let mut intersects_none = true;
                for index in self.chunks.keys() {
                    let chunk = WorldRect::new(
                        WorldCoordinate::new(
                            index.x as f32 * CHUNK_SIZE,
                            index.y as f32 * CHUNK_SIZE,
                        ),
                        WorldVector::new(
                            CHUNK_SIZE,
                            CHUNK_SIZE,
                        ).to_size());
                    if island.clipping_rect.intersects(&chunk) {
                        intersects_none = false;
                    }
                }
                if !intersects_none { // try shifting around
                    'outer: for x in -CHUNK_SIZE as isize..CHUNK_SIZE as isize +1 {
                        for y in -CHUNK_SIZE as isize..CHUNK_SIZE as isize + 1 {
                            let offset = WorldCoordinate::new(x as f32, y as f32);
                            // shift clipping rect in any direction until it works
                            let mut intersects_none = true;
                            for index in self.chunks.keys() {
                                let chunk = WorldRect::new(
                                    WorldCoordinate::new(
                                        index.x as f32 * CHUNK_SIZE,
                                        index.y as f32 * CHUNK_SIZE,
                                    ),
                                    WorldVector::new(
                                        CHUNK_SIZE,
                                        CHUNK_SIZE,
                                    ).to_size());
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
                    let chunk_min = ChunkIndex::new(
                        f32::floor(island.clipping_rect.origin.x / CHUNK_SIZE) as isize,
                        f32::floor(island.clipping_rect.origin.y / CHUNK_SIZE) as isize,
                    );
                    let chunk_max = ChunkIndex::new(
                        f32::floor(island.clipping_rect.max_x() / CHUNK_SIZE) as isize,
                        f32::floor(island.clipping_rect.max_y() / CHUNK_SIZE) as isize,
                    );
                    if chunk_min == chunk_max {
                        if let std::collections::hash_map::Entry::Vacant(e) = self.chunks.entry(chunk_min) {
                            log::debug!("Register chunk at {}, {}", chunk_min.x, chunk_min.y);
                            e.insert(Chunk::new());
                        }
                        else {
                            log::debug!("Chunk {} {} Already exists", chunk_min.x, chunk_min.y);
                        }
                    }
                    else {
                        for x in chunk_min.x..chunk_max.x {
                            for y in chunk_min.y..chunk_max.y {
                                let chunk_index = ChunkIndex::new(
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
                    log::debug!("Inserting island with clipping rect {:?} - {:?}", island.clipping_rect.origin, island.clipping_rect.size);
                    self.islands.push(island);
                }
            }
        }
        self.chunks.entry(ind).or_insert_with(|| {
            log::debug!("Register chunk at {}, {}", ind.x, ind.y);
            Chunk::new()
        });

        // re-generate clipping rect of world
        let mut min_pos = WorldCoordinate::new(f32::MAX, f32::MAX);
        let mut max_pos = WorldCoordinate::new(f32::MIN, f32::MIN);
        for index in self.chunks.keys() {
            if index.x as f32 * CHUNK_SIZE < min_pos.x {
                min_pos = WorldCoordinate::new(index.x as f32 * CHUNK_SIZE as f32, min_pos.y);
            }
            if index.y as f32 * CHUNK_SIZE < min_pos.y {
                min_pos = WorldCoordinate::new(min_pos.x, index.y as f32 * CHUNK_SIZE);
            }
            if index.x as f32 * CHUNK_SIZE + CHUNK_SIZE > max_pos.x {
                max_pos = WorldCoordinate::new(index.x as f32 * CHUNK_SIZE + CHUNK_SIZE, max_pos.y);
            }
            if index.y as f32 * CHUNK_SIZE + CHUNK_SIZE > max_pos.y {
                max_pos = WorldCoordinate::new(max_pos.x, index.y as f32 * CHUNK_SIZE + CHUNK_SIZE);
            }
        }
        let clipping_points = vec![min_pos, max_pos];
        self.clipping_rect = WorldRect::from_points(clipping_points.into_iter());
        log::debug!("New world clipping rect: {:?}", self.clipping_rect);
    }
}
