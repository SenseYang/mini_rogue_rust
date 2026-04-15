use crate::game::{Command, GamePhase, GameWorld};

/// Renders the game state to a display surface.
pub trait Renderer {
    /// Draw the entire game state (map, entities, HUD, messages).
    fn render(&mut self, world: &GameWorld);

    /// Display the player's inventory for item selection.
    fn render_inventory(&mut self, world: &GameWorld);

    /// Show a final screen (victory or defeat).
    fn render_end_screen(&mut self, world: &GameWorld);
}

/// Reads player input and translates it into game commands.
pub trait InputSource {
    /// Block until the player provides a command appropriate for the current phase.
    fn next_command(&mut self, phase: &GamePhase) -> Command;
}

pub mod terminal;
