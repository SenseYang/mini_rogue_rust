# MiniRogue — Rust Design Document

## 1. Introduction

This document describes the software architecture for MiniRogue, a terminal-based roguelike game, implemented in **idiomatic Rust**. It is written for a developer who is new to Rust and wants to learn the language through a real project.

### 1.1 Learning Goals

By building this project you will practice:

- **Enums with data** — Rust's most powerful feature for modeling game entities and states.
- **Pattern matching** — exhaustive `match` expressions to handle every variant.
- **Ownership and borrowing** — structuring data so the borrow checker is your ally, not your enemy.
- **Traits** — defining shared behavior (platform abstraction, display formatting).
- **Generics** — writing a game loop that works with any renderer or input source.
- **Modules** — organizing code into a clean, navigable crate structure.
- **`Option<T>` and `Result<T, E>`** — eliminating null-pointer bugs by design.
- **Iterators and closures** — expressive, zero-cost data processing.
- **Derive macros** — automatic `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`.
- **Testing** — unit tests, integration tests, and deterministic RNG seeding.

### 1.2 Crate Dependencies

| Crate | Purpose |
| :--- | :--- |
| [`rand`](https://crates.io/crates/rand) | Random number generation (dungeon layout, combat rolls, item drops). |
| [`crossterm`](https://crates.io/crates/crossterm) | Cross-platform terminal manipulation (raw mode, cursor control, colors). |

Both are mature, widely-used crates. Keep the dependency list minimal — part of learning Rust is using the standard library effectively.

---

## 2. Project Structure

```
minirogue/
├── Cargo.toml
├── src/
│   ├── main.rs              # Entry point: creates UI backend, calls run_game()
│   ├── lib.rs               # Crate root: declares modules, re-exports public API
│   │
│   ├── types.rs             # Position, Direction, Grid<T>
│   ├── combat.rs            # Hit/damage resolution, charm logic
│   ├── dungeon.rs           # Procedural dungeon generation algorithm
│   ├── game.rs              # GameWorld, GamePhase, turn loop, win/loss checks
│   │
│   ├── entity/
│   │   ├── mod.rs           # CombatStats, shared entity types
│   │   ├── player.rs        # Player struct, leveling, inventory
│   │   └── monster.rs       # Monster struct, MonsterKind, AI behavior
│   │
│   ├── item/
│   │   ├── mod.rs           # Item enum (Weapon / Scroll)
│   │   ├── weapon.rs        # Weapon struct, WeaponKind, stat tables
│   │   └── scroll.rs        # ScrollKind, effect application
│   │
│   └── ui/
│       ├── mod.rs           # Renderer and InputSource trait definitions
│       └── terminal.rs      # CLI implementation using crossterm
```

### 2.1 Why This Layout?

- **`lib.rs` + `main.rs` split:** The library crate (`lib.rs`) contains all game logic and is independently testable. The binary crate (`main.rs`) is a thin wrapper that wires up the terminal UI and calls into the library. This separation also enables the future Android port to link against the same library crate.
- **`entity/` and `item/` sub-modules:** Group related types together. Each sub-module is small and focused on one concept.
- **`ui/` module:** Isolates all platform-specific code behind traits. The rest of the crate never imports `crossterm` directly.

---

## 3. Core Types (`types.rs`)

### 3.1 Position

```rust
/// A 2D coordinate on the dungeon map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Manhattan distance to another position.
    pub fn distance_to(self, other: Position) -> i32 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// Returns the position after moving one step in the given direction.
    pub fn step(self, dir: Direction) -> Self {
        match dir {
            Direction::Up    => Self { x: self.x, y: self.y - 1 },
            Direction::Down  => Self { x: self.x, y: self.y + 1 },
            Direction::Left  => Self { x: self.x - 1, y: self.y },
            Direction::Right => Self { x: self.x + 1, y: self.y },
        }
    }
}
```

**Rust idioms demonstrated:**
- `#[derive(...)]` — automatically generates trait implementations. `Copy` is used because `Position` is small (two integers) and should be passed by value.
- `Self` — idiomatic shorthand for the implementing type.
- `self` by value — since `Position` is `Copy`, methods consume a copy rather than borrowing.

### 3.2 Direction

```rust
/// The four cardinal movement directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}
```

### 3.3 Grid — A Generic 2D Container

```rust
/// A fixed-size 2D grid stored as a flat Vec for cache-friendly access.
#[derive(Debug, Clone)]
pub struct Grid<T> {
    width: usize,
    height: usize,
    cells: Vec<T>,
}

impl<T: Clone> Grid<T> {
    /// Create a grid filled with a default value.
    pub fn new(width: usize, height: usize, default: T) -> Self {
        Self {
            width,
            height,
            cells: vec![default; width * height],
        }
    }

    pub fn width(&self) -> usize { self.width }
    pub fn height(&self) -> usize { self.height }

    /// Returns true if the position is within bounds.
    pub fn in_bounds(&self, pos: Position) -> bool {
        pos.x >= 0
            && pos.y >= 0
            && (pos.x as usize) < self.width
            && (pos.y as usize) < self.height
    }

    /// Returns a reference to the cell, or None if out of bounds.
    pub fn get(&self, pos: Position) -> Option<&T> {
        if self.in_bounds(pos) {
            Some(&self.cells[pos.y as usize * self.width + pos.x as usize])
        } else {
            None
        }
    }

    /// Returns a mutable reference to the cell, or None if out of bounds.
    pub fn get_mut(&mut self, pos: Position) -> Option<&mut T> {
        if self.in_bounds(pos) {
            let idx = pos.y as usize * self.width + pos.x as usize;
            Some(&mut self.cells[idx])
        } else {
            None
        }
    }
}
```

**Rust idioms demonstrated:**
- **Generics (`<T>`)** — `Grid` works with any cell type: `Grid<Tile>` for the map, or even `Grid<bool>` for visited tracking.
- **Flat `Vec<T>` with index math** — preferred over `Vec<Vec<T>>` because it prevents ragged rows, has better cache locality, and maintains a single invariant (`width * height == cells.len()`).
- **`Option<&T>` return** — Rust's alternative to returning null. Callers must handle the `None` case, preventing out-of-bounds bugs at compile time.
- **Trait bounds (`T: Clone`)** — the `vec![default; n]` macro requires `Clone`, so we express that constraint in the `impl` block.

---

## 4. Map Tiles (`types.rs` or `dungeon.rs`)

```rust
/// A single tile on the dungeon map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Wall,
    Floor,
    Stairway,
}

impl Tile {
    /// Whether actors can walk on this tile.
    pub fn is_walkable(self) -> bool {
        matches!(self, Tile::Floor | Tile::Stairway)
    }

    /// The character used to render this tile.
    pub fn glyph(self) -> char {
        match self {
            Tile::Wall     => '#',
            Tile::Floor    => ' ',
            Tile::Stairway => '+',
        }
    }
}
```

**Rust idioms demonstrated:**
- **Enum with methods** — instead of a `Renderable` trait, we put `glyph()` directly on the enum. This is idiomatic for *closed* sets (we'll never add tile types at runtime). Traits are for *open* extension points.
- **`matches!` macro** — a concise way to check if a value matches one or more patterns. Returns `bool`.
- **Exhaustive `match`** — the compiler guarantees every variant is handled. If you add `Tile::Lava` later, every `match` on `Tile` will produce a compile error until updated.

---

## 5. Entity System (`entity/`)

### 5.1 Combat Stats (`entity/mod.rs`)

```rust
/// Stats shared by any entity that participates in combat.
#[derive(Debug, Clone)]
pub struct CombatStats {
    pub hp: i32,
    pub max_hp: i32,
    pub strength: i32,
    pub dexterity: i32,
}

impl CombatStats {
    pub fn is_alive(&self) -> bool {
        self.hp > 0
    }

    pub fn take_damage(&mut self, amount: i32) {
        self.hp = (self.hp - amount).max(0);
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp = (self.hp + amount).min(self.max_hp);
    }
}
```

**Design note:** `CombatStats` is a plain struct, not a trait. The player and monsters both embed it via composition. We avoid a `HasCombatStats` trait because there's no need for polymorphic dispatch — functions that need stats simply accept `&mut CombatStats` directly.

### 5.2 Player (`entity/player.rs`)

```rust
use crate::types::Position;
use crate::entity::CombatStats;
use crate::item::{Item, weapon::WeaponKind};

/// Tracks the player's level progression (separate from combat stats).
#[derive(Debug, Clone)]
pub struct Progression {
    pub level: u32,
    pub experience: u32,
}

impl Progression {
    /// XP required to reach the next level.
    pub fn xp_to_next_level(&self) -> u32 {
        self.level * 10
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub position: Position,
    pub combat: CombatStats,
    pub progression: Progression,
    pub inventory: Vec<Item>,
    pub equipped_weapon: Option<WeaponKind>,
    pub invisible_turns: u32,
    pub charmed_turns: u32,
}

impl Player {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            combat: CombatStats {
                hp: 20,
                max_hp: 20,
                strength: 5,
                dexterity: 5,
            },
            progression: Progression {
                level: 1,
                experience: 0,
            },
            inventory: Vec::new(),
            equipped_weapon: None,
            invisible_turns: 0,
            charmed_turns: 0,
        }
    }

    pub fn glyph(&self) -> char {
        if self.combat.is_alive() { 'I' } else { 'X' }
    }

    pub fn is_invisible(&self) -> bool {
        self.invisible_turns > 0
    }

    /// Grant XP and apply level-ups. Returns the number of levels gained.
    pub fn grant_xp(&mut self, amount: u32) -> u32 {
        self.progression.experience += amount;
        let mut levels_gained = 0;

        while self.progression.level < 10
            && self.progression.experience >= self.progression.xp_to_next_level()
        {
            self.progression.experience -= self.progression.xp_to_next_level();
            self.progression.level += 1;
            self.combat.max_hp += 5;
            self.combat.hp = self.combat.max_hp;
            self.combat.strength += 1;
            self.combat.dexterity += 1;
            levels_gained += 1;
        }

        levels_gained
    }
}
```

**Rust idioms demonstrated:**
- **`Option<WeaponKind>`** — the player might not have a weapon equipped. `Option` makes this explicit and compiler-enforced. No sentinel values, no null checks that you might forget.
- **Composition over inheritance** — Rust has no class inheritance. `Player` embeds `CombatStats` and `Progression` as fields. This is clearer than OOP hierarchies and avoids the "diamond problem."
- **Builder-free construction** — for moderate structs, a `new()` method with sensible defaults is idiomatic. Use the builder pattern only when construction becomes complex.

### 5.3 Monster (`entity/monster.rs`)

```rust
use crate::types::Position;
use crate::entity::CombatStats;

/// The species of a monster. Each variant carries its own stat table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterKind {
    SnakeWoman,
    Goblin,
    Bogeyman,
    Dragon,
}

impl MonsterKind {
    /// The character used to render this monster on the map.
    pub fn glyph(self) -> char {
        match self {
            MonsterKind::SnakeWoman => 'S',
            MonsterKind::Goblin     => 'G',
            MonsterKind::Bogeyman   => 'B',
            MonsterKind::Dragon     => 'D',
        }
    }

    /// Base stats for a newly spawned monster of this kind.
    pub fn base_stats(self) -> CombatStats {
        match self {
            MonsterKind::SnakeWoman => CombatStats { hp: 8,  max_hp: 8,  strength: 3,  dexterity: 7 },
            MonsterKind::Goblin     => CombatStats { hp: 12, max_hp: 12, strength: 5,  dexterity: 4 },
            MonsterKind::Bogeyman   => CombatStats { hp: 18, max_hp: 18, strength: 7,  dexterity: 3 },
            MonsterKind::Dragon     => CombatStats { hp: 35, max_hp: 35, strength: 12, dexterity: 5 },
        }
    }

    /// How far (Manhattan distance) this monster can detect the player.
    pub fn smell_range(self) -> i32 {
        match self {
            MonsterKind::SnakeWoman => 4,
            MonsterKind::Goblin     => 5,
            MonsterKind::Bogeyman   => 6,
            MonsterKind::Dragon     => 8,
        }
    }

    /// Melee attack range (Manhattan distance).
    pub fn attack_range(self) -> i32 {
        match self {
            MonsterKind::Dragon => 2,
            _                   => 1,
        }
    }

    /// XP awarded to the player on kill.
    pub fn xp_reward(self) -> u32 {
        match self {
            MonsterKind::SnakeWoman => 5,
            MonsterKind::Goblin     => 8,
            MonsterKind::Bogeyman   => 15,
            MonsterKind::Dragon     => 30,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Monster {
    pub kind: MonsterKind,
    pub position: Position,
    pub combat: CombatStats,
    pub charmed_turns: u32,
}

impl Monster {
    pub fn new(kind: MonsterKind, position: Position) -> Self {
        Self {
            combat: kind.base_stats(),
            kind,
            position,
            charmed_turns: 0,
        }
    }

    pub fn glyph(&self) -> char {
        self.kind.glyph()
    }

    pub fn is_charmed(&self) -> bool {
        self.charmed_turns > 0
    }
}
```

**Rust idioms demonstrated:**
- **Data tables as `match` expressions** — each `MonsterKind` method returns data specific to the variant. This is Rust's idiomatic replacement for lookup tables or class hierarchies. The compiler ensures you handle every variant; adding `MonsterKind::Troll` later will cause compile errors everywhere a `match` is incomplete.
- **Wildcard pattern `_`** — in `attack_range()`, all monsters except Dragon have range 1. The `_` catches all non-Dragon variants.

---

## 6. Item System (`item/`)

### 6.1 Weapon (`item/weapon.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Sword,
    Axe,
    Hammer,
    Ward,
}

/// Weapon stats, queried from the kind.
#[derive(Debug, Clone, Copy)]
pub struct WeaponStats {
    pub attack_range: i32,
    pub dexterity_bonus: i32,
    pub damage_bonus: i32,
    pub charm_chance: u32,  // percentage, 0 for non-Ward weapons
}

impl WeaponKind {
    pub fn stats(self) -> WeaponStats {
        match self {
            WeaponKind::Sword  => WeaponStats { attack_range: 1, dexterity_bonus: 3, damage_bonus: 3, charm_chance: 0  },
            WeaponKind::Axe    => WeaponStats { attack_range: 2, dexterity_bonus: 1, damage_bonus: 4, charm_chance: 0  },
            WeaponKind::Hammer => WeaponStats { attack_range: 3, dexterity_bonus: 0, damage_bonus: 6, charm_chance: 0  },
            WeaponKind::Ward   => WeaponStats { attack_range: 4, dexterity_bonus: 2, damage_bonus: 2, charm_chance: 30 },
        }
    }

    pub fn glyph(self) -> char {
        match self {
            WeaponKind::Sword  => '(',
            WeaponKind::Axe    => '[',
            WeaponKind::Hammer => '<',
            WeaponKind::Ward   => '{',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Sword  => "Sword",
            WeaponKind::Axe    => "Axe",
            WeaponKind::Hammer => "Hammer",
            WeaponKind::Ward   => "Ward",
        }
    }
}
```

**Rust idioms demonstrated:**
- **`&'static str` return** — string literals in Rust have the `'static` lifetime, meaning they live for the entire program. Returning `&'static str` avoids allocation while being perfectly safe.
- **Associated data struct** — `WeaponStats` bundles all stats into one return value, avoiding multiple method calls.

### 6.2 Scroll (`item/scroll.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollKind {
    Health,
    Dexterity,
    Strength,
    LevelUp,
    Invisible,
}

impl ScrollKind {
    pub fn glyph(self) -> char {
        match self {
            ScrollKind::Health    => 'h',
            ScrollKind::Dexterity => 'd',
            ScrollKind::Strength  => 's',
            ScrollKind::LevelUp   => 'l',
            ScrollKind::Invisible => 'i',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ScrollKind::Health    => "Health Scroll",
            ScrollKind::Dexterity => "Dexterity Scroll",
            ScrollKind::Strength  => "Strength Scroll",
            ScrollKind::LevelUp   => "Level Up Scroll",
            ScrollKind::Invisible => "Invisible Scroll",
        }
    }
}
```

### 6.3 Item Enum (`item/mod.rs`)

```rust
pub mod weapon;
pub mod scroll;

use weapon::WeaponKind;
use scroll::ScrollKind;

/// An item that exists in the world or the player's inventory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    Weapon(WeaponKind),
    Scroll(ScrollKind),
}

