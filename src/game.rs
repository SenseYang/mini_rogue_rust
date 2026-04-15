use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;

use crate::combat::{resolve_attack, AttackResult};
use crate::dungeon::{generate_level, Room, MAP_HEIGHT, MAP_WIDTH, TOTAL_LEVELS};
use crate::entity::monster::{Monster, MonsterKind};
use crate::entity::player::Player;
use crate::item::scroll::ScrollKind;
use crate::item::weapon::WeaponKind;
use crate::item::{Item, PlacedItem};
use crate::types::{Direction, Grid, Position, Tile};
use crate::ui::{InputSource, Renderer};

/// The current phase of the game, modeled as a state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    Exploring,
    AttackMode { cursor: Position },
    ViewingInventory,
    GameOver,
    Victory,
}

/// A fully-resolved game command, independent of input method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    // Exploring phase
    Move(Direction),
    EnterAttackMode,
    PickUp,
    OpenInventory,
    Cheat,
    Quit,

    // Attack mode
    MoveCursor(Direction),
    ConfirmAttack,
    CancelAttack,

    // Inventory
    UseItem(usize),
    CloseInventory,
}

/// The complete game state.
pub struct GameWorld {
    pub map: Grid<Tile>,
    pub player: Player,
    pub monsters: Vec<Monster>,
    pub items: Vec<PlacedItem>,
    pub current_level: u32,
    pub total_levels: u32,
    pub phase: GamePhase,
    pub messages: Vec<String>,
    pub rng: StdRng,
    rooms: Vec<Room>,
}

impl GameWorld {
    /// Create a new game. Pass `Some(seed)` for deterministic games, `None` for random.
    pub fn new(seed: Option<u64>) -> Self {
        let mut rng = match seed {
            Some(s) => StdRng::seed_from_u64(s),
            None => StdRng::from_entropy(),
        };

        let (map, rooms) = generate_level(MAP_WIDTH, MAP_HEIGHT, &mut rng);
        let player_pos = rooms[0].center();
        let player = Player::new(player_pos);

        let mut occupied = HashSet::new();
        occupied.insert(player_pos);
        if let Some(last) = rooms.last() {
            occupied.insert(last.center()); // stairway
        }

        let monsters = spawn_monsters(1, &rooms, &mut occupied, &map, &mut rng);
        let items = spawn_items(1, &rooms, &mut occupied, &map, &mut rng);

        Self {
            map,
            player,
            monsters,
            items,
            current_level: 1,
            total_levels: TOTAL_LEVELS,
            phase: GamePhase::Exploring,
            messages: vec!["Welcome to MiniRogue! Find the exit on level 5.".to_string()],
            rng,
            rooms,
        }
    }
}

/// Run the game to completion.
pub fn run_game<U: Renderer + InputSource>(world: &mut GameWorld, ui: &mut U) {
    loop {
        // Render based on current phase
        match &world.phase {
            GamePhase::ViewingInventory => ui.render_inventory(world),
            GamePhase::GameOver | GamePhase::Victory => {
                ui.render_end_screen(world);
                let _ = ui.next_command(&world.phase);
                break;
            }
            _ => ui.render(world),
        }

        let command = ui.next_command(&world.phase);

        if command == Command::Quit {
            break;
        }

        let turn_consumed = process_command(world, command);

        // Monster turns only when a turn was consumed during exploration
        if turn_consumed && world.phase == GamePhase::Exploring {
            process_monster_turns(world);
            tick_status_effects(world);
            check_end_conditions(world);
        }
    }
}

// ---------- Command processing ----------

