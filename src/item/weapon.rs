#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponKind {
    Sword,
    Axe,
    Hammer,
    Ward,
}

/// Weapon stats looked up from kind.
#[derive(Debug, Clone, Copy)]
pub struct WeaponStats {
    pub attack_range: i32,
    pub dexterity_bonus: i32,
    pub damage_bonus: i32,
    /// Chance to charm target (percentage, 0–100). Only non-zero for Ward.
    pub charm_chance: u32,
}

impl WeaponKind {
    pub fn stats(self) -> WeaponStats {
        match self {
            WeaponKind::Sword => WeaponStats {
                attack_range: 1,
                dexterity_bonus: 3,
                damage_bonus: 3,
                charm_chance: 0,
            },
            WeaponKind::Axe => WeaponStats {
                attack_range: 2,
                dexterity_bonus: 1,
                damage_bonus: 4,
                charm_chance: 0,
            },
            WeaponKind::Hammer => WeaponStats {
                attack_range: 3,
                dexterity_bonus: 0,
                damage_bonus: 6,
                charm_chance: 0,
            },
            WeaponKind::Ward => WeaponStats {
                attack_range: 4,
                dexterity_bonus: 2,
                damage_bonus: 2,
                charm_chance: 30,
            },
        }
    }

    pub fn glyph(self) -> char {
        match self {
            WeaponKind::Sword => '(',
            WeaponKind::Axe => '[',
            WeaponKind::Hammer => '<',
            WeaponKind::Ward => '{',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WeaponKind::Sword => "Sword",
            WeaponKind::Axe => "Axe",
            WeaponKind::Hammer => "Hammer",
            WeaponKind::Ward => "Ward",
        }
    }

    /// All weapon kinds, useful for random selection.
    pub fn all() -> &'static [WeaponKind] {
        &[
            WeaponKind::Sword,
            WeaponKind::Axe,
            WeaponKind::Hammer,
            WeaponKind::Ward,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weapon_stats_consistency() {
        // Verify range ordering: Sword < Axe < Hammer < Ward
        assert!(WeaponKind::Sword.stats().attack_range < WeaponKind::Axe.stats().attack_range);
        assert!(WeaponKind::Axe.stats().attack_range < WeaponKind::Hammer.stats().attack_range);
        assert!(WeaponKind::Hammer.stats().attack_range < WeaponKind::Ward.stats().attack_range);
    }

    #[test]
    fn test_weapon_damage_ordering() {
        // Damage ordering: Ward < Sword < Axe < Hammer
        assert!(WeaponKind::Ward.stats().damage_bonus < WeaponKind::Sword.stats().damage_bonus);
        assert!(WeaponKind::Sword.stats().damage_bonus < WeaponKind::Axe.stats().damage_bonus);
        assert!(WeaponKind::Axe.stats().damage_bonus < WeaponKind::Hammer.stats().damage_bonus);
    }

    #[test]
    fn test_only_ward_has_charm() {
        for kind in WeaponKind::all() {
            if *kind == WeaponKind::Ward {
                assert!(kind.stats().charm_chance > 0);
            } else {
                assert_eq!(kind.stats().charm_chance, 0);
            }
        }
    }

    #[test]
    fn test_weapon_glyphs_unique() {
        let glyphs: Vec<char> = WeaponKind::all().iter().map(|w| w.glyph()).collect();
        for (i, g) in glyphs.iter().enumerate() {
            assert!(!glyphs[i + 1..].contains(g), "Duplicate glyph: {}", g);
        }
    }

    #[test]
    fn test_all_weapons_count() {
        assert_eq!(WeaponKind::all().len(), 4);
    }
}