impl Item {
    pub fn glyph(self) -> char {
        match self {
            Item::Weapon(w) => w.glyph(),
            Item::Scroll(s) => s.glyph(),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Item::Weapon(w) => w.name(),
            Item::Scroll(s) => s.name(),
        }
    }
}
```

**Rust idioms demonstrated:**
- **Enum with data variants** — `Item::Weapon(WeaponKind)` wraps a weapon kind inside the enum. This is Rust's version of a tagged union / sum type. It's far safer than inheritance hierarchies or `type` fields with downcasting.
- **Delegation** — `Item::glyph()` delegates to `WeaponKind::glyph()` or `ScrollKind::glyph()` via pattern matching. This is clean and explicit.

---

## 7. Items on the Map

```rust
/// An item placed at a specific position on the dungeon floor.
#[derive(Debug, Clone)]
pub struct PlacedItem {
    pub position: Position,
    pub item: Item,
}
```

The game world stores items as `Vec<PlacedItem>`. This is simple and sufficient for the expected item count (≤ 6 per level). To find an item at a position:

```rust
// Find the first item at the player's position
let found = world.items.iter().position(|placed| placed.position == player_pos);

if let Some(index) = found {
    let picked_up = world.items.swap_remove(index);
    player.inventory.push(picked_up.item);
}
```

**Rust idioms demonstrated:**
- **`iter().position()`** — returns `Option<usize>`, the index of the first matching element. Clean and functional.
- **`if let Some(index)`** — destructures `Option` without a full `match`. Idiomatic for "do something only if present."
- **`swap_remove(index)`** — O(1) removal from a `Vec` when order doesn't matter. Much faster than `remove(index)` which is O(n).

---

## 8. Game State Machine (`game.rs`)

### 8.1 Game Phase

```rust
/// The current phase of the game, modeled as an enum with data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GamePhase {
    /// Normal exploration: the player can move, pick up items, etc.
    Exploring,

    /// The player is selecting a target for an attack.
    AttackMode {
        cursor: Position,
    },

    /// The player is viewing their inventory.
    ViewingInventory,

    /// The player has died.
    GameOver,

    /// The player has reached the exit on the final level.
    Victory,
}
```

**Rust idioms demonstrated:**
- **Enum variants with data** — `AttackMode` carries a `cursor` position. This makes impossible states unrepresentable: there's no way to have a cursor position without being in attack mode, and no way to be in attack mode without a cursor position.
- **State machine pattern** — instead of boolean flags (`is_in_attack_mode`, `is_game_over`, `is_viewing_inventory`) that can conflict, a single enum enforces that the game is in exactly one phase at a time. This is one of Rust's most powerful design patterns.

### 8.2 Game Commands

```rust
/// A fully-resolved game command, independent of input method.
///
/// This is what the game engine processes — the UI layer translates
/// raw keypresses (or touch events) into these commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    // --- Exploring phase ---
    Move(Direction),
    EnterAttackMode,
    PickUp,
    OpenInventory,
    Cheat,
    Quit,

    // --- Attack mode ---
    MoveCursor(Direction),
    ConfirmAttack,
    CancelAttack,

    // --- Inventory ---
    UseItem(usize),   // 0-indexed inventory slot
    CloseInventory,
}
```

**Design note:** `Command` is intentionally **not** tied to keyboard keys. The terminal UI translates `'w'` → `Command::Move(Direction::Up)`. An Android UI would translate a swipe-up gesture to the same `Command`. This is how we decouple core logic from the platform.

### 8.3 Game World

```rust
use rand::rngs::StdRng;

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
}
```

**Why `StdRng` is a field:**
- The RNG is stored in the game world and passed explicitly to functions that need randomness.
- This enables **deterministic testing**: seed the RNG and the entire game plays out identically.
- Rust's `rand` crate uses `StdRng` for seedable, reproducible randomness.

### 8.4 The Game Loop

```rust
/// Run the game to completion.
///
/// Generic over the renderer and input source, enabling different
/// frontends (terminal, GUI, test harness) to plug in.
pub fn run_game<R, I>(world: &mut GameWorld, renderer: &mut R, input: &mut I)
where
    R: Renderer,
    I: InputSource,
{
    loop {
        renderer.render(world);

        let command = input.next_command(&world.phase);

        match command {
            Command::Quit => break,
            other => process_command(world, other),
        }

        if world.phase == GamePhase::Exploring {
            process_monster_turns(world);
            tick_status_effects(world);
            check_end_conditions(world);
        }

        if matches!(world.phase, GamePhase::GameOver | GamePhase::Victory) {
            renderer.render(world);
            break;
        }
    }
}
```

**Rust idioms demonstrated:**
- **Generics with trait bounds (`where R: Renderer`)** — the game loop works with *any* type that implements `Renderer`. No dynamic dispatch overhead (monomorphized at compile time).
- **`matches!` macro** — concise boolean check against multiple patterns.
- **Separation of concerns** — the game loop doesn't know about terminals, colors, or keyboards. It only speaks `Command`, `GameWorld`, and `Renderer`.

---

## 9. Platform Abstraction (`ui/`)

### 9.1 Trait Definitions (`ui/mod.rs`)

```rust
pub mod terminal;