/// Process a single command. Returns true if the action consumed a turn.
fn process_command(world: &mut GameWorld, command: Command) -> bool {
    let phase = world.phase.clone();
    match (phase, command) {
        // --- Exploring ---
        (GamePhase::Exploring, Command::Move(dir)) => process_move(world, dir),
        (GamePhase::Exploring, Command::EnterAttackMode) => {
            let range = world
                .player
                .equipped_weapon
                .map(|w| w.stats().attack_range)
                .unwrap_or(1);
            // Check there's at least one monster in range
            let has_target = world
                .monsters
                .iter()
                .any(|m| m.combat.is_alive() && world.player.position.distance_to(m.position) <= range);
            if has_target {
                world.phase = GamePhase::AttackMode {
                    cursor: world.player.position,
                };
            } else {
                world.messages.push("No enemies in range.".to_string());
            }
            false
        }
        (GamePhase::Exploring, Command::PickUp) => process_pickup(world),
        (GamePhase::Exploring, Command::OpenInventory) => {
            world.phase = GamePhase::ViewingInventory;
            false
        }
        (GamePhase::Exploring, Command::Cheat) => {
            world.player.apply_cheat();
            world.messages.push("Cheat mode activated!".to_string());
            true
        }

        // --- Attack Mode ---
        (GamePhase::AttackMode { cursor }, Command::MoveCursor(dir)) => {
            let new_cursor = cursor.step(dir);
            let range = world
                .player
                .equipped_weapon
                .map(|w| w.stats().attack_range)
                .unwrap_or(1);
            if world.player.position.distance_to(new_cursor) <= range
                && world.map.in_bounds(new_cursor)
            {
                world.phase = GamePhase::AttackMode {
                    cursor: new_cursor,
                };
            }
            false
        }
        (GamePhase::AttackMode { cursor }, Command::ConfirmAttack) => {
            world.phase = GamePhase::Exploring;
            process_attack(world, cursor)
        }
        (GamePhase::AttackMode { .. }, Command::CancelAttack) => {
            world.phase = GamePhase::Exploring;
            false
        }

        // --- Inventory ---
        (GamePhase::ViewingInventory, Command::UseItem(slot)) => {
            process_use_item(world, slot)
        }
        (GamePhase::ViewingInventory, Command::CloseInventory) => {
            world.phase = GamePhase::Exploring;
            false
        }

        _ => false,
    }
}

fn process_move(world: &mut GameWorld, dir: Direction) -> bool {
    let new_pos = world.player.position.step(dir);

    // Check bounds and walkability
    match world.map.get(new_pos) {
        Some(tile) if tile.is_walkable() => {}
        _ => return false,
    }

    // Check monster collision
    if world
        .monsters
        .iter()
        .any(|m| m.position == new_pos && m.combat.is_alive())
    {
        world
            .messages
            .push("A monster blocks your path!".to_string());
        return false;
    }

    world.player.position = new_pos;

    // Check stairway
    if world.map.get(new_pos) == Some(&Tile::Stairway) {
        if world.current_level >= world.total_levels {
            world.phase = GamePhase::Victory;
        } else {
            advance_level(world);
        }
    }

    true
}

fn process_pickup(world: &mut GameWorld) -> bool {
    let player_pos = world.player.position;

    let found = world
        .items
        .iter()
        .position(|placed| placed.position == player_pos);

    match found {
        Some(index) if world.player.can_pick_up() => {
            let picked = world.items.swap_remove(index);
            world
                .messages
                .push(format!("Picked up {}.", picked.item.name()));
            world.player.inventory.push(picked.item);
            true
        }
        Some(_) => {
            world.messages.push("Inventory is full!".to_string());
            false
        }
        None => {
            world
                .messages
                .push("Nothing to pick up here.".to_string());
            false
        }
    }
}

fn process_attack(world: &mut GameWorld, target_pos: Position) -> bool {
    // Find monster at target
    let monster_idx = world
        .monsters
        .iter()
        .position(|m| m.position == target_pos && m.combat.is_alive());

    let monster_idx = match monster_idx {
        Some(i) => i,
        None => {
            world.messages.push("Nothing to attack there.".to_string());
            return true; // turn consumed even on whiff
        }
    };

    let weapon_stats = world.player.equipped_weapon.map(|w| w.stats());
    let result = resolve_attack(
        &world.player.combat,
        &world.monsters[monster_idx].combat,
        weapon_stats.as_ref(),
        &mut world.rng,
    );

    let monster_name = world.monsters[monster_idx].kind.name();

    match result {
        AttackResult::Miss => {
            world
                .messages
                .push(format!("You attack {} — miss!", monster_name));
        }
        AttackResult::Hit { damage } => {
            world.monsters[monster_idx].combat.take_damage(damage);
            world
                .messages
                .push(format!("You hit {} for {} damage!", monster_name, damage));
        }
        AttackResult::HitAndCharm { damage } => {
            world.monsters[monster_idx].combat.take_damage(damage);
            world.monsters[monster_idx].charmed_turns = 3;
            world.messages.push(format!(
                "You hit {} for {} damage and charmed it!",
                monster_name, damage
            ));
        }
    }

    // Check death
    if !world.monsters[monster_idx].combat.is_alive() {
        let kind = world.monsters[monster_idx].kind;
        let xp = kind.xp_reward();
        let levels = world.player.grant_xp(xp);
        world.messages.push(format!(
            "{} defeated! +{} XP{}",
            kind.name(),
            xp,
            if levels > 0 {
                format!(" — Level up! (Lv. {})", world.player.progression.level)
            } else {
                String::new()
            }
        ));

        // Item drop (30% chance)
        let drop_pos = world.monsters[monster_idx].position;
        if world.rng.gen_range(0..100) < 30 {
            let item = random_item(&mut world.rng);
            world.items.push(PlacedItem {
                position: drop_pos,
                item,
            });
            world
                .messages
                .push(format!("{} dropped a {}!", kind.name(), item.name()));
        }
    }

    true
}

