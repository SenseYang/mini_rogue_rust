use std::io::{self, Write};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{cursor, execute};

use crate::game::{Command, GamePhase, GameWorld};
use crate::item::Item;
use crate::types::{Direction, Position};
use crate::ui::{InputSource, Renderer};

pub struct TerminalUi;

impl TerminalUi {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), cursor::Hide)?;
        Ok(Self)
    }

    /// Determine what character to display at a given position.
    fn char_at(world: &GameWorld, pos: Position) -> char {
        // Attack mode cursor
        if let GamePhase::AttackMode { cursor } = &world.phase {
            if *cursor == pos {
                return '|';
            }
        }

        // Player
        if world.player.position == pos {
            return world.player.glyph();
        }

        // Monsters (alive only)
        for monster in &world.monsters {
            if monster.position == pos && monster.combat.is_alive() {
                return monster.glyph();
            }
        }

        // Items
        for placed in &world.items {
            if placed.position == pos {
                return placed.item.glyph();
            }
        }

        // Default: tile
        world.map.get(pos).map_or('#', |t| t.glyph())
    }
}

impl Drop for TerminalUi {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

impl Renderer for TerminalUi {
    fn render(&mut self, world: &GameWorld) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All));

        // Draw map
        for y in 0..world.map.height() {
            for x in 0..world.map.width() {
                let ch = Self::char_at(world, Position::new(x as i32, y as i32));
                let _ = write!(stdout, "{}", ch);
            }
            let _ = write!(stdout, "\r\n");
        }

        // HUD
        let weapon_name = world
            .player
            .equipped_weapon
            .map_or("None", |w| w.name());
        let _ = write!(
            stdout,
            "HP: {}/{} | STR: {} | DEX: {} | LVL: {} | XP: {}/{} | Floor: {}/{} | Weapon: {}\r\n",
            world.player.combat.hp,
            world.player.combat.max_hp,
            world.player.combat.strength,
            world.player.combat.dexterity,
            world.player.progression.level,
            world.player.progression.experience,
            world.player.progression.xp_to_next_level(),
            world.current_level,
            world.total_levels,
            weapon_name,
        );

        // Status effects
        let mut status = String::new();
        if world.player.invisible_turns > 0 {
            status.push_str(&format!("Invisible({}) ", world.player.invisible_turns));
        }
        if world.player.charmed_turns > 0 {
            status.push_str(&format!("Charmed({}) ", world.player.charmed_turns));
        }
        if !status.is_empty() {
            let _ = write!(stdout, "Status: {}\r\n", status);
        }

        // Last message
        if let Some(msg) = world.messages.last() {
            let _ = write!(stdout, "> {}\r\n", msg);
        }

        // Phase indicator
        match &world.phase {
            GamePhase::AttackMode { .. } => {
                let _ = write!(
                    stdout,
                    "[ATTACK MODE] WASD to aim, F to attack, Q to cancel\r\n"
                );
            }
            _ => {}
        }

        let _ = stdout.flush();
    }

    fn render_inventory(&mut self, world: &GameWorld) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All));

        let _ = write!(stdout, "=== INVENTORY ===\r\n");

        if world.player.inventory.is_empty() {
            let _ = write!(stdout, "  (empty)\r\n");
        } else {
            for (i, item) in world.player.inventory.iter().enumerate() {
                let equipped_marker = match item {
                    Item::Weapon(kind) if Some(*kind) == world.player.equipped_weapon => {
                        " [EQUIPPED]"
                    }
                    _ => "",
                };
                let _ = write!(
                    stdout,
                    "  {}: {} {}{}\r\n",
                    i + 1,
                    item.glyph(),
                    item.name(),
                    equipped_marker,
                );
            }
        }

        let _ = write!(
            stdout,
            "\r\nPress 1-{} to use/equip, Q to close\r\n",
            world.player.inventory.len().max(1)
        );
        let _ = stdout.flush();
    }

    fn render_end_screen(&mut self, world: &GameWorld) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All));

        match world.phase {
            GamePhase::Victory => {
                let _ = write!(stdout, "=== VICTORY! ===\r\n");
                let _ = write!(stdout, "You escaped the dungeon!\r\n\r\n");
            }
            GamePhase::GameOver => {
                let _ = write!(stdout, "=== GAME OVER ===\r\n");
                let _ = write!(stdout, "You have perished in the dungeon.\r\n\r\n");
            }
            _ => {}
        }

        let _ = write!(stdout, "Final Stats:\r\n");
        let _ = write!(
            stdout,
            "  Level: {}\r\n",
            world.player.progression.level
        );
        let _ = write!(
            stdout,
            "  Floor reached: {}/{}\r\n",
            world.current_level, world.total_levels
        );
        let _ = write!(stdout, "\r\nPress any key to exit.\r\n");
        let _ = stdout.flush();
    }
}

impl InputSource for TerminalUi {
    fn next_command(&mut self, phase: &GamePhase) -> Command {
        loop {
            if let Ok(Event::Key(key_event)) = event::read() {
                // On Windows, crossterm fires Press, Repeat, and Release. Only react to Press.
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                let cmd = match phase {
                    GamePhase::Exploring => match key_event.code {
                        KeyCode::Char('w') | KeyCode::Up => Some(Command::Move(Direction::Up)),
                        KeyCode::Char('a') | KeyCode::Left => {
                            Some(Command::Move(Direction::Left))
                        }
                        KeyCode::Char('s') | KeyCode::Down => {
                            Some(Command::Move(Direction::Down))
                        }
                        KeyCode::Char('d') | KeyCode::Right => {
                            Some(Command::Move(Direction::Right))
                        }
                        KeyCode::Char('f') => Some(Command::EnterAttackMode),
                        KeyCode::Char('p') => Some(Command::PickUp),
                        KeyCode::Char('i') => Some(Command::OpenInventory),
                        KeyCode::Char('c') => Some(Command::Cheat),
                        KeyCode::Char('q') | KeyCode::Esc => Some(Command::Quit),
                        _ => None,
                    },
                    GamePhase::AttackMode { .. } => match key_event.code {
                        KeyCode::Char('w') | KeyCode::Up => {
                            Some(Command::MoveCursor(Direction::Up))
                        }
                        KeyCode::Char('a') | KeyCode::Left => {
                            Some(Command::MoveCursor(Direction::Left))
                        }
                        KeyCode::Char('s') | KeyCode::Down => {
                            Some(Command::MoveCursor(Direction::Down))
                        }
                        KeyCode::Char('d') | KeyCode::Right => {
                            Some(Command::MoveCursor(Direction::Right))
                        }
                        KeyCode::Char('f') | KeyCode::Enter => Some(Command::ConfirmAttack),
                        KeyCode::Char('q') | KeyCode::Esc => Some(Command::CancelAttack),
                        _ => None,
                    },
                    GamePhase::ViewingInventory => match key_event.code {
                        KeyCode::Char('q') | KeyCode::Esc => Some(Command::CloseInventory),
                        KeyCode::Char(c) if c.is_ascii_digit() => {
                            let slot = if c == '0' { 9 } else { (c as usize) - ('1' as usize) };
                            Some(Command::UseItem(slot))
                        }
                        _ => None,
                    },
                    GamePhase::GameOver | GamePhase::Victory => Some(Command::Quit),
                };

                if let Some(cmd) = cmd {
                    return cmd;
                }
            }
        }
    }
}
