use minirogue::game::{run_game, Command, GamePhase, GameWorld};
use minirogue::types::Direction;
use minirogue::ui::{InputSource, Renderer};

/// A mock UI that feeds a pre-recorded sequence of commands.
struct MockUi {
    commands: Vec<Command>,
    index: usize,
}

impl MockUi {
    fn new(commands: Vec<Command>) -> Self {
        Self { commands, index: 0 }
    }
}

impl Renderer for MockUi {
    fn render(&mut self, _world: &GameWorld) {}
    fn render_inventory(&mut self, _world: &GameWorld) {}
    fn render_end_screen(&mut self, _world: &GameWorld) {}
}

impl InputSource for MockUi {
    fn next_command(&mut self, _phase: &GamePhase) -> Command {
        if self.index < self.commands.len() {
            let cmd = self.commands[self.index];
            self.index += 1;
            cmd
        } else {
            Command::Quit
        }
    }
}

#[test]
fn test_game_starts_and_quits() {
    let mut world = GameWorld::new(Some(42));
    let mut ui = MockUi::new(vec![Command::Quit]);
    run_game(&mut world, &mut ui);
}

#[test]
fn test_player_can_move_around() {
    let mut world = GameWorld::new(Some(42));
    let commands = vec![
        Command::Move(Direction::Right),
        Command::Move(Direction::Down),
        Command::Move(Direction::Left),
        Command::Move(Direction::Up),
        Command::Quit,
    ];
    let mut ui = MockUi::new(commands);
    run_game(&mut world, &mut ui);
}

#[test]
fn test_cheat_and_survive_many_turns() {
    let mut world = GameWorld::new(Some(42));
    let mut commands = vec![Command::Cheat];
    for _ in 0..50 {
        commands.push(Command::Move(Direction::Right));
        commands.push(Command::Move(Direction::Down));
        commands.push(Command::Move(Direction::Left));
        commands.push(Command::Move(Direction::Up));
    }
    commands.push(Command::Quit);
    let mut ui = MockUi::new(commands);
    run_game(&mut world, &mut ui);
    assert!(world.player.combat.is_alive());
}

#[test]
fn test_open_and_close_inventory() {
    let mut world = GameWorld::new(Some(42));
    let commands = vec![
        Command::OpenInventory,
        Command::CloseInventory,
        Command::Quit,
    ];
    let mut ui = MockUi::new(commands);
    run_game(&mut world, &mut ui);
    assert_eq!(world.phase, GamePhase::Exploring);
}

#[test]
fn test_deterministic_world_generation() {
    let world1 = GameWorld::new(Some(12345));
    let world2 = GameWorld::new(Some(12345));
    assert_eq!(world1.player.position, world2.player.position);
    assert_eq!(world1.monsters.len(), world2.monsters.len());
    for (m1, m2) in world1.monsters.iter().zip(world2.monsters.iter()) {
        assert_eq!(m1.position, m2.position);
        assert_eq!(m1.kind, m2.kind);
    }
}