fn process_use_item(world: &mut GameWorld, slot: usize) -> bool {
    if slot >= world.player.inventory.len() {
        return false;
    }

    let item = world.player.inventory[slot];

    match item {
        Item::Scroll(scroll_kind) => {
            apply_scroll(world, scroll_kind);
            world.player.inventory.remove(slot);
            world.phase = GamePhase::Exploring;
            true
        }
        Item::Weapon(weapon_kind) => {
            // Swap equipped weapon
            if let Some(old_weapon) = world.player.equipped_weapon.take() {
                world.player.inventory[slot] = Item::Weapon(old_weapon);
            } else {
                world.player.inventory.remove(slot);
            }
            world.player.equipped_weapon = Some(weapon_kind);
            world
                .messages
                .push(format!("Equipped {}.", weapon_kind.name()));
            world.phase = GamePhase::Exploring;
            true
        }
    }
}

fn apply_scroll(world: &mut GameWorld, kind: ScrollKind) {
    match kind {
        ScrollKind::Health => {
            world.player.combat.heal(10);
            world.messages.push("Restored 10 HP.".to_string());
        }
        ScrollKind::Dexterity => {
            world.player.combat.dexterity += 3;
            world.messages.push("Dexterity increased by 3!".to_string());
        }
        ScrollKind::Strength => {
            world.player.combat.strength += 3;
            world.messages.push("Strength increased by 3!".to_string());
        }
        ScrollKind::LevelUp => {
            let _levels = world.player.grant_xp(world.player.progression.xp_to_next_level());
            world.messages.push(format!(
                "Level up! (Lv. {})",
                world.player.progression.level
            ));
        }
        ScrollKind::Invisible => {
            world.player.invisible_turns = 5;
            world
                .messages
                .push("You are invisible for 5 turns!".to_string());
        }
    }
}

// ---------- Monster AI ----------

fn process_monster_turns(world: &mut GameWorld) {
    let player_pos = world.player.position;
    let player_invisible = world.player.is_invisible();

    // Collect monster positions to avoid collisions
    let monster_count = world.monsters.len();

    for i in 0..monster_count {
        if !world.monsters[i].combat.is_alive() {
            continue;
        }

        // Charm check
        if world.monsters[i].charmed_turns > 0 {
            world.monsters[i].charmed_turns -= 1;
            continue;
        }

        let dist = world.monsters[i].position.distance_to(player_pos);
        let kind = world.monsters[i].kind;

        // Attack check
        if dist <= kind.attack_range() && !player_invisible {
            let result = resolve_attack(
                &world.monsters[i].combat,
                &world.player.combat,
                None,
                &mut world.rng,
            );

            match result {
                AttackResult::Miss => {
                    world
                        .messages
                        .push(format!("{} attacks — miss!", kind.name()));
                }
                AttackResult::Hit { damage } => {
                    world.player.combat.take_damage(damage);
                    world.messages.push(format!(
                        "{} hits you for {} damage!",
                        kind.name(),
                        damage
                    ));
                }
                AttackResult::HitAndCharm { damage } => {
                    world.player.combat.take_damage(damage);
                    world.messages.push(format!(
                        "{} hits you for {} damage!",
                        kind.name(),
                        damage
                    ));
                }
            }
            continue;
        }

        // Smell/track
        if dist <= kind.smell_range() {
            let dx = player_pos.x - world.monsters[i].position.x;
            let dy = player_pos.y - world.monsters[i].position.y;

            // Prefer horizontal if tie
            let dir = if dx.abs() >= dy.abs() {
                if dx > 0 {
                    Direction::Right
                } else {
                    Direction::Left
                }
            } else if dy > 0 {
                Direction::Down
            } else {
                Direction::Up
            };

            let new_pos = world.monsters[i].position.step(dir);

            // Check walkability and collisions
            let walkable = world
                .map
                .get(new_pos)
                .map_or(false, |t| t.is_walkable());
            let blocked_by_player = new_pos == player_pos;
            let blocked_by_monster = world
                .monsters
                .iter()
                .enumerate()
                .any(|(j, m)| j != i && m.position == new_pos && m.combat.is_alive());

            if walkable && !blocked_by_player && !blocked_by_monster {
                world.monsters[i].position = new_pos;
            }
        }
    }
}