use crate::game::{GameWorld, GamePhase, Command};

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
```

**Why traits here:**
This is the one place where traits are essential. We need to swap out the entire UI layer. Traits define the contract; `terminal.rs` provides one implementation; the Android port provides another.

### 9.2 Terminal Implementation (`ui/terminal.rs`)

```rust
use crossterm::{
    event::{self, Event, KeyCode},
    terminal, cursor, execute,
};
use std::io::{self, Write};

pub struct TerminalUi {
    // Terminal state, e.g., original terminal mode for cleanup
}

impl TerminalUi {
    pub fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        // Hide cursor, clear screen, etc.
        Ok(Self { })
    }
}

impl Drop for TerminalUi {
    fn drop(&mut self) {
        // Restore terminal to normal mode on exit (even on panic).
        let _ = terminal::disable_raw_mode();
    }
}

impl Renderer for TerminalUi {
    fn render(&mut self, world: &GameWorld) {
        // Clear screen, draw map, draw entities, draw HUD
        todo!()
    }

    fn render_inventory(&mut self, world: &GameWorld) {
        todo!()
    }

    fn render_end_screen(&mut self, world: &GameWorld) {
        todo!()
    }
}

impl InputSource for TerminalUi {
    fn next_command(&mut self, phase: &GamePhase) -> Command {
        // Read a key event and translate to a Command based on the current phase
        todo!()
    }
}
```

**Rust idioms demonstrated:**
- **`Drop` trait** — Rust's destructor. Guarantees terminal restoration even if the program panics. This is Rust's version of RAII (Resource Acquisition Is Initialization).
- **`io::Result<Self>`** — terminal setup can fail. We return `Result` instead of panicking, letting the caller decide how to handle the error.
- **`todo!()` macro** — placeholder that compiles but panics at runtime. Useful for stubbing out functions during development.

---

## 10. Combat System (`combat.rs`)

```rust
use rand::Rng;
use crate::entity::CombatStats;
use crate::item::weapon::WeaponStats;

