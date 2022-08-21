mod game;
mod glob;
mod renderer;
mod user_cmds;
mod world;
use std::sync::Arc;
use std::sync::Mutex;

fn main() -> Result<(), String> {
    let world = Arc::new(Mutex::new(world::World::new(
        glob::types::WorldCoordinate::new(0.0, 0.0),
    )));
    let user_cmds = Arc::new(Mutex::new(user_cmds::UserFeedback::new()));
    let mut renderer = renderer::Renderer::new(
        renderer::settings::Settings {
            fps: 60.0,
            screen_size: glob::types::ScreenCoordinate::new(1280.0, 720.0),
            scale: 1.0,
        },
        world.clone(),
        user_cmds.clone(),
    )?;
    let game = game::Game::new(world.clone(), user_cmds.clone());
    loop {
        let update_necessary = renderer.next_frame();
        if update_necessary {
            game.update();
        }
        let exit_requested = user_cmds.lock().unwrap().exit;
        if exit_requested {
            break;
        }
    }
    println!("Quit");
    Ok(())
}