// ---------- Status effects ----------

fn tick_status_effects(world: &mut GameWorld) {
    if world.player.invisible_turns > 0 {
        world.player.invisible_turns -= 1;
        if world.player.invisible_turns == 0 {
            world.messages.push("You are no longer invisible.".to_string());
        }
    }
}

// ---------- End conditions ----------

fn check_end_conditions(world: &mut GameWorld) {
    if !world.player.combat.is_alive() {
        world.phase = GamePhase::GameOver;
    }
}

// ---------- Level advancement ----------

fn advance_level(world: &mut GameWorld) {
    world.current_level += 1;

    let (map, rooms) = generate_level(MAP_WIDTH, MAP_HEIGHT, &mut world.rng);
    world.map = map;
    world.rooms = rooms;

    let player_pos = world.rooms[0].center();
    world.player.position = player_pos;

    let mut occupied = HashSet::new();
    occupied.insert(player_pos);
    if let Some(last) = world.rooms.last() {
        occupied.insert(last.center());
    }

    world.monsters = spawn_monsters(
        world.current_level,
        &world.rooms,
        &mut occupied,
        &world.map,
        &mut world.rng,
    );
    world.items = spawn_items(
        world.current_level,
        &world.rooms,
        &mut occupied,
        &world.map,
        &mut world.rng,
    );

    world.messages.push(format!(
        "You descend to level {}/{}.",
        world.current_level, world.total_levels
    ));
}

// ---------- Spawning helpers ----------

fn spawn_monsters(
    level: u32,
    rooms: &[Room],
    occupied: &mut HashSet<Position>,
    map: &Grid<Tile>,
    rng: &mut impl Rng,
) -> Vec<Monster> {
    let count = (2 + level * 2) as usize;
    let available_kinds = MonsterKind::available_at(level);
    if available_kinds.is_empty() {
        return Vec::new();
    }

    let mut monsters = Vec::new();
    let floor_tiles = collect_floor_tiles(rooms, occupied, map);

    for &pos in floor_tiles.iter().take(count) {
        let kind = available_kinds[rng.gen_range(0..available_kinds.len())];
        monsters.push(Monster::new(kind, pos));
        occupied.insert(pos);
    }

    monsters
}

fn spawn_items(
    level: u32,
    rooms: &[Room],
    occupied: &mut HashSet<Position>,
    map: &Grid<Tile>,
    rng: &mut impl Rng,
) -> Vec<PlacedItem> {
    let count = (1 + level) as usize;
    let mut items = Vec::new();
    let floor_tiles = collect_floor_tiles(rooms, occupied, map);

    for &pos in floor_tiles.iter().take(count) {
        let item = random_item(rng);
        items.push(PlacedItem { position: pos, item });
        occupied.insert(pos);
    }

    items
}

fn collect_floor_tiles(
    rooms: &[Room],
    occupied: &HashSet<Position>,
    map: &Grid<Tile>,
) -> Vec<Position> {
    rooms
        .iter()
        .flat_map(|r| r.floor_positions())
        .filter(|p| {
            !occupied.contains(p) && map.get(*p).map_or(false, |t| *t == Tile::Floor)
        })
        .collect()
}