/// The outcome of a single attack.
#[derive(Debug)]
pub enum AttackResult {
    Miss,
    Hit { damage: i32 },
    HitAndCharm { damage: i32 },
}

/// Resolve an attack between an attacker and a defender.
pub fn resolve_attack(
    attacker: &CombatStats,
    defender: &CombatStats,
    weapon: Option<&WeaponStats>,
    rng: &mut impl Rng,
) -> AttackResult {
    let weapon_dex = weapon.map_or(0, |w| w.dexterity_bonus);
    let weapon_dmg = weapon.map_or(0, |w| w.damage_bonus);
    let charm_chance = weapon.map_or(0, |w| w.charm_chance);

    // Hit chance
    let hit_chance = (50 + (attacker.dexterity - defender.dexterity) * 5 + weapon_dex)
        .clamp(5, 95);

    let roll: i32 = rng.gen_range(1..=100);
    if roll > hit_chance {
        return AttackResult::Miss;
    }

    // Damage
    let damage = (attacker.strength + weapon_dmg + rng.gen_range(-2..=2)).max(1);

    // Charm check
    if charm_chance > 0 && rng.gen_range(0..100) < charm_chance as i32 {
        AttackResult::HitAndCharm { damage }
    } else {
        AttackResult::Hit { damage }
    }
}
```

**Rust idioms demonstrated:**
- **`Option::map_or(default, f)`** — if the `Option` is `Some(v)`, apply `f(v)`; otherwise return `default`. Clean one-liner for "use it if present, else use a fallback."
- **`impl Rng` parameter** — accepts any type implementing `Rng`. This is "impl Trait" syntax: simpler than writing a named generic. In tests, you pass a seeded `StdRng`; in production, you pass the world's RNG.
- **`clamp(min, max)`** — standard library method that bounds a value. No need for `min(max(...))` chains.
- **Enum return type** — `AttackResult` forces the caller to handle miss, hit, and charm cases explicitly. No booleans, no magic numbers.
- **Early return** — `return AttackResult::Miss` exits immediately on a miss. Idiomatic Rust uses early returns to reduce nesting.

---

## 11. Dungeon Generation (`dungeon.rs`)

### 11.1 Algorithm Overview

```rust
use rand::Rng;

