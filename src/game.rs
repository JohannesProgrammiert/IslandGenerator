//! The `game` reacts to output of `Renderer` and alters the world accordingly
use crate::glob::types::*;
use crate::user_cmds::*;
use crate::world::*;
use allegro::KeyCode;
pub fn update(world: &mut World, renderer_feedback: &RendererFeedback) {
    // determine chunks that lie inside the rendered world area
    let start_chunk = ChunkIndex::new(
        f32::floor(renderer_feedback.loaded_world_area.min_x() / CHUNK_SIZE) as isize,
        f32::floor(renderer_feedback.loaded_world_area.min_y() / CHUNK_SIZE) as isize,
    );
    let end_chunk = ChunkIndex::new(
        (renderer_feedback.loaded_world_area.max_x() / CHUNK_SIZE) as isize,
        (renderer_feedback.loaded_world_area.max_y() / CHUNK_SIZE) as isize,
    );
    let mut needed_chunks: Vec<ChunkIndex> = Vec::new();
    if start_chunk == end_chunk {
        if world.chunks.contains_key(&start_chunk) {
        } else {
            needed_chunks.push(start_chunk);
        }
    } else {
        for x in start_chunk.x..end_chunk.x {
            for y in start_chunk.y..end_chunk.y {
                if world.chunks.contains_key(&ChunkIndex::new(x, y)) {
                    continue;
                }
                needed_chunks.push(ChunkIndex::new(x, y));
            }
        }
    }
    // generate chunks that are missing
    for ind in needed_chunks {
        world.gen_chunk(ind);
    }
    const SPEED: f32 = 0.5;
    if renderer_feedback.key_states[KeyCode::W as usize] == KeyState::Pressed {
        world.screen_pos -= WorldVector::new(SPEED, SPEED);
    }
    if renderer_feedback.key_states[KeyCode::A as usize] == KeyState::Pressed {
        world.screen_pos += WorldVector::new(-SPEED, SPEED);
    }
    if renderer_feedback.key_states[KeyCode::S as usize] == KeyState::Pressed {
        world.screen_pos += WorldVector::new(SPEED, SPEED);
    }
    if renderer_feedback.key_states[KeyCode::D as usize] == KeyState::Pressed {
        world.screen_pos += WorldVector::new(SPEED, -SPEED);
    }
    if renderer_feedback.mouse.right {
        world.screen_pos += renderer_feedback.mouse.pos_diff;
    }
}