fn random_item(rng: &mut impl Rng) -> Item {
    let weapons = WeaponKind::all();
    let scrolls = ScrollKind::all();
    let total = weapons.len() + scrolls.len();
    let idx = rng.gen_range(0..total);

    if idx < weapons.len() {
        Item::Weapon(weapons[idx])
    } else {
        Item::Scroll(scrolls[idx - weapons.len()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_game_world() {
        let world = GameWorld::new(Some(42));
        assert_eq!(world.current_level, 1);
        assert_eq!(world.total_levels, TOTAL_LEVELS);
        assert_eq!(world.phase, GamePhase::Exploring);
        assert!(world.player.combat.is_alive());
        assert!(!world.monsters.is_empty());
    }

    #[test]
    fn test_player_move() {
        let mut world = GameWorld::new(Some(42));
        let _start_pos = world.player.position;

        // Try all four directions, at least one should succeed (player starts in a room)
        let moved = process_command(&mut world, Command::Move(Direction::Right))
            || process_command(&mut world, Command::Move(Direction::Down))
            || process_command(&mut world, Command::Move(Direction::Left))
            || process_command(&mut world, Command::Move(Direction::Up));

        assert!(moved, "Player couldn't move in any direction from starting position");
    }

    #[test]
    fn test_player_blocked_by_wall() {
        let mut world = GameWorld::new(Some(42));
        // Move to edge of map (should hit a wall)
        world.player.position = Position::new(0, 0);
        let moved = process_command(&mut world, Command::Move(Direction::Up));
        assert!(!moved);
    }

    #[test]
    fn test_enter_attack_mode() {
        let mut world = GameWorld::new(Some(42));
        // Place a monster adjacent to player
        let adj = world.player.position.step(Direction::Right);
        world
            .monsters
            .push(Monster::new(MonsterKind::Goblin, adj));

        let consumed = process_command(&mut world, Command::EnterAttackMode);
        assert!(!consumed); // Entering attack mode doesn't consume a turn
        assert!(matches!(world.phase, GamePhase::AttackMode { .. }));
    }

    #[test]
    fn test_cancel_attack_mode() {
        let mut world = GameWorld::new(Some(42));
        world.phase = GamePhase::AttackMode {
            cursor: world.player.position,
        };

        let consumed = process_command(&mut world, Command::CancelAttack);
        assert!(!consumed);
        assert_eq!(world.phase, GamePhase::Exploring);
    }

    #[test]
    fn test_pickup_item() {
        let mut world = GameWorld::new(Some(42));
        let player_pos = world.player.position;
        world.items.push(PlacedItem {
            position: player_pos,
            item: Item::Weapon(WeaponKind::Sword),
        });

        let consumed = process_command(&mut world, Command::PickUp);
        assert!(consumed);
        assert_eq!(world.player.inventory.len(), 1);
        assert!(world.items.iter().all(|i| i.position != player_pos));
    }

    #[test]
    fn test_pickup_nothing() {
        let mut world = GameWorld::new(Some(42));
        // Clear items at player pos
        let player_pos = world.player.position;
        world.items.retain(|i| i.position != player_pos);

        let consumed = process_command(&mut world, Command::PickUp);
        assert!(!consumed);
    }

    #[test]
    fn test_use_scroll() {
        let mut world = GameWorld::new(Some(42));
        world.player.combat.hp = 10;
        world
            .player
            .inventory
            .push(Item::Scroll(ScrollKind::Health));
        world.phase = GamePhase::ViewingInventory;

        let consumed = process_command(&mut world, Command::UseItem(0));
        assert!(consumed);
        assert_eq!(world.player.combat.hp, 20);
        assert!(world.player.inventory.is_empty());
        assert_eq!(world.phase, GamePhase::Exploring);
    }

    #[test]
    fn test_equip_weapon() {
        let mut world = GameWorld::new(Some(42));
        world
            .player
            .inventory
            .push(Item::Weapon(WeaponKind::Sword));
        world.phase = GamePhase::ViewingInventory;

        let consumed = process_command(&mut world, Command::UseItem(0));
        assert!(consumed);
        assert_eq!(world.player.equipped_weapon, Some(WeaponKind::Sword));
        assert_eq!(world.phase, GamePhase::Exploring);
    }

    #[test]
    fn test_equip_weapon_swaps_old() {
        let mut world = GameWorld::new(Some(42));
        world.player.equipped_weapon = Some(WeaponKind::Axe);
        world
            .player
            .inventory
            .push(Item::Weapon(WeaponKind::Sword));
        world.phase = GamePhase::ViewingInventory;

        let consumed = process_command(&mut world, Command::UseItem(0));
        assert!(consumed);
        assert_eq!(world.player.equipped_weapon, Some(WeaponKind::Sword));
        // Old weapon should be in inventory
        assert!(world
            .player
            .inventory
            .contains(&Item::Weapon(WeaponKind::Axe)));
    }

    #[test]
    fn test_cheat_mode() {
        let mut world = GameWorld::new(Some(42));
        let consumed = process_command(&mut world, Command::Cheat);
        assert!(consumed);
        assert_eq!(world.player.combat.hp, 999);
        assert_eq!(world.player.combat.strength, 99);
    }

    #[test]
    fn test_monster_turn_charmed_skip() {
        let mut world = GameWorld::new(Some(42));
        // Remove existing monsters and add one charmed monster near player
        world.monsters.clear();
        let adj = world.player.position.step(Direction::Right);
        let mut m = Monster::new(MonsterKind::Goblin, adj);
        m.charmed_turns = 3;
        world.monsters.push(m);

        let old_hp = world.player.combat.hp;
        process_monster_turns(&mut world);

        // Monster should not have attacked (it's charmed)
        assert_eq!(world.player.combat.hp, old_hp);
        assert_eq!(world.monsters[0].charmed_turns, 2);
    }

    #[test]
    fn test_invisible_player_not_attacked() {
        let mut world = GameWorld::new(Some(42));
        world.monsters.clear();
        let adj = world.player.position.step(Direction::Right);
        world
            .monsters
            .push(Monster::new(MonsterKind::Goblin, adj));
        world.player.invisible_turns = 5;

        let old_hp = world.player.combat.hp;
        process_monster_turns(&mut world);
        // Monster is adjacent but player is invisible — should not attack
        assert_eq!(world.player.combat.hp, old_hp);
    }

    #[test]
    fn test_game_over_on_death() {
        let mut world = GameWorld::new(Some(42));
        world.player.combat.hp = 0;
        check_end_conditions(&mut world);
        assert_eq!(world.phase, GamePhase::GameOver);
    }

    #[test]
    fn test_victory_on_final_stairway() {
        let mut world = GameWorld::new(Some(42));
        world.current_level = world.total_levels;

        // Find the stairway position
        let mut stairway_pos = None;
        for y in 0..world.map.height() {
            for x in 0..world.map.width() {
                let pos = Position::new(x as i32, y as i32);
                if world.map.get(pos) == Some(&Tile::Stairway) {
                    stairway_pos = Some(pos);
                }
            }
        }

        if let Some(pos) = stairway_pos {
            // Place player adjacent to stairway
            world.player.position = pos.step(Direction::Left);
            // Clear any monsters at the stairway
            world.monsters.retain(|m| m.position != pos);

            // Ensure the tile to the left of stairway is walkable
            if let Some(tile) = world.map.get_mut(world.player.position) {
                *tile = Tile::Floor;
            }

            let _ = process_command(&mut world, Command::Move(Direction::Right));
            assert_eq!(world.phase, GamePhase::Victory);
        }
    }

    #[test]
    fn test_advance_level() {
        let mut world = GameWorld::new(Some(42));
        let old_level = world.current_level;
        advance_level(&mut world);
        assert_eq!(world.current_level, old_level + 1);
        assert!(world.player.combat.is_alive());
    }

    #[test]
    fn test_random_item_distribution() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut weapons = 0;
        let mut scrolls = 0;
        for _ in 0..1000 {
            match random_item(&mut rng) {
                Item::Weapon(_) => weapons += 1,
                Item::Scroll(_) => scrolls += 1,
            }
        }
        // 4 weapons + 5 scrolls = 9 total. Expect roughly 4/9 weapons.
        assert!(weapons > 300 && weapons < 600, "weapons: {}", weapons);
        assert!(scrolls > 300 && scrolls < 600, "scrolls: {}", scrolls);
    }

    #[test]
    fn test_status_effect_tick() {
        let mut world = GameWorld::new(Some(42));
        world.player.invisible_turns = 2;
        tick_status_effects(&mut world);
        assert_eq!(world.player.invisible_turns, 1);
        tick_status_effects(&mut world);
        assert_eq!(world.player.invisible_turns, 0);
    }
}