/// A rectangular room defined by its top-left corner and dimensions.
#[derive(Debug, Clone)]
struct Room {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Room {
    fn center(&self) -> Position {
        Position::new(self.x + self.width / 2, self.y + self.height / 2)
    }

    fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.width + 1
            && self.x + self.width + 1 >= other.x
            && self.y <= other.y + other.height + 1
            && self.y + self.height + 1 >= other.y
    }
}

/// Generate a complete dungeon level.
pub fn generate_level(
    width: usize,
    height: usize,
    level: u32,
    rng: &mut impl Rng,
) -> (Grid<Tile>, Vec<Room>) {
    let mut map = Grid::new(width, height, Tile::Wall);

    // 1. Generate rooms
    let rooms = generate_rooms(width, height, rng);

    // 2. Carve rooms into the map
    for room in &rooms {
        carve_room(&mut map, room);
    }

    // 3. Connect rooms with corridors
    for pair in rooms.windows(2) {
        carve_corridor(&mut map, pair[0].center(), pair[1].center(), rng);
    }

    // 4. Place stairway in the last room
    let stairway_pos = rooms.last().unwrap().center();
    if let Some(tile) = map.get_mut(stairway_pos) {
        *tile = Tile::Stairway;
    }

    (map, rooms)
}
```

**Rust idioms demonstrated:**
- **`windows(2)` iterator** — slides a 2-element window over a slice. Produces overlapping pairs `[room0, room1], [room1, room2], ...`. Perfect for connecting consecutive rooms.
- **Tuple return `(Grid<Tile>, Vec<Room>)`** — Rust functions can return multiple values via tuples. No out-parameters needed.
- **`unwrap()` on a known-safe case** — we know `rooms` is non-empty because generation guarantees at least 5 rooms. In production, you might add a debug assertion.

---

## 12. Error Handling

For this game, heavyweight error handling (`Result<T, E>` with custom error types) is only needed at the **boundary** — terminal setup, file I/O for future save/load. Inside the game engine, use:

| Situation | Approach |
| :--- | :--- |
| A function might not find a result | Return `Option<T>` |
| Terminal I/O can fail | Return `io::Result<T>` |
| Game logic invariant is violated | `debug_assert!()` or `unreachable!()` |
| Impossible state reached | `unreachable!()` macro — compiles to a panic in debug, UB hint in release |

**Do not over-engineer error handling.** A game that panics with a clear message is better than one buried in `Result` plumbing everywhere.

---

## 13. Testing Strategy

### 13.1 Unit Tests

Place tests in the same file as the code they test, inside a `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;

    #[test]
    fn test_position_distance() {
        let a = Position::new(0, 0);
        let b = Position::new(3, 4);
        assert_eq!(a.distance_to(b), 7);
    }

    #[test]
    fn test_combat_always_hits_with_max_dexterity() {
        let mut rng = StdRng::seed_from_u64(42);
        let attacker = CombatStats { hp: 10, max_hp: 10, strength: 5, dexterity: 99 };
        let defender = CombatStats { hp: 10, max_hp: 10, strength: 5, dexterity: 1 };

        for _ in 0..100 {
            let result = resolve_attack(&attacker, &defender, None, &mut rng);
            assert!(!matches!(result, AttackResult::Miss));
        }
    }

    #[test]
    fn test_player_levels_up_at_threshold() {
        let mut player = Player::new(Position::new(0, 0));
        assert_eq!(player.progression.level, 1);

        let levels = player.grant_xp(10);
        assert_eq!(levels, 1);
        assert_eq!(player.progression.level, 2);
        assert_eq!(player.combat.max_hp, 25);
    }
}
```

**Rust idioms demonstrated:**
- **`#[cfg(test)]`** — conditional compilation. The test module is only compiled when running `cargo test`, keeping the production binary lean.
- **`SeedableRng::seed_from_u64(42)`** — deterministic RNG for reproducible tests. Same seed → same random numbers → same test outcome every time.
- **`assert!`, `assert_eq!`** — standard test assertions. `assert_eq!` prints both values on failure.

