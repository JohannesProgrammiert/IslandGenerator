mod egui_allegro_backend;
mod engine;
mod gui;
mod map;
pub mod settings;
use crate::glob;
use map::MapRenderer;
use crate::glob::types::*;
use crate::user_cmds::{KeyState, UserFeedback};
use crate::world::World;
use engine::Engine;
use settings::Settings;
use std::sync::Arc;
use std::sync::Mutex;

pub struct Renderer {
    settings: Settings,
    engine: Engine,
    egui_engine: egui_allegro_backend::Backend,
    world: Arc<Mutex<World>>,
    user: Arc<Mutex<UserFeedback>>,
    mouse: ScreenCoordinate,
    apparent_tile_size: ScreenCoordinate,
    rendered_screen_area: ScreenRect,
    screen_on_world: WorldRect,
    rendered_world_area: WorldRect,
    gui_info: gui::GuiInfo,
    last_draw: std::time::Instant,
    map_renderer: MapRenderer,
}

impl Renderer {
    pub fn new(
        init_settings: Settings,
        world: Arc<Mutex<World>>,
        user: Arc<Mutex<UserFeedback>>,
    ) -> Result<Self, String> {
        let engine = Engine::new(init_settings.fps, init_settings.screen_size)?;
        engine.start_timer();
        let egui_screen_size = egui::Rect {
            min: egui::Pos2 { x: 0.0, y: 0.0 },
            max: egui::Pos2 {
                x: init_settings.screen_size.x(),
                y: init_settings.screen_size.y(),
            },
        };
        let egui_engine = egui_allegro_backend::Backend::new(
            egui_screen_size,
            engine.core.clone(),
            engine.display.clone(),
            engine.primitives_addon.clone(),
        );
        let mut gui_info = gui::GuiInfo::default();
        let rendered_screen_area = ScreenRect::new(
            ScreenCoordinate::new(0.0, 0.0),
            init_settings.screen_size - ScreenCoordinate::new(gui_info.min_side_panel_width, 0.0),
        );
        gui_info.rendered_rect = rendered_screen_area;
        let map_renderer = MapRenderer::new(&*engine.core);
        Ok(Renderer {
            settings: init_settings,
            engine,
            egui_engine,
            world,
            user,
            mouse: ScreenCoordinate::new(0.0, 0.0),
            apparent_tile_size: glob::TILE_SIZE * init_settings.scale,
            rendered_screen_area,
            screen_on_world: WorldRect::from_screen(
                rendered_screen_area,
                init_settings.scale,
                WorldCoordinate::new(0.0, 0.0),
            ),
            rendered_world_area: WorldRect::default(),
            gui_info,
            last_draw: std::time::Instant::now(),
            map_renderer
        })
    }
    const MOUSE_SCALE_FACTOR: f32 = 0.2;
    const MAX_SCALE: f32 = 7.0;
    const MIN_SCALE: f32 = 0.2;
    pub fn next_frame(&mut self) -> bool {
        //-- trick borrow checker to use shared memory and lock only once per call
        let world = self.world.clone();
        let mut world = world.lock().unwrap();
        let user = self.user.clone();
        let mut user = user.lock().unwrap();
        //--------------------------------------------------------------------------
        self.screen_on_world = WorldRect::from_screen(
            self.rendered_screen_area,
            self.settings.scale,
            world.screen_pos,
        );
        let mut redraw: bool = false;
        loop {
            if self.engine.event_queue.is_empty() {
                break;
            }
            let event = self.engine.event_queue.wait_for_event();
            self.egui_engine.handle_allegro_event(&event);
            match event {
                allegro::TimerTick { .. } => {
                    redraw = true;
                }
                allegro::DisplayClose { .. } => {
                    user.exit = true;
                    return false;
                }
                allegro::KeyDown { keycode, .. } => {
                    if keycode == allegro::KeyCode::Q {
                        user.exit = true;
                        return false;
                    } else if (keycode as usize) < user.key_states.len() {
                        user.key_states[keycode as usize] = KeyState::Pressed;
                    }
                }
                allegro::KeyUp { keycode, .. } => {
                    if (keycode as usize) < user.key_states.len() {
                        user.key_states[keycode as usize] = KeyState::Released;
                    }
                }
                allegro::MouseAxes { x, y, dz, .. } => {
                    self.mouse = ScreenCoordinate::new(x as f32, y as f32);
                    if dz != 0 {
                        self.settings.scale +=
                            dz as f32 * self.settings.scale * Renderer::MOUSE_SCALE_FACTOR;

                        if self.settings.scale > Renderer::MAX_SCALE {
                            self.settings.scale = Renderer::MAX_SCALE;
                        }
                        if self.settings.scale < Renderer::MIN_SCALE {
                            self.settings.scale = Renderer::MIN_SCALE;
                        }
                        self.apply_settings(world.screen_pos);
                    }
                }
                allegro::DisplayResize { width, height, .. } => {
                    self.settings.screen_size = ScreenCoordinate::new(width as f32, height as f32);
                    self.engine
                        .display
                        .acknowledge_resize()
                        .expect("Failed to resize window");
                    self.apply_settings(world.screen_pos);
                }
                allegro::MouseButtonDown { button, .. } => match button {
                    1 => user.mouse.left = true,
                    3 => user.mouse.middle = true,
                    2 => user.mouse.right = true,
                    _ => {}
                },
                allegro::MouseButtonUp { button, .. } => match button {
                    1 => user.mouse.left = false,
                    3 => user.mouse.middle = false,
                    2 => user.mouse.right = false,
                    _ => {}
                },
                _ => {}
            }
        }
        if redraw {
            if world.map_needs_update {
                self.map_renderer.update(&world, &*self.engine.core, &*self.engine.display);
                world.map_needs_update = false;
            }
            let mouse_in_world = WorldCoordinate::from_screen(
                self.mouse,
                self.screen_on_world.upper_left(),
                self.settings.scale,
            );
            user.mouse.pos_diff = user.mouse.pos - mouse_in_world;
            user.mouse.pos = mouse_in_world;
            // check if corners are outside of world -> shift loaded world
            /* if self.screen_on_world.upper_left().x() <= world.tiles[0][0].pos().x() {
            println!("Extend west!");
            user.extend_request = Direction::West;
        } else if self.screen_on_world.upper_right().y() <= world.tiles[0][0].pos().y() {
            println!("Extend north!");
            user.extend_request = Direction::North;
        } else if self.screen_on_world.lower_right().x()
            >= world.tiles.last().unwrap()[0].pos().x()
            {
            println!("Extend east!");
            user.extend_request = Direction::East;
        } else if self.screen_on_world.lower_left().y()
            >= world.tiles[0].last().unwrap().pos().y()
            {
            println!("Extend south!");
            user.extend_request = Direction::South;
        }*/

            self.draw(&world, &user);
            self.rendered_world_area = WorldRect::new(
                WorldCoordinate::new(
                    self.screen_on_world.upper_left().x(),
                    self.screen_on_world.upper_right().y(),
                ),
                WorldCoordinate::new(
                    self.screen_on_world.lower_right().x(),
                    self.screen_on_world.lower_left().y(),
                ),
            );
            user.loaded_world_area = self.rendered_world_area;
        }
        return redraw;
    }

