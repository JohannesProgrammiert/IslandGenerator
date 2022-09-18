use crate::glob::types::*;
use crate::world::{World, CHUNK_SIZE};

pub struct MapRenderer {
    map: allegro::Bitmap,
    map_size: ScreenCoordinate
}

impl MapRenderer {
    pub fn new(allegro_core: &allegro::Core) -> Self {
        let map = allegro::Bitmap::new(allegro_core, 1, 1).unwrap();
        let map_size = ScreenCoordinate::new(1.0, 1.0);
        MapRenderer {
            map,
            map_size,
        }
    }
    pub fn update(
        &mut self,
        world: &World,
        allegro_core: &allegro::Core,
        allegro_display: &allegro::Display) {
        // create texture in RAM
        allegro_core.set_new_bitmap_flags(allegro::MEMORY_BITMAP);
        let map = allegro::Bitmap::new(
            &*allegro_core,
            world.clipping_rect.width() as i32,
            world.clipping_rect.height() as i32,
        )
            .expect("Cannot create map texture");
        allegro_core.set_target_bitmap(Some(&map));
        log::debug!("Number of islands {}", world.islands.len());
        for x in 0..world.clipping_rect.width() as usize {
            for y in 0..world.clipping_rect.height() as usize {
                allegro_core.put_pixel(x as i32, y as i32, allegro::Color::from_rgba(255, 255, 255, 255));
            }
        }
        for (ind, _chunk) in &world.chunks {
            for x in 0..CHUNK_SIZE as usize {
                for y in 0..CHUNK_SIZE as usize {
                    let pos = WorldCoordinate::new(
                        ind.x() as f32 * CHUNK_SIZE + x as f32,
                        ind.y() as f32 * CHUNK_SIZE + y as f32,
                    ) - world.clipping_rect.upper_left();
                    let color = allegro::Color::from_rgba(0, 255, 0, 128);
                    allegro_core.put_pixel(pos.x() as i32, pos.y() as i32, color);
                }
            }
        }
        for island in &world.islands {
            for x in 0..island.tiles.len() {
                for tile in &island.tiles[x] {
                    let color: allegro::Color;
                    if tile.height <= 0.0 {
                        color = allegro::Color::from_rgba(0, 0, 255, 255);
                    }
                    else if tile.height <= 1.0 {
                        color = allegro::Color::from_rgba(255, 255, 0, 255);
                    }
                    else if tile.height <= 2.0 {
                        color = allegro::Color::from_rgba(255, 0, 255, 255);
                    }
                    else {
                        color = allegro::Color::from_rgba(128, 128, 128, 255);
                    }
                    let pos = tile.pos - world.clipping_rect.upper_left();
                    allegro_core.put_pixel(pos.x() as i32, pos.y() as i32, color);
                }
            }
        }
        allegro_core.set_target_bitmap(Some(allegro_display.get_backbuffer()));
        allegro_core.set_new_bitmap_flags(allegro::VIDEO_BITMAP);
        self.map = allegro_display.convert_bitmap(&map).unwrap();
        self.map_size =
            ScreenCoordinate::new(world.clipping_rect.width(), world.clipping_rect.height());
    }
    pub fn draw(&self, allegro_core: &allegro::Core, pos: ScreenCoordinate, world: &World) {
        let flags = allegro::core::FLIP_NONE;
        allegro_core.draw_tinted_scaled_rotated_bitmap_region(
            &self.map,
            // texture start
            0.0,
            0.0,
            // texture dimensions
            self.map_size.x(),
            self.map_size.y(),
            allegro::Color::from_rgb_f(1.0, 1.0, 1.0),
            0.0,
            0.0,
            // position
            pos.x(),
            pos.y(),
            // scale
            1.0,
            1.0,
            0.0,
            flags,
        );
        allegro_core.draw_pixel(
            // position
            pos.x() + world.screen_pos.x() - world.clipping_rect.upper_left().x(),
            pos.y() + world.screen_pos.y() - world.clipping_rect.upper_left().y(),
            allegro::Color::from_rgb(255, 0, 0));
        allegro_core.draw_pixel(
            // position
            pos.x() + world.screen_pos.x() - world.clipping_rect.upper_left().x()-1.0,
            pos.y() + world.screen_pos.y() - world.clipping_rect.upper_left().y(),
            allegro::Color::from_rgb(255, 0, 0));
        allegro_core.draw_pixel(
            // position
            pos.x() + world.screen_pos.x() - world.clipping_rect.upper_left().x()+1.0,
            pos.y() + world.screen_pos.y() - world.clipping_rect.upper_left().y(),
            allegro::Color::from_rgb(255, 0, 0));
    }
}