### 13.2 What to Test

| Area | What to Test |
| :--- | :--- |
| `Position` | Distance calculation, step in each direction, boundary cases |
| `Grid` | In-bounds checks, get/set at corners, out-of-bounds returns `None` |
| `CombatStats` | Damage clamped to 0, healing clamped to max |
| `combat` | Hit chance boundaries (5%, 95%), damage range, charm trigger |
| `Player` | XP grant, multi-level-up, max level cap, inventory add/remove |
| `Monster` | AI movement toward player, charm skip, smell range boundary |
| `dungeon` | All rooms reachable, stairway placed, no overlapping rooms |

---

## 14. Suggested Implementation Order

Build the project incrementally. Each phase produces something you can run and test:

### Phase 1 — Data Foundation
Set up the Cargo project, define `Position`, `Direction`, `Grid<T>`, `Tile`. Write unit tests for all of them.

### Phase 2 — Entities and Items
Define `CombatStats`, `Player`, `MonsterKind`, `Monster`, `WeaponKind`, `ScrollKind`, `Item`. Write unit tests for stat calculations and XP/leveling.

### Phase 3 — Dungeon Generation
Implement room generation, corridor carving, and stairway placement. Write a temporary `main()` that prints the dungeon to stdout with `println!`.

### Phase 4 — Minimal Game Loop
Implement `Renderer` and `InputSource` traits. Build the `TerminalUi`. Wire up the game loop with player movement only (no monsters, no combat). You should be able to walk around the dungeon.

