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

/// initialize log4rs framework
fn configure_logging() {
    let config_str = include_str!("logging.yaml");
    let config = serde_yaml::from_str(config_str).expect("Cannot parse log4rs config");
    log4rs::init_raw_config(config).expect("Cannot initialize log4rs config");
}

fn main() {
    configure_logging();
    let mut world = world::World::default();
    let mut renderer = renderer::Renderer::new(
        renderer::settings::Settings::default()
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
    log::info!("Quit");
}
