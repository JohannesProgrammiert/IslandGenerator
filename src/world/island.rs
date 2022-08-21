use crate::glob::types::*;
use crate::world::{Tile};
use rand::distributions::Distribution;
pub struct Island {
    pub clipping_rect: WorldRect,
    pub tiles: Vec<Vec<Tile>>,
}
const MAX_RANDMAP_EXP: usize = 5;
const MIN_RANDMAP_EXP: usize = 3;
const MAX_INTERPOLATION_SCALE: usize = 12;
const MIN_INTERPOLATION_SCALE: usize = 8;
const GAUSS_WINDOW_SIZE: usize = 7;
const GAUSS_WINDOW: [[f32; GAUSS_WINDOW_SIZE]; GAUSS_WINDOW_SIZE] = [
    [0.00000067, 0.00002292, 0.00019117, 0.00038771, 0.00019117, 0.00002292, 0.00000067],
    [0.00002292, 0.00078633, 0.00655965, 0.01330373, 0.00655965, 0.00078633, 0.00002292],
    [0.00019117, 0.00655965, 0.05472157, 0.11098164, 0.05472157, 0.00655965, 0.00019117],
    [0.00038771, 0.01330373, 0.11098164, 0.22508352, 0.11098164, 0.01330373, 0.00038771],
    [0.00019117, 0.00655965, 0.05472157, 0.11098164, 0.05472157, 0.00655965, 0.00019117],
    [0.00002292, 0.00078633, 0.00655965, 0.01330373, 0.00655965, 0.00078633, 0.00002292],
    [0.00000067, 0.00002292, 0.00019117, 0.00038771, 0.00019117, 0.00002292, 0.00000067]];
impl Island {
    pub fn new(origin: WorldCoordinate) -> Option<Self> {
        // 1. generate random map with diamond square algorithm
        // note: array must be quadratic with edge len 2^n + 1
        // we add water padding, so 2^n + 3
        let mut rng = rand::thread_rng();
        let die = rand::distributions::Uniform::new(MIN_RANDMAP_EXP, MAX_RANDMAP_EXP);
        let exp = die.sample(&mut rng) as u32;
        let randmap_size = (u32::pow(2, exp) + 3) as usize;
        println!("Randmap size {}", randmap_size);
        let mut randmap: Vec<Vec<f32>> = vec![vec![0.0; randmap_size]; randmap_size];

        // define area in which to apply diamond-square algorithm
        let area = Rect::new(
            Coord::new(1, 1),
            Coord::new(randmap.len()-2, randmap[0].len()-2));

        Island::diamond_square_gen(
            &mut randmap,
            area,
            0
        );

        // set outer border to -1.0
        for x in 0..randmap_size {
            for y in 0..randmap_size {
                if x > 0 && (x < randmap_size - 1) && y > 0 && y < (randmap_size - 1) {
                    continue;
                }
                randmap[x][y] = -0.1;
            }
        }

        // 2. Generate heightmap using bilinear interpolation of randmap
        let die = rand::distributions::Uniform::new(MIN_INTERPOLATION_SCALE, MAX_INTERPOLATION_SCALE);
        let interpolation_scale = die.sample(&mut rng) as usize;
        let heightmap = Island::interpolate(randmap, interpolation_scale);

        // 3. smooth it
        // let mut smooth_heightmap = Island::average_smooth(heightmap, 3);
        let smooth_heightmap = Island::gauss_smooth(heightmap);

        let cut_heightmap = Island::cut_map(smooth_heightmap);
        if cut_heightmap.len() == 0 {
            return None;
        }
        if cut_heightmap[0].len() == 0 {
            return None;
        }
        // calculate clipping rect
        let clipping_rect = WorldRect::new(
            WorldCoordinate::new(-(cut_heightmap.len() as f32)/2.0, -(cut_heightmap.len() as f32)/2.0) + origin,
            WorldCoordinate::new((cut_heightmap[0].len() as f32)/2.0, (cut_heightmap[0].len() as f32)/2.0) + origin,
        );

        let mut tiles: Vec<Vec<Tile>> = Vec::new();
        for x in 0..cut_heightmap.len() as usize {
            let mut new_col: Vec<Tile> = Vec::new();
            for y in 0..cut_heightmap[x].len() as usize {
                let mut tile = Tile::new(clipping_rect.upper_left() + WorldCoordinate::new(x as f32, y as f32));
                tile.height = cut_heightmap[x][y];
                new_col.push(tile);
            }
            tiles.push(new_col);
        }
        println!("Island created");
        Some(Island {
            clipping_rect,
            tiles,
        })
    }