### Phase 5 — Combat
Implement `resolve_attack`, monster AI, and the attack mode targeting system. Spawn monsters and fight them.

### Phase 6 — Items and Inventory
Spawn items on the map. Implement pick-up, inventory display, scroll effects, and weapon equipping.

### Phase 7 — Polish
Add level transitions, victory/defeat screens, the message log, status effects (invisibility, charm), and the cheat command. Playtest and balance.

---

## 15. Integration Testing

### 15.1 Running the Game

```bash
# Build and run (release mode recommended for smoother rendering)
cargo run --release

# Or build first, then run the binary
cargo build --release
./target/release/minirogue      # Linux/macOS
.\target\release\minirogue.exe  # Windows
```

### 15.2 Manual Integration Test Checklist

Use this checklist to manually verify all game features work end-to-end. Launch the game with `cargo run` and walk through each scenario:

**Basic Movement & Map**
- [ ] Game starts and displays a dungeon map with `#` walls and open floor
- [ ] Player `I` is visible in the first room
- [ ] Stairway `+` is visible somewhere on the map
- [ ] Move with `W/A/S/D` — player moves on the map
- [ ] Walking into a wall does nothing (no movement, no crash)
- [ ] Walking into a monster does nothing (blocked)

**Combat**
- [ ] Press `F` to enter attack mode — `[ATTACK MODE]` indicator appears
- [ ] Move cursor with `W/A/S/D` — cursor `|` moves within weapon range
- [ ] Press `F` on a monster — damage message appears, monster HP decreases
- [ ] Press `Q` to cancel attack mode — returns to exploration
- [ ] Kill a monster — XP gained message, potential item drop on the floor
- [ ] Level up from XP — level-up message, stats increase

