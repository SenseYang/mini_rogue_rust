# Game Design Specification: MiniRogue

## 1. Overview

MiniRogue is a classic dungeon adventure game, designed as a turn-based, two-dimensional roguelike (similar to a turn-based *Diablo*).

The player descends through procedurally generated dungeon levels, fighting monsters, collecting items, and growing stronger. The goal is to survive all dungeon levels and reach the exit on the final floor.

**Current Platform:** Linux/Windows command-line interface (CLI).
**Future Platform:** Android GUI.

*Architectural Note:* The system is designed to cleanly separate core game logic from the user interface. All rendering and input handling is abstracted behind interfaces, so the CLI frontend can be replaced with a graphical frontend (e.g., Android) without modifying the game engine.

---

## 2. Game Flow

### 2.1 High-Level Flow

```
New Game
  └─► Generate Dungeon Level 1
        └─► Game Loop:
              ├─ 1. Render the current map
              ├─ 2. Wait for player input
              ├─ 3. Execute player action (move / attack / pick up / use item)
              ├─ 4. Execute monster turns (all monsters act in spawn order)
              ├─ 5. Apply status effect ticks (invisibility countdown, charm countdown)
              ├─ 6. Check end conditions:
              │     ├─ Player HP ≤ 0 → Game Over (defeat)
              │     └─ Player on stairway of final level → Victory
              └─ 7. Loop back to step 1
```

### 2.2 Level Transition

When the player steps onto the `+` stairway tile on any level except the last:
1. The current level is discarded.
2. A new dungeon level is generated (level number increments by 1).
3. The player is placed at the entry point of the new level.
4. Monsters and items are spawned for the new level.

### 2.3 Win and Loss Conditions

| Condition | Trigger | Result |
| :--- | :--- | :--- |
| **Victory** | Player reaches the `+` stairway on the final dungeon level (Level 5). | Display victory screen with final stats. |
| **Defeat** | Player HP drops to 0 or below. | Player glyph changes to `X`. Display game-over screen. |

---

## 3. Denotations (Map & Entities)

### 3.1 Environments

| Glyph | Name | Description |
| :---: | :--- | :--- |
| `#` | Wall | Impassable terrain. |
| ` ` | Floor | Walkable open space. |
| `+` | Stairway | Descends to the next dungeon level. On the final level, serves as the dungeon exit. |

### 3.2 Actors

| Glyph | Name | Description |
| :---: | :--- | :--- |
| `I` | Player (alive) | The player character. |
| `X` | Player (dead) | Shown when the player's HP reaches 0. |
| `S` | Snake Woman | Fast and evasive monster. |
| `G` | Goblin | Balanced low-level monster. |
| `B` | Bogeyman | Tough melee brute. |
| `D` | Dragon | Powerful boss-tier monster. |

### 3.3 Items

**Weapons:**

| Glyph | Name |
| :---: | :--- |
| `(` | Sword |
| `{` | Ward |
| `[` | Axe |
| `<` | Hammer |

**Scrolls:**

| Glyph | Name |
| :---: | :--- |
| `h` | Health Scroll |
| `d` | Dexterity Scroll |
| `l` | Level Up Scroll |
| `i` | Invisible Scroll |
| `s` | Strength Scroll |

---

## 4. Player

### 4.1 Starting Stats

| Stat | Starting Value |
| :--- | :--- |
| HP | 20 |
| Max HP | 20 |
| Strength | 5 |
| Dexterity | 5 |
| Level | 1 |
| Experience (XP) | 0 |
| Equipped Weapon | None (bare fists: range 1, +0 dexterity, +0 damage) |
| Inventory Capacity | 10 items |

### 4.2 Leveling and Experience

* **XP to next level:** `current_level × 10` (i.e., 10 XP to reach Level 2, 20 XP to reach Level 3, etc.)
* **On level up:**
  * Max HP increases by **5**. Current HP is restored to new Max HP.
  * Strength increases by **1**.
  * Dexterity increases by **1**.
  * Remaining XP carries over to the next level threshold.
* **Maximum level:** 10.

### 4.3 Player Actions

