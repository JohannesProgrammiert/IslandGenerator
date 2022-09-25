use crate::glob::types::*;
use crate::world::Tile;
use rand::distributions::Distribution;
#[derive(Debug)]
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
        log::trace!("Randmap size {}", randmap_size);
        let mut randmap: Vec<Vec<f32>> = vec![vec![0.0; randmap_size]; randmap_size];

        // define area in which to apply diamond-square algorithm
        let area = euclid::default::Rect::<usize>::new(
            euclid::default::Point2D::new(1, 1),
            euclid::default::Size2D::new(randmap.len()-2, randmap[0].len()-2));

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
            origin + WorldVector::new(-(cut_heightmap.len() as f32)/2.0, -(cut_heightmap.len() as f32)/2.0),
            WorldVector::new(cut_heightmap[0].len() as f32, cut_heightmap[0].len() as f32).to_size(),
        );

        let mut tiles: Vec<Vec<Tile>> = Vec::new();
        for x in 0..cut_heightmap.len() as usize {
            let mut new_col: Vec<Tile> = Vec::new();
            for y in 0..cut_heightmap[x].len() as usize {
                let mut tile = Tile::new(clipping_rect.origin + WorldVector::new(x as f32, y as f32));
                tile.height = cut_heightmap[x][y];
                new_col.push(tile);
            }
            tiles.push(new_col);
        }
        log::info!("Island created");
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

    // fn average_smooth(map: Vec<Vec<f32>>, window_size: isize) -> Vec<Vec<f32>> {
    //     let mut ret: Vec<Vec<f32>> = Vec::new();
    //     for x in 0..map.len() {
    //         let mut new_col: Vec<f32> = Vec::new();
    //         for y in 0..map[x].len() {
    //             let mut acc: f32 = 0.0;
    //             let x_signed = x as isize;
    //             let y_signed = y as isize;
    //             for dx in -window_size/2..(window_size/2+1) {
    //                 for dy in -window_size/2..(window_size/2+1) {
    //                     let xx = x_signed+dx;
    //                     let yy = y_signed+dy;
    //                     if (yy >= 0) && ((yy as usize) < map[x].len())
    //                         && (xx >= 0) && ((xx as usize) < map.len()) {
    //                             acc += map[xx as usize][yy as usize];
    //                         }
    //                     else {
    //                         acc -= 0.2;
    //                     }
    //                 }
    //             }
    //             let avg = acc / window_size as f32;
    //             new_col.push(avg);
    //         }
    //         ret.push(new_col);
    //     }
    //     ret
    // }

    const HEIGHT_RAND_MAX: f32 = 0.1;
    const RAND_MAG: f32 = 0.1;
    fn diamond_square_gen(map: &mut Vec<Vec<f32>>, corners: euclid::default::Rect<usize>, it: usize) {
        if corners.width() < 2 || corners.height() < 2 {
            return;
        }
        let local_center_coord = corners.center();
        let mut rng = rand::thread_rng();
        let mag = f32::powf(2.0, -Island::RAND_MAG * it as f32);
        let die = rand::distributions::Uniform::new(-Island::HEIGHT_RAND_MAX * mag, Island::HEIGHT_RAND_MAX * mag);

        let upper_left = map[corners.min_x()][corners.min_y()];
        let lower_left = map[corners.min_x()][corners.max_y()];
        let upper_right = map[corners.max_x()][corners.min_y()];
        let lower_right = map[corners.max_y()][corners.max_y()];
        // "diamond step"
        // set center of rect to average plus random
        let center = (upper_left + upper_right + lower_left + lower_right) / 4.0 + die.sample(&mut rng);
        map[local_center_coord.x][local_center_coord.y] += center;
        // "square" step
        let center_weight = 2.0;
        let avg_divider = 4.0;
        let west_average = (upper_left + lower_left + center_weight * center) / avg_divider + die.sample(&mut rng);
        let north_average = (upper_left + upper_right + center_weight * center) / avg_divider + die.sample(&mut rng);
        let east_average = (upper_right + lower_right + center_weight * center) / avg_divider + die.sample(&mut rng);
        let south_average = (lower_right + lower_left + center_weight * center) / avg_divider + die.sample(&mut rng);

        let west_coord = corners.origin + euclid::default::Vector2D::new(0, corners.height() / 2);
        let north_coord = corners.origin + euclid::default::Vector2D::new(corners.width() / 2, 0);
        let east_coord = west_coord + euclid::default::Vector2D::new(corners.width(), 0);
        let south_coord = north_coord + euclid::default::Vector2D::new(0, corners.height());

        map[west_coord.x][west_coord.y]   += west_average;
        map[north_coord.x][north_coord.y] += north_average;
        map[east_coord.x][east_coord.y]   += east_average;
        map[south_coord.x][south_coord.y] += south_average;
        // sub squares
        let next_squares = [
            euclid::default::Rect::from_points(vec![corners.origin, local_center_coord].into_iter()),
            euclid::default::Rect::from_points(vec![west_coord, south_coord].into_iter()),
            euclid::default::Rect::from_points(vec![north_coord, east_coord].into_iter()),
            euclid::default::Rect::from_points(vec![local_center_coord, euclid::default::Point2D::new(corners.max_x(), corners.max_y())].into_iter())
        ];
        for square in next_squares {
            Island::diamond_square_gen(map, square, it + 1);
        }
    }

    pub fn shift(&mut self, offset: WorldCoordinate) {
        let shift_matrix = euclid::Translation2D::<f32, WorldSpace, WorldSpace>::new(
            offset.x, offset.y,
        );
        for col in &mut self.tiles {
            for tile in col {
                tile.pos = shift_matrix.transform_point(tile.pos);
            }
        }
        self.clipping_rect = shift_matrix.transform_rect(&self.clipping_rect);
    }
}
