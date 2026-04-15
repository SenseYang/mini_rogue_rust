use minirogue::game::{run_game, GameWorld};
use minirogue::ui::terminal::TerminalUi;

fn main() {
    let mut ui = match TerminalUi::new() {
        Ok(ui) => ui,
        Err(e) => {
            eprintln!("Failed to initialize terminal: {}", e);
            std::process::exit(1);
        }
    };

    let mut world = GameWorld::new(None);
    run_game(&mut world, &mut ui);
}
