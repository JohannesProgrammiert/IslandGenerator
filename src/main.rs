//! Single-threaded game
//!
//! Game loop visualization:
//!
//! ```text
//!   ┌─────► Renderer──────┐
//!   │                     ▼
//! World                 UI input
//!   ▲                     │
//!   └─────── game ◄───────┘
//! ```
//!
//! The `Renderer` paints a (readonly) world and detects user input.
//! The `game` reacts to (readonly) user input and alters the `World` accordingly
mod game;
mod glob;
mod renderer;
mod user_cmds;
mod world;
use serde_yaml;
use log::info;

/// initialize log4rs framework
fn configure_logging() {
    let config_str = include_str!("logging.yaml");
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).expect("Cannot initialize log4rs config");
}

fn main() {
    configure_logging();
    let mut world = world::World::new(
        glob::types::WorldCoordinate::new(0.0, 0.0),
    );
    let mut renderer = renderer::Renderer::new(
        renderer::settings::Settings {
            fps: 60.0,
            screen_size: glob::types::ScreenCoordinate::new(1280.0, 720.0),
            scale: 1.0,
        }
    ).expect("Failed to initialize renderer");
    loop {
        let renderer_fb = renderer.next_frame(&world);
        if renderer_fb.update_necessary {
            game::update(&mut world, &renderer_fb);
        }
        if renderer_fb.exit {
            break;
        }
    }
    info!("Quit");
}
