use crate::entity::CombatStats;
use crate::types::Position;

/// The species of a monster. Each variant carries its own stat table.
/// 
/// # Derives
/// - `Debug`: Enables printing and debugging monster kinds
/// - `Clone`: Allows copying monster kind values
/// - `Copy`: Enables implicit copying without consuming the value
/// - `PartialEq`: Allows comparing monster kinds for equality, useful for filtering,
///   pattern matching, and assertions (e.g., in tests like `available.contains()` or
///   `*kind != MonsterKind::Dragon`)
/// - `Eq`: Since `MonsterKind` has no floating-point or NaN values, it can implement
///   full equality semantics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterKind {
    SnakeWoman,
    Goblin,
    Bogeyman,
    Dragon,
}

impl MonsterKind {
    pub fn glyph(self) -> char {
        match self {
            MonsterKind::SnakeWoman => 'S',
            MonsterKind::Goblin => 'G',
            MonsterKind::Bogeyman => 'B',
            MonsterKind::Dragon => 'D',
        }
    }

    /// Base stats for a newly spawned monster of this kind.
    pub fn base_stats(self) -> CombatStats {
        match self {
            MonsterKind::SnakeWoman => CombatStats {
                hp: 8,
                max_hp: 8,
                strength: 3,
                dexterity: 7,
            },
            MonsterKind::Goblin => CombatStats {
                hp: 12,
                max_hp: 12,
                strength: 5,
                dexterity: 4,
            },
            MonsterKind::Bogeyman => CombatStats {
                hp: 18,
                max_hp: 18,
                strength: 7,
                dexterity: 3,
            },
            MonsterKind::Dragon => CombatStats {
                hp: 35,
                max_hp: 35,
                strength: 12,
                dexterity: 5,
            },
        }
    }

    /// How far (Manhattan distance) this monster can detect the player.
    pub fn smell_range(self) -> i32 {
        match self {
            MonsterKind::SnakeWoman => 4,
            MonsterKind::Goblin => 5,
            MonsterKind::Bogeyman => 6,
            MonsterKind::Dragon => 8,
        }
    }

    /// Melee attack range (Manhattan distance).
    pub fn attack_range(self) -> i32 {
        match self {
            MonsterKind::Dragon => 2,
            _ => 1,
        }
    }

    /// XP awarded to the player on kill.
    pub fn xp_reward(self) -> u32 {
        match self {
            MonsterKind::SnakeWoman => 5,
            MonsterKind::Goblin => 8,
            MonsterKind::Bogeyman => 15,
            MonsterKind::Dragon => 30,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            MonsterKind::SnakeWoman => "Snake Woman",
            MonsterKind::Goblin => "Goblin",
            MonsterKind::Bogeyman => "Bogeyman",
            MonsterKind::Dragon => "Dragon",
        }
    }

    /// Which dungeon levels this monster can appear on (1-indexed).
    pub fn level_range(self) -> (u32, u32) {
        match self {
            MonsterKind::SnakeWoman => (1, 3),
            MonsterKind::Goblin => (1, 4),
            MonsterKind::Bogeyman => (2, 5),
            MonsterKind::Dragon => (4, 5),
        }
    }

    /// All monster kinds, useful for filtering/selection.
    pub fn all() -> &'static [MonsterKind] {
        &[
            MonsterKind::SnakeWoman,
            MonsterKind::Goblin,
            MonsterKind::Bogeyman,
            MonsterKind::Dragon,
        ]
    }

    /// Monster kinds available at a given dungeon level.
    pub fn available_at(level: u32) -> Vec<MonsterKind> {
        MonsterKind::all()
            .iter()
            .copied()
            .filter(|k| {
                let (min, max) = k.level_range();
                level >= min && level <= max
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monster_new_has_base_stats() {
        let m = Monster::new(MonsterKind::Goblin, Position::new(0, 0));
        assert_eq!(m.combat.hp, 12);
        assert_eq!(m.combat.strength, 5);
        assert_eq!(m.combat.dexterity, 4);
    }

    #[test]
    fn test_monster_glyphs_unique() {
        let glyphs: Vec<char> = MonsterKind::all().iter().map(|m| m.glyph()).collect();
        for (i, g) in glyphs.iter().enumerate() {
            assert!(!glyphs[i + 1..].contains(g));
        }
    }

    #[test]
    fn test_monster_charm() {
        let mut m = Monster::new(MonsterKind::SnakeWoman, Position::new(0, 0));
        assert!(!m.is_charmed());
        m.charmed_turns = 3;
        assert!(m.is_charmed());
    }

    #[test]
    fn test_available_at_level_1() {
        let available = MonsterKind::available_at(1);
        assert!(available.contains(&MonsterKind::SnakeWoman));
        assert!(available.contains(&MonsterKind::Goblin));
        assert!(!available.contains(&MonsterKind::Bogeyman));
        assert!(!available.contains(&MonsterKind::Dragon));
    }

    #[test]
    fn test_available_at_level_5() {
        let available = MonsterKind::available_at(5);
        assert!(!available.contains(&MonsterKind::SnakeWoman));
        assert!(!available.contains(&MonsterKind::Goblin)); // Goblin range is 1–4
        assert!(available.contains(&MonsterKind::Bogeyman));
        assert!(available.contains(&MonsterKind::Dragon));
    }

    #[test]
    fn test_dragon_has_range_2() {
        assert_eq!(MonsterKind::Dragon.attack_range(), 2);
    }

    #[test]
    fn test_non_dragon_has_range_1() {
        for kind in MonsterKind::all() {
            if *kind != MonsterKind::Dragon {
                assert_eq!(kind.attack_range(), 1);
            }
        }
    }

    #[test]
    fn test_xp_rewards_increase_with_difficulty() {
        assert!(MonsterKind::SnakeWoman.xp_reward() < MonsterKind::Goblin.xp_reward());
        assert!(MonsterKind::Goblin.xp_reward() < MonsterKind::Bogeyman.xp_reward());
        assert!(MonsterKind::Bogeyman.xp_reward() < MonsterKind::Dragon.xp_reward());
    }
}