*Note: In the CLI frontend, all inputs require pressing `ENTER` to execute. A future raw-mode terminal or GUI frontend may process keys immediately.*

| Input | Action | Description |
| :---: | :--- | :--- |
| `w` | Move Up | Move one tile north. Blocked by walls and monsters. |
| `a` | Move Left | Move one tile west. |
| `s` | Move Down | Move one tile south. |
| `d` | Move Right | Move one tile east. |
| `f` | Attack Mode | Enter attack targeting mode (see §4.4). |
| `p` | Pick Up | Pick up an item on the player's current tile. |
| `i` | Inventory | Open the inventory window (see §4.5). |
| `c` | Cheat | Set player stats to maximum values (debug/testing only). |
| `q` | Quit | Exit the game. |

Moving into a wall or off the map edge is a no-op (the turn is **not** consumed).

### 4.4 Attack Mode

1. Press `f` to enter Attack Mode. A targeting cursor `|` appears at the player's position.
2. Use `w/a/s/d` to move the cursor. The cursor can move up to the equipped weapon's **attack range** (Manhattan distance) from the player.
3. Press `f` to confirm the attack on the target tile.
   * If a monster occupies the target tile, resolve combat (see §6).
   * If the tile is empty, the attack misses (turn is consumed).
4. Press `q` to cancel Attack Mode. The player may still take another action this turn.

### 4.5 Inventory Management

1. Press `i` to open the inventory window. The inventory displays numbered items.
2. Press the **number key** (1–9, 0 for slot 10) corresponding to an item to use it:
   * **Scroll:** The scroll effect is applied immediately and the scroll is removed from inventory.
   * **Weapon:** The weapon becomes the equipped weapon. Any previously equipped weapon returns to inventory.
3. Press `q` to close the inventory window without using an item.

Opening and closing the inventory does **not** consume a turn. Using an item **does** consume a turn.

---

## 5. Monsters

### 5.1 Monster Stats

| Monster | Glyph | HP | Strength | Dexterity | Smell Range | Attack Range | XP Reward | Dungeon Levels |
| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: | :--- |
| Snake Woman | `S` | 8 | 3 | 7 | 4 | 1 | 5 | 1–3 |
| Goblin | `G` | 12 | 5 | 4 | 5 | 1 | 8 | 1–4 |
| Bogeyman | `B` | 18 | 7 | 3 | 6 | 1 | 15 | 2–5 |
| Dragon | `D` | 35 | 12 | 5 | 8 | 2 | 30 | 4–5 |

### 5.2 Monster Behavior (AI)

Each monster executes the following logic on its turn, **in order**:

1. **Charmed check:** If the monster has remaining charm turns, decrement by 1 and skip this turn entirely.
2. **Attack check:** If the player is within the monster's **attack range** (Manhattan distance) *and* the player is **not invisible**, attack the player (see §6).
3. **Smell/track:** If the player is within the monster's **smell range** (Manhattan distance), move one step toward the player along the axis with the greatest distance. Ties are broken by preferring horizontal movement.
   * Monsters **can** smell an invisible player and will move toward them, but **cannot** attack them.
4. **Idle:** If neither condition is met, the monster does not act.

### 5.3 Monster Spawning

* Monsters per level: `2 + current_level × 2` (i.e., 4 on Level 1, up to 12 on Level 5).
* Monster types are drawn randomly from those available at the current dungeon level (see "Dungeon Levels" column in §5.1).
* Monsters spawn in random floor tiles inside rooms, never on the player's starting tile or the stairway tile.

---

## 6. Combat System

### 6.1 Hit Chance

```
hit_chance = 50 + (attacker_dexterity - defender_dexterity) × 5 + weapon_dexterity_bonus
```

* Clamped to the range **[5%, 95%]**.
* A random number from 1 to 100 is rolled. If the roll ≤ `hit_chance`, the attack hits.
* Bare-fist attacks have +0 weapon dexterity bonus.

### 6.2 Damage Calculation

On a successful hit:

```
damage = max(1, attacker_strength + weapon_damage_bonus + random(-2..=2))
```

* Damage is always at least **1** on a hit.
* Bare-fist attacks have +0 weapon damage bonus.