**Items & Inventory**
- [ ] Walk over an item glyph (`(`, `[`, `<`, `{`, `h`, `d`, `s`, `l`, `i`)
- [ ] Press `P` to pick up — "Picked up X" message, item disappears from floor
- [ ] Press `I` to open inventory — inventory list appears
- [ ] Press a number key to equip a weapon — "Equipped X" message
- [ ] Press a number key to use a scroll — effect applied (check HUD stats)
- [ ] Health Scroll: HP increases (check HUD)
- [ ] Strength Scroll: STR increases by 3
- [ ] Dexterity Scroll: DEX increases by 3
- [ ] Invisible Scroll: "Invisible(5)" status appears, decrements each turn
- [ ] Level Up Scroll: Level increases
- [ ] Press `Q` to close inventory without using anything

**Monsters & AI**
- [ ] Monsters move toward you when within smell range
- [ ] Monsters attack you when adjacent — damage messages appear
- [ ] Charmed monsters (hit with Ward) skip their turn for 3 turns
- [ ] Invisible player is not attacked by adjacent monsters
- [ ] Invisible player is still tracked (monsters move toward you)

**Level Progression**
- [ ] Walk onto `+` stairway — "You descend to level X" message
- [ ] New dungeon generated with new monsters and items
- [ ] Reach stairway on level 5 — Victory screen!

**Game Over**
- [ ] Let monsters kill you — "GAME OVER" screen with final stats
- [ ] Press any key to exit

**Cheat & Quit**
- [ ] Press `C` — stats become 999/99/99 ("Cheat mode activated!")
- [ ] Press `Q` (in exploration mode) — game exits cleanly, terminal restored

### 15.3 Automated Integration Tests with Mock UI

Create `tests/integration.rs` to run full game scenarios programmatically using a mock UI:

```rust
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
    // Should exit without panic
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
    // Move around a lot — with cheat stats, player should survive
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
```

Run integration tests:
```bash
cargo test --test integration
```

### 15.4 Seeded Deterministic Testing

The game accepts an `Option<u64>` seed. Use this for reproducible test scenarios:

```rust
// Same seed always produces the same dungeon and RNG sequence
let world1 = GameWorld::new(Some(12345));
let world2 = GameWorld::new(Some(12345));
assert_eq!(world1.player.position, world2.player.position);
assert_eq!(world1.monsters.len(), world2.monsters.len());
```

This makes integration tests fully reproducible — if a test fails, rerun with the same seed to get identical behavior.

---

## Appendix A: Key Rust Concepts Reference

| Concept | Where Used | Why |
| :--- | :--- | :--- |
| `enum` with data | `Item`, `GamePhase`, `Command`, `AttackResult` | Model states where each variant may carry different data. The compiler ensures exhaustive handling. |
| `match` | Everywhere | Exhaustive pattern matching — the compiler catches missing cases. |
| `Option<T>` | Equipped weapon, grid lookups, item search | Explicit "might not exist" — eliminates null pointer bugs. |
| `Result<T, E>` | Terminal setup, future save/load | Explicit "might fail" — forces error handling at call site. |
| `impl Trait` params | `rng: &mut impl Rng` | Accept any type implementing a trait, without naming the generic. |
| Generic functions | `run_game<R: Renderer, I: InputSource>` | Write code once, works with any frontend. Zero-cost abstraction. |
| `#[derive(...)]` | Most structs and enums | Auto-generate `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`. |
| `Drop` trait | `TerminalUi` | Cleanup resources (restore terminal) even on panic. |
| Composition | `Player` has `CombatStats` + `Progression` | Rust has no inheritance. Combine behavior through struct fields. |
| Module system | `entity/`, `item/`, `ui/` | Organize code into focused, navigable units. |
| `#[cfg(test)]` | Every module | Tests live next to the code, compiled only for `cargo test`. |
| Iterators | `items.iter().position(...)`, `rooms.windows(2)` | Expressive, zero-cost data processing without manual loops. |

---

## Appendix B: Cargo.toml

```toml
[package]
name = "minirogue"
version = "0.1.0"
edition = "2021"

[dependencies]
rand = "0.8"
crossterm = "0.28"
```

Use `cargo new minirogue` to scaffold the project, then add the dependencies with `cargo add rand crossterm`.
