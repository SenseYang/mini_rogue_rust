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
///
/// `weapon` is `None` for bare-fist attacks (and for monster attacks).
pub fn resolve_attack(
    attacker: &CombatStats,
    defender: &CombatStats,
    weapon: Option<&WeaponStats>,
    rng: &mut impl Rng,
) -> AttackResult {
    let weapon_dex = weapon.map_or(0, |w| w.dexterity_bonus);
    let weapon_dmg = weapon.map_or(0, |w| w.damage_bonus);
    let charm_chance = weapon.map_or(0, |w| w.charm_chance);

    // Hit chance: 50 + (atk_dex - def_dex) * 5 + weapon_dex, clamped [5, 95]
    let hit_chance =
        (50 + (attacker.dexterity - defender.dexterity) * 5 + weapon_dex).clamp(5, 95);

    let roll: i32 = rng.gen_range(1..=100);
    if roll > hit_chance {
        return AttackResult::Miss;
    }

    // Damage: max(1, strength + weapon_dmg + rand(-2..=2))
    let damage = (attacker.strength + weapon_dmg + rng.gen_range(-2..=2)).max(1);

    // Charm check (Ward only)
    if charm_chance > 0 && rng.gen_range(0..100) < charm_chance as i32 {
        AttackResult::HitAndCharm { damage }
    } else {
        AttackResult::Hit { damage }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    fn make_stats(strength: i32, dexterity: i32) -> CombatStats {
        CombatStats {
            hp: 100,
            max_hp: 100,
            strength,
            dexterity,
        }
    }

    #[test]
    fn test_high_dex_attacker_rarely_misses() {
        let mut rng = StdRng::seed_from_u64(42);
        let attacker = make_stats(5, 50);
        let defender = make_stats(5, 1);

        let mut misses = 0;
        for _ in 0..1000 {
            if matches!(resolve_attack(&attacker, &defender, None, &mut rng), AttackResult::Miss) {
                misses += 1;
            }
        }
        // With 95% hit rate, expect ~50 misses out of 1000
        assert!(misses < 100, "Too many misses: {}", misses);
    }

    #[test]
    fn test_low_dex_attacker_misses_often() {
        let mut rng = StdRng::seed_from_u64(42);
        let attacker = make_stats(5, 1);
        let defender = make_stats(5, 50);

        let mut misses = 0;
        for _ in 0..1000 {
            if matches!(resolve_attack(&attacker, &defender, None, &mut rng), AttackResult::Miss) {
                misses += 1;
            }
        }
        // With 5% hit rate, expect ~950 misses
        assert!(misses > 900, "Too few misses: {}", misses);
    }

    #[test]
    fn test_damage_is_at_least_one() {
        let mut rng = StdRng::seed_from_u64(123);
        let attacker = make_stats(0, 50); // 0 strength
        let defender = make_stats(5, 1);

        for _ in 0..100 {
            match resolve_attack(&attacker, &defender, None, &mut rng) {
                AttackResult::Hit { damage } | AttackResult::HitAndCharm { damage } => {
                    assert!(damage >= 1, "Damage below 1: {}", damage);
                }
                AttackResult::Miss => {}
            }
        }
    }

    #[test]
    fn test_weapon_bonus_affects_damage() {
        let mut rng = StdRng::seed_from_u64(42);
        let attacker = make_stats(5, 50);
        let defender = make_stats(5, 1);
        let weapon = WeaponStats {
            attack_range: 1,
            dexterity_bonus: 0,
            damage_bonus: 10,
            charm_chance: 0,
        };

        let mut total_dmg_with = 0;
        let mut total_dmg_without = 0;
        let mut rng2 = StdRng::seed_from_u64(42);

        for _ in 0..100 {
            if let AttackResult::Hit { damage } =
                resolve_attack(&attacker, &defender, Some(&weapon), &mut rng)
            {
                total_dmg_with += damage;
            }
            if let AttackResult::Hit { damage } =
                resolve_attack(&attacker, &defender, None, &mut rng2)
            {
                total_dmg_without += damage;
            }
        }
        assert!(total_dmg_with > total_dmg_without);
    }

    #[test]
    fn test_charm_only_with_ward() {
        let mut rng = StdRng::seed_from_u64(42);
        let attacker = make_stats(5, 50);
        let defender = make_stats(5, 1);

        // No weapon — should never charm
        for _ in 0..200 {
            assert!(
                !matches!(
                    resolve_attack(&attacker, &defender, None, &mut rng),
                    AttackResult::HitAndCharm { .. }
                ),
                "Charm without weapon!"
            );
        }

        // Ward weapon — should sometimes charm
        let ward = WeaponStats {
            attack_range: 4,
            dexterity_bonus: 2,
            damage_bonus: 2,
            charm_chance: 30,
        };
        let mut charms = 0;
        for _ in 0..1000 {
            if matches!(
                resolve_attack(&attacker, &defender, Some(&ward), &mut rng),
                AttackResult::HitAndCharm { .. }
            ) {
                charms += 1;
            }
        }
        assert!(charms > 0, "Ward never charmed in 1000 attacks");
        assert!(charms < 500, "Ward charmed too often: {}", charms);
    }

    #[test]
    fn test_deterministic_with_seed() {
        let attacker = make_stats(5, 5);
        let defender = make_stats(5, 5);

        let run = |seed: u64| -> Vec<bool> {
            let mut rng = StdRng::seed_from_u64(seed);
            (0..50)
                .map(|_| {
                    matches!(
                        resolve_attack(&attacker, &defender, None, &mut rng),
                        AttackResult::Hit { .. }
                    )
                })
                .collect()
        };

        assert_eq!(run(99), run(99));
    }
}
