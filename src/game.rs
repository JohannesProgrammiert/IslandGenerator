use crate::glob::types::*;
use crate::user_cmds::*;
use crate::world::*;
use allegro::KeyCode;
use std::sync::Arc;
use std::sync::Mutex;
pub struct Game {
    user: Arc<Mutex<UserFeedback>>,
}

impl Game {
    pub fn new(user_feedback: Arc<Mutex<UserFeedback>>) -> Self {
        Game {
            user: user_feedback,
        }
    }
    pub fn update(&self, world: &mut World) {
        let user = self.user.lock().unwrap();
        // determine chunks that lie inside the rendered world area
        let start_chunk = Coord::new(
            f32::floor(user.loaded_world_area.upper_left().x() / crate::world::CHUNK_SIZE) as isize,
            f32::floor(user.loaded_world_area.upper_left().y() / crate::world::CHUNK_SIZE) as isize,
        );
        let end_chunk = Coord::new(
            (user.loaded_world_area.lower_right().x() / crate::world::CHUNK_SIZE) as isize,
            (user.loaded_world_area.lower_right().y() / crate::world::CHUNK_SIZE) as isize,
        );
        /*println!(
            "Start Chunk {} {}, End Chunk {} {}",
            start_chunk.x(),
            start_chunk.y(),
            end_chunk.x(),
            end_chunk.y()
        );*/
        let mut needed_chunks: Vec<Coord<isize>> = Vec::new();
        if start_chunk == end_chunk {
            if world.chunks.contains_key(&start_chunk) {
            } else {
                needed_chunks.push(start_chunk);
            }
        } else {
            for x in start_chunk.x()..end_chunk.x() {
                for y in start_chunk.y()..end_chunk.y() {
                    if world.chunks.contains_key(&Coord::new(x, y)) {
                        continue;
                    }
                    needed_chunks.push(Coord::new(x, y));
                }
            }
        }
        // generate chunks that are missing
        for ind in needed_chunks {
            world.gen_chunk(ind);
        }
        let speed = 0.5;
        if user.key_states[KeyCode::W as usize] == KeyState::Pressed {
            world.screen_pos -= WorldCoordinate::new(speed, speed);
        }
        if user.key_states[KeyCode::A as usize] == KeyState::Pressed {
            world.screen_pos += WorldCoordinate::new(-speed, speed);
        }
        if user.key_states[KeyCode::S as usize] == KeyState::Pressed {
            world.screen_pos += WorldCoordinate::new(speed, speed);
        }
        if user.key_states[KeyCode::D as usize] == KeyState::Pressed {
            world.screen_pos += WorldCoordinate::new(speed, -speed);
        }
        if user.mouse.right {
            world.screen_pos += user.mouse.pos_diff;
        }
    }
}
