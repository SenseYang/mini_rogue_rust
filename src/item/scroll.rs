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
            ScrollKind::Health => 'h',
            ScrollKind::Dexterity => 'd',
            ScrollKind::Strength => 's',
            ScrollKind::LevelUp => 'l',
            ScrollKind::Invisible => 'i',
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ScrollKind::Health => "Health Scroll",
            ScrollKind::Dexterity => "Dexterity Scroll",
            ScrollKind::Strength => "Strength Scroll",
            ScrollKind::LevelUp => "Level Up Scroll",
            ScrollKind::Invisible => "Invisible Scroll",
        }
    }

    /// All scroll kinds, useful for random selection.
    pub fn all() -> &'static [ScrollKind] {
        &[
            ScrollKind::Health,
            ScrollKind::Dexterity,
            ScrollKind::Strength,
            ScrollKind::LevelUp,
            ScrollKind::Invisible,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll_glyphs_unique() {
        let glyphs: Vec<char> = ScrollKind::all().iter().map(|s| s.glyph()).collect();
        for (i, g) in glyphs.iter().enumerate() {
            assert!(!glyphs[i + 1..].contains(g), "Duplicate glyph: {}", g);
        }
    }

    #[test]
    fn test_all_scrolls_count() {
        assert_eq!(ScrollKind::all().len(), 5);
    }

    #[test]
    fn test_scroll_names_non_empty() {
        for kind in ScrollKind::all() {
            assert!(!kind.name().is_empty());
        }
    }
}
