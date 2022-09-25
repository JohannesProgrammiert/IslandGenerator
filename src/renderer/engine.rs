use crate::glob::*;
use std::rc::Rc;
// use std::ops::Drop;

const PATH_NAMES: [&str; TextureType::NumBitmaps as usize] = [
    "textures/terrain/focused_red.png",
    "textures/terrain/tile.png",
    "textures/terrain/focused_green.png",
    "textures/terrain/water.png",
    "textures/terrain/sand.png",
    "textures/terrain/grass.png",
    "textures/terrain/grass_rock.png",
    "textures/terrain/rock.png",
    "textures/terrain/tree.png",
    "textures/building/house.png",
    "textures/building/forester.png",
];

#[allow(unused)]
pub enum TextureType {
    FocusedRed = 0,
    Tile,
    FocusedGreen,
    Water,
    Sand,
    Grass,
    GrassRock,
    Rock,
    Tree,
    House,
    Forester,
    NumBitmaps,
}

pub struct Engine {
    pub core: Rc<allegro::Core>,
    pub event_queue: allegro::EventQueue,
    pub display: Rc<allegro::Display>,
    pub primitives_addon: Rc<allegro_primitives::PrimitivesAddon>,
    _image_addon: allegro_image::ImageAddon,
    _font_addon: allegro_font::FontAddon,
    _font: allegro_font::Font,
    pub bitmaps: Vec<allegro::Bitmap>,
    timer: allegro::Timer,
}

impl Engine {
    pub fn new(fps: f32, screen_size: types::ScreenCoordinate) -> Result<Self, String> {
        let core: allegro::Core;
        match allegro::Core::init() {
            Ok(c) => core = c,
            Err(e) => return Err(e),
        }
        let core = Rc::new(core);

        let event_queue: allegro::EventQueue;
        match allegro::EventQueue::new(&core) {
            Ok(ev) => event_queue = ev,
            Err(_) => return Err("Cannot create event queue".to_string()),
        }

        core.set_new_display_flags(allegro::display::RESIZABLE);
        let display: allegro::Display;
        match allegro::Display::new(&core, screen_size.x as i32, screen_size.y as i32) {
            Ok(d) => display = d,
            Err(_) => return Err("Cannot create display".to_string()),
        }
        display.set_window_title("Game");
        let display = Rc::new(display);

        let primitives_addon: allegro_primitives::PrimitivesAddon;
        match allegro_primitives::PrimitivesAddon::init(&core) {
            Ok(p) => primitives_addon = p,
            Err(e) => return Err(e),
        }
        let primitives_addon = Rc::new(primitives_addon);

        event_queue.register_event_source(display.get_event_source());
        let image_addon: allegro_image::ImageAddon;
        match allegro_image::ImageAddon::init(&core) {
            Ok(i) => image_addon = i,
            Err(e) => return Err(e),
        }

        let font_addon: allegro_font::FontAddon;
        match allegro_font::FontAddon::init(&core) {
            Ok(f) => font_addon = f,
            Err(e) => return Err(e),
        }

        // load bitmaps TODO
        let mut bitmaps: Vec<allegro::Bitmap> = Vec::new();
        for i in 0..PATH_NAMES.len() {
            match allegro::Bitmap::load(&core, PATH_NAMES[i]) {
                Ok(b) => bitmaps.push(b),
                Err(_) => return Err(format!("Failed to load bitmap {}", PATH_NAMES[i])),
            }
        }

        let font: allegro_font::Font;
        match allegro_font::Font::new_builtin(&font_addon) {
            Ok(f) => font = f,
            Err(_) => return Err("Failed to create font".to_string()),
        }

        let timer: allegro::Timer;
        match allegro::Timer::new(&core, 1.0 / fps as f64) {
            Ok(t) => timer = t,
            Err(_) => return Err("Failed to create timer".to_string()),
        }

        if let Err(_) = allegro::core::Core::install_keyboard(&core) {
            return Err("Failed to install keyboard".to_string());
        }
        if let Err(_) = allegro::core::Core::install_mouse(&core) {
            return Err("Failed to install mouse".to_string());
        }
        
        event_queue.register_event_source(timer.get_event_source());
        event_queue.register_event_source(core.get_keyboard_event_source().unwrap());
        event_queue.register_event_source(core.get_mouse_event_source().unwrap());
        
        return Ok(Engine {
            core,
            event_queue,
            display,
            primitives_addon,
            _image_addon: image_addon,
            _font_addon: font_addon,
            _font: font,
            bitmaps,
            timer,
        });
    }
    pub fn start_timer(&self) {
        self.timer.start();
    }
}
