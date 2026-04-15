use crate::entity::CombatStats;
use crate::item::weapon::WeaponKind;
use crate::item::Item;
use crate::types::Position;

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

pub const MAX_LEVEL: u32 = 10;
pub const INVENTORY_CAPACITY: usize = 10;

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
        if self.combat.is_alive() {
            'I'
        } else {
            'X'
        }
    }

    pub fn is_invisible(&self) -> bool {
        self.invisible_turns > 0
    }

    /// Grant XP and apply level-ups. Returns the number of levels gained.
    pub fn grant_xp(&mut self, amount: u32) -> u32 {
        self.progression.experience += amount;
        let mut levels_gained = 0;

        while self.progression.level < MAX_LEVEL
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

    /// Activate cheat mode: set stats to maximum.
    pub fn apply_cheat(&mut self) {
        self.combat.hp = 999;
        self.combat.max_hp = 999;
        self.combat.strength = 99;
        self.combat.dexterity = 99;
    }

    /// Whether the inventory has room for another item.
    pub fn can_pick_up(&self) -> bool {
        self.inventory.len() < INVENTORY_CAPACITY
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_new() {
        let p = Player::new(Position::new(5, 5));
        assert_eq!(p.combat.hp, 20);
        assert_eq!(p.combat.max_hp, 20);
        assert_eq!(p.progression.level, 1);
        assert_eq!(p.progression.experience, 0);
        assert!(p.equipped_weapon.is_none());
        assert!(p.inventory.is_empty());
    }

    #[test]
    fn test_player_glyph_alive_and_dead() {
        let mut p = Player::new(Position::new(0, 0));
        assert_eq!(p.glyph(), 'I');
        p.combat.hp = 0;
        assert_eq!(p.glyph(), 'X');
    }

    #[test]
    fn test_grant_xp_level_up() {
        let mut p = Player::new(Position::new(0, 0));
        assert_eq!(p.progression.xp_to_next_level(), 10);

        let levels = p.grant_xp(10);
        assert_eq!(levels, 1);
        assert_eq!(p.progression.level, 2);
        assert_eq!(p.combat.max_hp, 25);
        assert_eq!(p.combat.hp, 25); // healed to max
        assert_eq!(p.combat.strength, 6);
        assert_eq!(p.combat.dexterity, 6);
        assert_eq!(p.progression.experience, 0);
    }

    #[test]
    fn test_grant_xp_multi_level() {
        let mut p = Player::new(Position::new(0, 0));
        // Level 1->2 costs 10, level 2->3 costs 20, total = 30
        let levels = p.grant_xp(30);
        assert_eq!(levels, 2);
        assert_eq!(p.progression.level, 3);
    }

    #[test]
    fn test_grant_xp_carry_over() {
        let mut p = Player::new(Position::new(0, 0));
        let _ = p.grant_xp(15); // 10 to level up, 5 carries over
        assert_eq!(p.progression.level, 2);
        assert_eq!(p.progression.experience, 5);
    }

    #[test]
    fn test_grant_xp_max_level_cap() {
        let mut p = Player::new(Position::new(0, 0));
        p.progression.level = MAX_LEVEL;
        let levels = p.grant_xp(1000);
        assert_eq!(levels, 0);
        assert_eq!(p.progression.level, MAX_LEVEL);
    }

    #[test]
    fn test_cheat_mode() {
        let mut p = Player::new(Position::new(0, 0));
        p.apply_cheat();
        assert_eq!(p.combat.hp, 999);
        assert_eq!(p.combat.max_hp, 999);
        assert_eq!(p.combat.strength, 99);
        assert_eq!(p.combat.dexterity, 99);
    }

    #[test]
    fn test_inventory_capacity() {
        let mut p = Player::new(Position::new(0, 0));
        assert!(p.can_pick_up());
        for _ in 0..INVENTORY_CAPACITY {
            p.inventory.push(Item::Scroll(crate::item::scroll::ScrollKind::Health));
        }
        assert!(!p.can_pick_up());
    }

    #[test]
    fn test_invisible() {
        let mut p = Player::new(Position::new(0, 0));
        assert!(!p.is_invisible());
        p.invisible_turns = 5;
        assert!(p.is_invisible());
    }
}