    fn cut_map(map: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
        // cut to create minimal rectangle
        // -- find minimal coordinates, find maximum coordinates
        let mut min_col = map.len()-1;
        let mut max_col = 0;
        let mut min_row = map[0].len()-1;
        let mut max_row = 0;
        for x in 0..map.len() {
            for y in 0..map[x].len() {
                if map[x][y] > 0.0 {
                    if min_col > x {min_col = x;}
                    if max_col < x {max_col = x;}
                    if min_row > y {min_row = y;}
                    if max_row < y {max_row = y;}
                }
            }
        }
        let mut cut_heightmap: Vec<Vec<f32>> = Vec::new();
        for x in min_col..(max_col+1) {
            let mut new_col: Vec<f32> = Vec::new();
            for y in min_row..(max_row+1) {
                new_col.push(map[x][y]);
            }
            cut_heightmap.push(new_col);
        }
        cut_heightmap
    }

    // bilinear interpolation of randmap to array of size randmap.len() * interpolation_scale
    // formula from wikipedia
    fn interpolate(randmap: Vec<Vec<f32>>, interpolation_scale: usize) -> Vec<Vec<f32>> {
        let heightmap_size = (randmap.len()-1) * interpolation_scale;
        let mut ret = vec![vec![0.0; heightmap_size]; heightmap_size];
        for x in 0..ret.len() {
            for y in 0..ret[x].len() {
                // according indices in randmap
                let rand_x = x as f32 / interpolation_scale as f32;
                let rand_y = y as f32 / interpolation_scale as f32;
                // println!("x {} y {} rand_x {} rand_y {} inter {}", x, y, rand_x, rand_y, interpolation_scale);

                // upper left corner in randmap
                let x1 = rand_x as usize;
                let y1 = rand_y as usize;
                // lower right corner in randmap
                let x2 = rand_x as usize + 1;
                let y2 = rand_y as usize + 1;

                // intermediate results
                // let x1_dist = (x - (rand_x) * interpolation_scale) as f32;
                // let x2_dist = ((rand_x+1) * interpolation_scale - x) as f32;
                // let y1_dist = (y - (rand_y) * interpolation_scale) as f32;
                // let y2_dist = ((rand_y+1) * interpolation_scale - y) as f32;

                // normalize factor
                // let div: f32 = (interpolation_scale * interpolation_scale) as f32;

                // not-normalized interpolation
                let inter: f32 =
                    randmap[x1][y1] * (1.0 - f32::fract(rand_x)) * (1.0 - f32::fract(rand_y))
                    + randmap[x2][y1] * f32::fract(rand_x) * (1.0 - f32::fract(rand_y))
                    + randmap[x1][y2] * (1.0 - f32::fract(rand_x)) * f32::fract(rand_y)
                    + randmap[x2][y2] * f32::fract(rand_x) * f32::fract(rand_y);

                ret[x][y] = inter / 4.0;
            }
        }
        ret
    }
    fn gauss_smooth(map: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
        let mut ret: Vec<Vec<f32>> = Vec::new();
        for x in 0..map.len() {
            let mut new_col: Vec<f32> = Vec::new();
            for y in 0..map[x].len() {
                let mut acc: f32 = 0.0;
                let x_signed = x as isize;
                let y_signed = y as isize;
                for dx in -(GAUSS_WINDOW_SIZE as isize)/2..((GAUSS_WINDOW_SIZE/2+1) as isize) {
                    for dy in -(GAUSS_WINDOW_SIZE as isize)/2..((GAUSS_WINDOW_SIZE/2+1) as isize) {
                        let xx = x_signed+dx;
                        let yy = y_signed+dy;
                        if (yy >= 0) && ((yy as usize) < map[x].len())
                            && (xx >= 0) && ((xx as usize) < map.len()) {
                                acc += 2.0 *map[xx as usize][yy as usize] * GAUSS_WINDOW[(dx + (GAUSS_WINDOW_SIZE as isize)/2) as usize][(dy + (GAUSS_WINDOW_SIZE as isize)/2) as usize];
                            }
                        else {
                            // acc -= 0.5 * gauss_filter[(dx + GAUSS_WINDOW_SIZE/2) as usize][(dy + GAUSS_WINDOW_SIZE/2) as usize];
                        }
                    }
                }
                let avg = acc / GAUSS_WINDOW_SIZE as f32;
                new_col.push(avg);
            }
            ret.push(new_col);
        }
        ret
    }

