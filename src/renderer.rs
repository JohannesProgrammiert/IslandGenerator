//! The renderer renders a `World` (readonly) and detects user input which it hands to the `game``
mod egui_allegro_backend;
mod engine;
mod gui;
mod map;
pub mod settings;
use crate::glob;
use map::MapRenderer;
use crate::glob::types::*;
use crate::user_cmds::{KeyState, MouseState, RendererFeedback, NUM_KEYS};
use crate::world::World;
use engine::Engine;
use settings::Settings;

pub struct Renderer {
    settings: Settings,
    engine: Engine,
    egui_engine: egui_allegro_backend::Backend,
    mouse: ScreenCoordinate,
    apparent_tile_size: ScreenCoordinate,
    rendered_screen_area: ScreenRect,
    screen_on_world: WorldRect,
    rendered_world_area: WorldRect,
    gui_info: gui::GuiInfo,
    last_draw: std::time::Instant,
    map_renderer: MapRenderer,
    key_states: [KeyState; NUM_KEYS],
    mouse_state: MouseState,
}
use thiserror::Error;
#[derive(Error, Debug)]
pub enum RendererError {
    #[error("Failed to initialize allegro engine: {0}")]
    Engine(engine::EngineError)
}
impl Renderer {
    pub fn new(
        init_settings: Settings,
    ) -> Result<Self, RendererError> {
        let engine: Engine;
        match Engine::new(init_settings.fps, init_settings.screen_size) {
            Ok(e) => engine = e,
            Err(e) => return Err(RendererError::Engine(e))
        }
        engine.start_timer();
        let egui_screen_size = egui::Rect {
            min: egui::Pos2 { x: 0.0, y: 0.0 },
            max: egui::Pos2 {
                x: init_settings.screen_size.x,
                y: init_settings.screen_size.y,
            },
        };
        let egui_ctx = egui::Context::default();
        let mut style = (*egui_ctx.style()).clone();
        style.visuals.dark_mode = true;
        style.visuals.window_shadow.extrusion = 0.0;
        egui_ctx.set_style(style);
        let egui_engine = egui_allegro_backend::Backend::new(
            egui_ctx,
            egui_screen_size,
            engine.core.clone(),
            engine.display.clone(),
            engine.primitives_addon.clone(),
        );
        let mut gui_info = gui::GuiInfo::default();
        let rendered_screen_area = ScreenRect::new(
            ScreenCoordinate::new(0.0, 0.0),
            (init_settings.screen_size - ScreenCoordinate::new(gui_info.min_side_panel_width, 0.0)).to_size(),
        );
        gui_info.rendered_rect = rendered_screen_area;
        let map_renderer = MapRenderer::new(&*engine.core);
        let camera_start_pos = WorldCoordinate::new(0.0, 0.0);
        let s2w = gen_s2w_matrix(init_settings.scale, camera_start_pos);
        Ok(Renderer {
            settings: init_settings,
            engine,
            egui_engine,
            mouse: ScreenCoordinate::new(0.0, 0.0),
            apparent_tile_size: glob::TILE_SIZE * init_settings.scale,
            rendered_screen_area,
            screen_on_world: visible_world_rect(rendered_screen_area, s2w),
            rendered_world_area: WorldRect::default(),
            gui_info,
            last_draw: std::time::Instant::now(),
            map_renderer,
            key_states: [KeyState::Released; NUM_KEYS],
            mouse_state: MouseState::default(),
        })
    }
    const MOUSE_SCALE_FACTOR: f32 = 0.2;
    const MAX_SCALE: f32 = 7.0;
    const MIN_SCALE: f32 = 0.2;
    pub fn next_frame(&mut self, world: &World) -> RendererFeedback {
        let s2w = gen_s2w_matrix(self.settings.scale, world.screen_pos);
        self.screen_on_world = visible_world_rect(self.rendered_screen_area, s2w);
        let mut ret = RendererFeedback::default();
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
                    ret.exit = true;
                    return ret;
                }
                allegro::KeyDown { keycode, .. } => {
                    if keycode == allegro::KeyCode::Q {
                        ret.exit = true;
                        return ret;
                    } else if (keycode as usize) < ret.key_states.len() {
                        self.key_states[keycode as usize] = KeyState::Pressed;
                    }
                }
                allegro::KeyUp { keycode, .. } => {
                    if (keycode as usize) < ret.key_states.len() {
                        self.key_states[keycode as usize] = KeyState::Released;
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
                    1 => self.mouse_state.left = true,
                    3 => self.mouse_state.middle = true,
                    2 => self.mouse_state.right = true,
                    _ => {}
                },
                allegro::MouseButtonUp { button, .. } => match button {
                    1 => self.mouse_state.left = false,
                    3 => self.mouse_state.middle = false,
                    2 => self.mouse_state.right = false,
                    _ => {}
                },
                _ => {}
            }
        }
        if redraw {
            self.map_renderer.update(&world, &*self.engine.core, &*self.engine.display);
            let s2w = gen_s2w_matrix(self.settings.scale, world.screen_pos);
            let mouse_in_world = s2w.transform_point(self.mouse);
            self.mouse_state.pos_diff = self.mouse_state.pos - mouse_in_world;
            self.mouse_state.pos = mouse_in_world;

            self.draw(&world);
            let points = vec![
                WorldCoordinate::new(
                    self.screen_on_world.min_x(),
                    self.screen_on_world.min_y(),
                ),
                WorldCoordinate::new(
                    self.screen_on_world.max_x(),
                    self.screen_on_world.max_y(),
                ),
            ];
            self.rendered_world_area = WorldRect::from_points(points.into_iter());
        }
        ret.loaded_world_area = self.rendered_world_area;
        ret.key_states = self.key_states;
        ret.mouse = self.mouse_state;
        ret.update_necessary = redraw;
        return ret;
    }

    fn apply_settings(&mut self, screen_pos: WorldCoordinate) {
        self.apparent_tile_size = glob::TILE_SIZE * self.settings.scale;
        self.rendered_screen_area = ScreenRect::new(
            ScreenCoordinate::new(0.0, 0.0),
            (self.settings.screen_size - ScreenCoordinate::new(self.gui_info.min_side_panel_width, 0.0)).to_size(),
        );
        let s2w = gen_s2w_matrix(self.settings.scale, screen_pos);
        self.screen_on_world = visible_world_rect(self.rendered_screen_area, s2w);
    }

    fn draw(&mut self, world: &World) {
        let elapsed = self.last_draw.elapsed();
        self.last_draw = std::time::Instant::now();
        self.gui_info.fps = 1.0 / elapsed.as_secs_f32();
        self.engine
            .core
            .clear_to_color(allegro::Color::from_rgb_f(0.0, 0.0, 0.0));
        if self.gui_info.show_map {
            self.map_renderer.draw(&*self.engine.core, self.rendered_screen_area.origin, world);
            self.gui_info.drawn_tiles = 0;
        } else {
            self.engine.core.hold_bitmap_drawing(true);
            self.gui_info.drawn_tiles = self.draw_world(world);
            self.engine.core.hold_bitmap_drawing(false);
        }
        self.gui_info.mouse_pos = self.mouse_state.pos;
        self.gui_info.rendered_rect = self.rendered_screen_area;
        self.egui_engine.draw(gui::draw_gui, &mut self.gui_info);
        self.rendered_screen_area = self.gui_info.rendered_rect; // copy back user values
        let s2w = gen_s2w_matrix(self.settings.scale, world.screen_pos);
        self.screen_on_world = visible_world_rect(self.rendered_screen_area, s2w);
        self.engine.core.flip_display();
    }
    fn draw_world(&self, world: &World) -> usize {
        let mut drawn_cells = 0;
        let flags = allegro::core::FLIP_NONE;
        let w2s = gen_w2s_matrix(self.settings.scale, world.screen_pos);
        for island in &world.islands {
            if !island.clipping_rect.intersects(&self.rendered_world_area) {
                log::trace!("Island rect {:?} does not intersect rendered area {:?}", island.clipping_rect, self.rendered_world_area);
                continue;
            }
            for x in 0..island.tiles.len() {
                for tile in &island.tiles[x] {
                    let tile_pos = tile.pos;
                    let tile_screen_pos = w2s.transform_point(tile_pos);
                    let tile_screen_base = tile_screen_pos
                        - ScreenCoordinate::new(
                            128.0 * self.settings.scale / 2.0,
                            128.0 * self.settings.scale / 2.0,
                        );
                    // skip if tile is out of screen
                    if tile_screen_base.x < self.rendered_screen_area.min_x() - 2.0 * self.apparent_tile_size.x
                        || tile_screen_base.y < self.rendered_screen_area.min_y() - 4.0 * self.apparent_tile_size.y
                        || tile_screen_base.x >= self.rendered_screen_area.max_x()
                        || tile_screen_base.y >= self.rendered_screen_area.max_y()

                    {
                        continue;
                    }
                    let bitmap: engine::TextureType;
                    if tile.height <= 0.0 {
                        continue;
                    }
                    else {
                        bitmap = engine::TextureType::Tile;
                    }
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
                        tile_screen_base.x,
                        tile_screen_base.y,
                        self.settings.scale,
                        self.settings.scale,
                        0.0,
                        flags,
                    );
                    drawn_cells += 1;
                    if self.mouse_state.pos.x > tile_pos.x
                        && self.mouse_state.pos.x < (tile_pos.x + 1.0)
                        && self.mouse_state.pos.y > tile_pos.y
                        && self.mouse_state.pos.y < (tile_pos.y + 1.0)
                    {
                        self.engine.core.draw_tinted_scaled_rotated_bitmap_region(
                            &self.engine.bitmaps[engine::TextureType::FocusedGreen as usize],
                            // texture start
                            0.0,
                            0.0,
                            // texture dimensions
                            glob::TILE_TEXTURE_SIZE.x,
                            glob::TILE_TEXTURE_SIZE.y,
                            allegro::Color::from_rgb_f(1.0, 1.0, 1.0),
                            0.0,
                            0.0,
                            // position
                            tile_screen_base.x,
                            tile_screen_base.y,
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
