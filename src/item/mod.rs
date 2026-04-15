pub mod weapon;
pub mod scroll;

use crate::types::Position;
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

/// An item placed at a specific position on the dungeon floor.
#[derive(Debug, Clone)]
pub struct PlacedItem {
    pub position: Position,
    pub item: Item,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_weapon_delegates_glyph() {
        let item = Item::Weapon(WeaponKind::Sword);
        assert_eq!(item.glyph(), '(');
        assert_eq!(item.name(), "Sword");
    }

    #[test]
    fn test_item_scroll_delegates_glyph() {
        let item = Item::Scroll(ScrollKind::Health);
        assert_eq!(item.glyph(), 'h');
        assert_eq!(item.name(), "Health Scroll");
    }
}