    fn average_smooth(map: Vec<Vec<f32>>, window_size: isize) -> Vec<Vec<f32>> {
        let mut ret: Vec<Vec<f32>> = Vec::new();
        for x in 0..map.len() {
            let mut new_col: Vec<f32> = Vec::new();
            for y in 0..map[x].len() {
                let mut acc: f32 = 0.0;
                let x_signed = x as isize;
                let y_signed = y as isize;
                for dx in -window_size/2..(window_size/2+1) {
                    for dy in -window_size/2..(window_size/2+1) {
                        let xx = x_signed+dx;
                        let yy = y_signed+dy;
                        if (yy >= 0) && ((yy as usize) < map[x].len())
                            && (xx >= 0) && ((xx as usize) < map.len()) {
                                acc += map[xx as usize][yy as usize];
                            }
                        else {
                            acc -= 0.2;
                        }
                    }
                }
                let avg = acc / window_size as f32;
                new_col.push(avg);
            }
            ret.push(new_col);
        }
        ret
    }

    const HEIGHT_RAND_MAX: f32 = 0.1;
    const RAND_MAG: f32 = 0.1;
    fn diamond_square_gen(map: &mut Vec<Vec<f32>>, corners: Rect<usize>, it: usize) {
        if corners.width() < 2 || corners.height() < 2 {
            return;
        }
        // println!("Terrain generator iteration {}: {:?}", it, corners);
        let local_center_coord = (corners.upper_left() + corners.lower_right()) / 2;
        // let global_lower_right = Coord::new(self.tiles.len(), self.tiles[0].len());
        // let global_center_coord = global_lower_right / 2;
        // let distance_to_center = f32::sqrt(
        //     f32::powf(local_center_coord.x() as f32 - global_center_coord.x() as f32, 2.0)
        //   + f32::powf(local_center_coord.y() as f32 - global_center_coord.y() as f32, 2.0));
        // println!("{:?} Distance to center {}", local_center_coord, distance_to_center );
        // let distance_to_ocean = (global_lower_right.x() / 2) as f32 - distance_to_center;
        let mut rng = rand::thread_rng();
        let mag = f32::powf(2.0, -Island::RAND_MAG * it as f32);
        let die = rand::distributions::Uniform::new(-Island::HEIGHT_RAND_MAX * mag, Island::HEIGHT_RAND_MAX * mag);
        // let upper_left_adj = corners.upper_left().x() == 0 || corners.upper_left().y() == 0;
        // let upper_right_adj = (corners.upper_right().x() == (self.tiles.len()-1)) || corners.upper_right().y() == 0;
        // let lower_right_adj = (corners.lower_right().x() == (self.tiles.len()-1)) || corners.lower_right().y() == (self.tiles[0].len()-1);
        // let lower_left_adj = (corners.lower_left().x() == 0) || (corners.lower_left().y() == (self.tiles[0].len()-1));

        let upper_left = map[corners.upper_left().x()][corners.upper_left().y()];
        let lower_left = map[corners.lower_left().x()][corners.lower_left().y()];
        let upper_right = map[corners.upper_right().x()][corners.upper_right().y()];
        let lower_right = map[corners.lower_right().x()][corners.lower_right().y()];
        // "diamond step"
        // set center of rect to average plus random
        let center = (upper_left + upper_right + lower_left + lower_right) / 4.0 + die.sample(&mut rng);
        map[local_center_coord.x()][local_center_coord.y()] += center;
        // "square" step
        let center_weight = 2.0;
        let avg_divider = 4.0;
        let west_average = (upper_left + lower_left + center_weight * center) / avg_divider + die.sample(&mut rng);
        let north_average = (upper_left + upper_right + center_weight * center) / avg_divider + die.sample(&mut rng);
        let east_average = (upper_right + lower_right + center_weight * center) / avg_divider + die.sample(&mut rng);
        let south_average = (lower_right + lower_left + center_weight * center) / avg_divider + die.sample(&mut rng);

        let west_coord = (corners.upper_left() + corners.lower_left()) / 2;
        let north_coord = (corners.upper_left() + corners.upper_right()) / 2;
        let east_coord = (corners.upper_right() + corners.lower_right()) / 2;
        let south_coord = (corners.lower_right() + corners.lower_left()) / 2;

        map[west_coord.x()][west_coord.y()]   += west_average;
        map[north_coord.x()][north_coord.y()] += north_average;
        map[east_coord.x()][east_coord.y()]   += east_average;
        map[south_coord.x()][south_coord.y()] += south_average;
        // sub squares
        let next_squares = [
            Rect::new(corners.upper_left(), local_center_coord),
            Rect::new(west_coord, south_coord),
            Rect::new(north_coord, east_coord),
            Rect::new(local_center_coord, corners.lower_right())
        ];
        for square in next_squares {
            Island::diamond_square_gen(map, square, it + 1);
        }
    }
    /*fn extend(&mut self, dir: Direction) {
    match dir {
    Direction::North => {
    // extend columns in negative direction (-y)
    let start_pos = self.tiles[0][0].pos - WorldCoordinate::new(0.0, 1.0);
    for x in 0..self.tiles.len() {
    let pos = start_pos + WorldCoordinate::new(x as f32, 0.0);
    self.tiles[x].insert(0, Tile::new(TileType::Water, pos));
}
}
    Direction::East => {
    // push back new column (+x)
    let start_pos =
    self.tiles.last().unwrap()[0].pos + WorldCoordinate::new(1.0, 0.0);
    let col_size = self.tiles[0].len();
    let mut new_col: Vec<Tile> = Vec::new();
    for y in 0..col_size {
    let pos = start_pos + WorldCoordinate::new(0.0, y as f32);
    new_col.push(Tile::new(TileType::Water, pos));
}
    self.tiles.push(new_col);
}
    Direction::South => {
    // extend cols (+y)
    let start_pos =
    self.tiles[0].last().unwrap().pos + WorldCoordinate::new(0.0, 1.0);
    for x in 0..self.tiles.len() {
    let pos = start_pos + WorldCoordinate::new(x as f32, 0.0);
    self.tiles[x].push(Tile::new(TileType::Water, pos));
}
}
    Direction::West => {
    // insert new column at start (-x)
    let start_pos = self.tiles[0][0].pos - WorldCoordinate::new(1.0, 0.0);
    let col_size = self.tiles[0].len();
    let mut new_col: Vec<Tile> = Vec::new();
    for y in 0..col_size {
    let pos = start_pos + WorldCoordinate::new(0.0, y as f32);
    new_col.push(Tile::new(TileType::Water, pos));
}
    self.tiles.insert(0, new_col);
}
    _ => return,
}
}
    fn extend_ring(&mut self) -> usize {
    // check inner ring
    let mut extension_requests: [Direction; 4] = [Direction::NoDirection; 4];
    for x in 0..self.tiles.len() {
    for y in 0..self.tiles[x].len() {
    // skip all non-ring entries
    if x > 0 && (x < self.tiles.len() - 1) && y > 0 && y < (self.tiles[x].len() - 1) {
    continue;
}
    // request extension of all borders that contain a non-water tile
    if self.tiles[x][y].tile_type != TileType::Water {
    if x == 0 {
    // extension_requests.push(Direction::North);
    extension_requests[0] = Direction::West;
}
    if y == 0 {
    // extension_requests.push(Direction::West);
    extension_requests[1] = Direction::North;
}
    if x == self.tiles.len() - 1 {
    // extension_requests.push(Direction::South);
    extension_requests[2] = Direction::East;
}
    if y == self.tiles[x].len() - 1 {
    // extension_requests.push(Direction::East);
    extension_requests[3] = Direction::South;
}
}
}
}
    for dir in &extension_requests {
    // println!("Extending");
    self.extend(*dir);
}
    self.clipping_rect = WorldRect::new(
    self.tiles[0][0].pos,
    self.tiles.last().unwrap().last().unwrap().pos,
);
    for request in extension_requests {
    if request != Direction::NoDirection {
    return 1;
}
}
    return 0;
}
    fn interpolate(neighbour_score: f32, size_score: f32) -> TileType {
    // size reduces probability of non-water
    // neighbours increase the probability
    // max neighbours = 3
    let mut rng = rand::thread_rng();
    // let size_rand_val = die.sample(&mut rng);
    // let mut rand_max: u32 = (100.0/neighbour_score/neighbour_score + size_score - size_rand_val) as u32;
    let die = rand::distributions::Uniform::new(150.0, 350.0);
    // println!("neighbours: {} size: {}", neighbour_score, size_score);
    let mut probability: f64 = (neighbour_score * neighbour_score / 25.0 * die.sample(&mut rng)
    / f32::powf(size_score - 8.8, 0.5)) as f64;
    if probability >= 1.0 {
    probability = 0.999;
}
    if probability < 0.2 {
    probability = 0.0;
}
    // first die - hard code probability
    if size_score <= 9.0 {
    probability = 0.99;
}
    // println!("probability {}", probability);
    let die = rand::distributions::Bernoulli::new(probability).unwrap();
    // let die = rand::distributions::Uniform::from(0..rand_max);
    let rand_val = die.sample(&mut rng);
    if rand_val {
    return TileType::Earth;
}
    return TileType::Water;
}
    fn interpolate_outer(&mut self) {
    // calculate score from size
    let size_score = self.tiles.len() * self.tiles[0].len();
    for x in 0..self.tiles.len() {
    for y in 0..self.tiles[0].len() {
    // skip all non-ring entries
    // first check borders then corners because corners have less neighbours
    if x > 0 && (x < self.tiles.len() - 1) && y > 0 && (y < self.tiles[0].len() - 1) {
    continue;
}
    // correct corners
    let mut corner_correction: f32 = 0.0;
    if (x == 0 && y == 0)
    || (x == 0 && (y == self.tiles[x].len() - 1))
    || ((x == self.tiles.len() - 1) && y == 0)
    || ((x == self.tiles.len() - 1) && (y == self.tiles[x].len() - 1))
    {
    corner_correction = 1.0;
}
    // correct next-to-corners
    if y == 0 {
    if x == 1 || x == (self.tiles.len()-2) {
    corner_correction = 0.5;

}
}
    else if y == 1 {
    if x == 0 || (x == self.tiles.len() -1) {
    corner_correction = 0.5;
}

}
    else if y == (self.tiles[x].len()-2) {
    if x == 0 || (x == self.tiles.len() -1) {
    corner_correction = 0.5;
}
}
    else if y == (self.tiles[x].len()-1) {
    if x == 1 || x == (self.tiles.len()-2) {
    corner_correction = 0.5;
}
}
    self.tiles[x][y].tile_type = Island::interpolate(self.neighbour_score(x as usize, y as usize) as f32 + corner_correction, size_score as f32);
}
}
}

    fn neighbour_score(&self, x: usize, y: usize) -> usize {
    let mut score = 0;
    let x_signed = x as isize;
    let y_signed = y as isize;
    for i in -1..2 as isize {
    for j in -1..2 as isize {
    // skip out-of-map tiles
    if (x_signed + i) < 1 || (x_signed + i) > ((self.tiles.len() - 2) as isize) {
    continue;
}
    if (y_signed + j) < 1
    || (y_signed + j) > ((self.tiles[(x_signed + i) as usize].len() - 2) as isize)
    {
    continue;
}
    if self.tiles[(x_signed + i) as usize][(y_signed + j) as usize].tile_type
    != TileType::Water
    {
    score += 1;
}
}
}
    return score;
}*/

    pub fn shift(&mut self, offset: WorldCoordinate) {
        for col in &mut self.tiles {
            for tile in col {
                tile.pos += offset;
            }
        }
        self.clipping_rect.shift(offset);
    }
}
