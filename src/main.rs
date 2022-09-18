mod game;
mod glob;
mod renderer;
mod user_cmds;
mod world;
use std::sync::Arc;
use std::sync::Mutex;
use serde_yaml;
use log::info;

/// initialize log4rs framework
fn configure_logging() {
    let config_str = include_str!("logging.yaml");
    let config = serde_yaml::from_str(config_str).unwrap();
    log4rs::init_raw_config(config).unwrap();
}
fn main() {
    configure_logging();
    let mut world = world::World::new(
        glob::types::WorldCoordinate::new(0.0, 0.0),
    );
    let user_cmds = Arc::new(Mutex::new(user_cmds::UserFeedback::new()));
    let mut renderer = renderer::Renderer::new(
        renderer::settings::Settings {
            fps: 60.0,
            screen_size: glob::types::ScreenCoordinate::new(1280.0, 720.0),
            scale: 1.0,
        },
        user_cmds.clone(),
    ).expect("Failed to initialize renderer");
    let game = game::Game::new(user_cmds.clone());
    loop {
        let update_necessary = renderer.next_frame(&world);
        if update_necessary {
            game.update(&mut world);
        }
        let exit_requested = user_cmds.lock().unwrap().exit;
        if exit_requested {
            break;
        }
    }
    info!("Quit");
}