### 6.3 Special: Charm (Ward only)

When attacking with the Ward weapon and the attack hits:
* There is a **30%** chance the target becomes **charmed** for **3 turns**.
* A charmed entity skips its entire turn (no movement or attacks).
* Charm can affect both monsters (from the player's Ward) and the player (from theoretical future charm sources).
* Charm does not stack; applying charm to an already-charmed target resets the duration to 3.

### 6.4 Monster Death

When a monster's HP drops to 0 or below:
1. The monster is removed from the map.
2. The player gains the monster's **XP reward**.
3. There is a **30%** chance the monster drops a random item on the tile where it died.
   * Dropped item type is selected uniformly at random from all weapon and scroll types.

---

## 7. Item Specifications

### 7.1 Weapons

| Weapon | Glyph | Attack Range | Dexterity Bonus | Damage Bonus | Special |
| :--- | :---: | :---: | :---: | :---: | :--- |
| Sword | `(` | 1 | +3 | +3 | None |
| Axe | `[` | 2 | +1 | +4 | None |
| Hammer | `<` | 3 | +0 | +6 | None |
| Ward | `{` | 4 | +2 | +2 | 30% charm for 3 turns |

*Range ordering: Sword (melee only) < Axe < Hammer < Ward (longest reach).*

### 7.2 Scrolls (Consumables)

| Scroll | Glyph | Effect |
| :--- | :---: | :--- |
| Health Scroll | `h` | Restores **10 HP** (cannot exceed Max HP). |
| Dexterity Scroll | `d` | Permanently increases dexterity by **3**. |
| Strength Scroll | `s` | Permanently increases strength by **3**. |
| Level Up Scroll | `l` | Instantly grants **1 level** (applies all level-up bonuses from §4.2). |
| Invisible Scroll | `i` | Grants **Invisibility** for **5 turns**. |

### 7.3 Item Spawning

* Items per level: `1 + current_level` (i.e., 2 on Level 1, up to 6 on Level 5).
* Items spawn on random floor tiles inside rooms, never on the player's starting tile or the stairway tile.
* Item types are selected uniformly at random from all weapon and scroll types.

---

## 8. Status Effects

| Effect | Duration | Applied By | Behavior |
| :--- | :--- | :--- | :--- |
| **Invisibility** | 5 turns | Invisible Scroll | Monsters cannot target the player for attacks. Monsters **can** still smell and move toward the player. Duration decrements at the end of each player turn. |
| **Charm** | 3 turns | Ward weapon (30% on hit) | Target skips its entire turn. Duration decrements at the start of the affected entity's turn. Does not stack (resets to 3). |

---

## 9. Dungeon Generation

### 9.1 Map Dimensions

* **Width:** 80 tiles
* **Height:** 24 tiles
* **Total dungeon levels:** 5

### 9.2 Room Generation

1. The map starts filled entirely with Wall (`#`) tiles.
2. Generate **5 to 8** rectangular rooms per level.
   * Room width: random between **4 and 12** tiles.
   * Room height: random between **4 and 8** tiles.
   * Rooms are placed at random positions. Rooms may **not** overlap each other (minimum 1-tile gap between rooms).
3. Each room's interior (excluding walls) is filled with Floor (` `) tiles.

### 9.3 Corridor Generation

* Connect each room to the next room in the list using **L-shaped corridors** (horizontal-first, then vertical, or vice versa chosen randomly).
* Corridors are 1 tile wide and carved out of walls, creating Floor tiles.

### 9.4 Stairway Placement

* The stairway `+` is placed on a random floor tile in the **last room** generated.

### 9.5 Player Placement

* The player starts on a random floor tile in the **first room** generated.

### 9.6 Entity Placement

* Monsters and items are placed on random floor tiles across all rooms (see §5.3 and §7.3 for counts).
* No two entities (player, monster, item, stairway) may occupy the same tile at generation time.

---

## 10. Cheat Mode

Pressing `c` activates cheat mode, applying the following changes to the player:
* HP and Max HP set to **999**.
* Strength set to **99**.
* Dexterity set to **99**.
* A message is displayed: `"Cheat mode activated!"`.

This is intended for debugging and testing only.