    fn apply_settings(&mut self, screen_pos: WorldCoordinate) {
        self.apparent_tile_size = glob::TILE_SIZE * self.settings.scale;
        self.rendered_screen_area = ScreenRect::new(
            ScreenCoordinate::new(0.0, 0.0),
            self.settings.screen_size
                - ScreenCoordinate::new(self.gui_info.min_side_panel_width, 0.0),
        );
        self.screen_on_world =
            WorldRect::from_screen(self.rendered_screen_area, self.settings.scale, screen_pos);
    }

    fn draw(&mut self, world: &World, user: &UserFeedback) {
        let elapsed = self.last_draw.elapsed();
        self.last_draw = std::time::Instant::now();
        self.gui_info.fps = 1.0 / elapsed.as_secs_f32();
        self.engine
            .core
            .clear_to_color(allegro::Color::from_rgb_f(0.0, 0.0, 0.0));
        if self.gui_info.show_map {
            self.map_renderer.draw(&*self.engine.core, self.rendered_screen_area.upper_left(), world);
            self.gui_info.drawn_tiles = 0;
        } else {
            self.engine.core.hold_bitmap_drawing(true);
            self.gui_info.drawn_tiles = self.draw_world(world, user);
            self.engine.core.hold_bitmap_drawing(false);
        }
        self.gui_info.mouse_pos = user.mouse.pos;
        self.gui_info.rendered_rect = self.rendered_screen_area;
        self.egui_engine.draw(gui::draw_gui, &mut self.gui_info);
        self.rendered_screen_area = self.gui_info.rendered_rect; // copy back user values
        self.screen_on_world = WorldRect::from_screen(
            self.rendered_screen_area,
            self.settings.scale,
            world.screen_pos,
        );
        self.engine.core.flip_display();
    }
    fn draw_world(&self, world: &World, user: &UserFeedback) -> usize {
        let mut drawn_cells = 0;
        let flags = allegro::core::FLIP_NONE;
        for island in &world.islands {
            if !island.clipping_rect.intersects(&self.rendered_world_area) {
                continue;
            }
            // println!("Island intersects world");
            for x in 0..island.tiles.len() {
                for tile in &island.tiles[x] {
                    // if tile.tile_type == Water {
                    //   continue;
                    // }
                    let tile_pos = tile.pos;
                    let tile_screen_pos = ScreenCoordinate::from_world(
                        tile_pos,
                        self.screen_on_world.upper_left(),
                        self.settings.scale,
                    );
                    let tile_screen_base = tile_screen_pos
                        - ScreenCoordinate::new(
                            128.0 * self.settings.scale / 2.0,
                            128.0 * self.settings.scale / 2.0,
                        );
                    // skip if tile is out of screen
                    if tile_screen_base.x() < self.rendered_screen_area.upper_left().x() - 2.0 * self.apparent_tile_size.x()
                        || tile_screen_base.y() < self.rendered_screen_area.upper_left().y() - 4.0 * self.apparent_tile_size.y()
                        || tile_screen_base.x() >= self.rendered_screen_area.upper_right().x()
                        || tile_screen_base.y() >= self.rendered_screen_area.lower_right().y()
                    {
                        continue;
                    }
                    let bitmap: engine::TextureType;
                    if tile.height <= 0.0 {
                        continue;
                        // bitmap = engine::TextureType::Water
                    }
                    else {
                        bitmap = engine::TextureType::Tile;
                    }
                    // else if tile.height <= 1.0 {
                    //     bitmap = engine::TextureType::Sand;
                    // }
                    // else if tile.height <= 2.0 {
                    // }
                    // else {
                    //     bitmap = engine::TextureType::Rock;
                    // }
                    self.engine.core.draw_tinted_scaled_rotated_bitmap_region(
                        &self.engine.bitmaps[bitmap as usize],
                        // texture start
                        0.0,
                        0.0,
                        // texture dimensions
                        128.0,
                        128.0,
                        allegro::Color::from_rgb_f(1.0, 1.0, 1.0),
                        0.0,
                        0.0,
                        // position
                        tile_screen_base.x(),
                        tile_screen_base.y(),
                        self.settings.scale,
                        self.settings.scale,
                        0.0,
                        flags,
                    );
                    drawn_cells += 1;
                    if user.mouse.pos.x() > tile_pos.x()
                        && user.mouse.pos.x() < (tile_pos.x() + 1.0)
                        && user.mouse.pos.y() > tile_pos.y()
                        && user.mouse.pos.y() < (tile_pos.y() + 1.0)
                    {
                        self.engine.core.draw_tinted_scaled_rotated_bitmap_region(
                            &self.engine.bitmaps[engine::TextureType::FocusedGreen as usize],
                            // texture start
                            0.0,
                            0.0,
                            // texture dimensions
                            glob::TILE_TEXTURE_SIZE.x(),
                            glob::TILE_TEXTURE_SIZE.y(),
                            allegro::Color::from_rgb_f(1.0, 1.0, 1.0),
                            0.0,
                            0.0,
                            // position
                            tile_screen_base.x(),
                            tile_screen_base.y(),
                            self.settings.scale,
                            self.settings.scale * glob::PERSPECTIVE_DISTORTION_Y,
                            0.0,
                            flags,
                        );
                        drawn_cells += 1;
                    }
                }
            }
        }
        return drawn_cells;
    }
}