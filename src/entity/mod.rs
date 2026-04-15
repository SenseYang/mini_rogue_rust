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

pub mod player;
pub mod monster;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_stats_take_damage() {
        let mut stats = CombatStats { hp: 20, max_hp: 20, strength: 5, dexterity: 5 };
        stats.take_damage(8);
        assert_eq!(stats.hp, 12);
        assert!(stats.is_alive());
    }

    #[test]
    fn test_combat_stats_damage_clamps_to_zero() {
        let mut stats = CombatStats { hp: 5, max_hp: 20, strength: 5, dexterity: 5 };
        stats.take_damage(100);
        assert_eq!(stats.hp, 0);
        assert!(!stats.is_alive());
    }

    #[test]
    fn test_combat_stats_heal() {
        let mut stats = CombatStats { hp: 10, max_hp: 20, strength: 5, dexterity: 5 };
        stats.heal(5);
        assert_eq!(stats.hp, 15);
    }

    #[test]
    fn test_combat_stats_heal_clamps_to_max() {
        let mut stats = CombatStats { hp: 18, max_hp: 20, strength: 5, dexterity: 5 };
        stats.heal(100);
        assert_eq!(stats.hp, 20);
    }
}
