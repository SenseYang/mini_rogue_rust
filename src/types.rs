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
            Tile::Wall => '#',
            Tile::Floor => ' ',
            Tile::Stairway => '+',
        }
    }
}

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
            Direction::Up => Self { x: self.x, y: self.y - 1 },
            Direction::Down => Self { x: self.x, y: self.y + 1 },
            Direction::Left => Self { x: self.x - 1, y: self.y },
            Direction::Right => Self { x: self.x + 1, y: self.y },
        }
    }
}

/// The four cardinal movement directions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

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

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    // --- Position tests ---

    #[test]
    fn test_position_distance() {
        let a = Position::new(0, 0);
        let b = Position::new(3, 4);
        assert_eq!(a.distance_to(b), 7);
    }

    #[test]
    fn test_position_distance_symmetric() {
        let a = Position::new(1, 2);
        let b = Position::new(4, 6);
        assert_eq!(a.distance_to(b), b.distance_to(a));
    }

    #[test]
    fn test_position_distance_to_self() {
        let p = Position::new(3, 7);
        assert_eq!(p.distance_to(p), 0);
    }

    #[test]
    fn test_position_step() {
        let p = Position::new(5, 5);
        assert_eq!(p.step(Direction::Up), Position::new(5, 4));
        assert_eq!(p.step(Direction::Down), Position::new(5, 6));
        assert_eq!(p.step(Direction::Left), Position::new(4, 5));
        assert_eq!(p.step(Direction::Right), Position::new(6, 5));
    }

    #[test]
    fn test_position_step_negative() {
        let p = Position::new(0, 0);
        assert_eq!(p.step(Direction::Up), Position::new(0, -1));
        assert_eq!(p.step(Direction::Left), Position::new(-1, 0));
    }

    // --- Tile tests ---

    #[test]
    fn test_tile_walkability() {
        assert!(!Tile::Wall.is_walkable());
        assert!(Tile::Floor.is_walkable());
        assert!(Tile::Stairway.is_walkable());
    }

    #[test]
    fn test_tile_glyphs() {
        assert_eq!(Tile::Wall.glyph(), '#');
        assert_eq!(Tile::Floor.glyph(), ' ');
        assert_eq!(Tile::Stairway.glyph(), '+');
    }

    // --- Grid tests ---

    #[test]
    fn test_grid_dimensions() {
        let grid: Grid<Tile> = Grid::new(10, 5, Tile::Wall);
        assert_eq!(grid.width(), 10);
        assert_eq!(grid.height(), 5);
    }

    #[test]
    fn test_grid_in_bounds() {
        let grid: Grid<Tile> = Grid::new(10, 5, Tile::Wall);
        assert!(grid.in_bounds(Position::new(0, 0)));
        assert!(grid.in_bounds(Position::new(9, 4)));
        assert!(!grid.in_bounds(Position::new(10, 0)));
        assert!(!grid.in_bounds(Position::new(0, 5)));
        assert!(!grid.in_bounds(Position::new(-1, 0)));
        assert!(!grid.in_bounds(Position::new(0, -1)));
    }

    #[test]
    fn test_grid_get_set() {
        let mut grid = Grid::new(10, 5, Tile::Wall);
        let pos = Position::new(3, 2);
        assert_eq!(grid.get(pos), Some(&Tile::Wall));

        if let Some(cell) = grid.get_mut(pos) {
            *cell = Tile::Floor;
        }
        assert_eq!(grid.get(pos), Some(&Tile::Floor));
    }

    #[test]
    fn test_grid_out_of_bounds_returns_none() {
        let grid = Grid::new(5, 5, Tile::Wall);
        assert_eq!(grid.get(Position::new(-1, 0)), None);
        assert_eq!(grid.get(Position::new(5, 0)), None);
        assert_eq!(grid.get(Position::new(0, -1)), None);
        assert_eq!(grid.get(Position::new(0, 5)), None);
    }

    #[test]
    fn test_grid_corners() {
        let mut grid = Grid::new(3, 3, 0u8);
        *grid.get_mut(Position::new(0, 0)).unwrap() = 1;
        *grid.get_mut(Position::new(2, 0)).unwrap() = 2;
        *grid.get_mut(Position::new(0, 2)).unwrap() = 3;
        *grid.get_mut(Position::new(2, 2)).unwrap() = 4;

        assert_eq!(grid.get(Position::new(0, 0)), Some(&1));
        assert_eq!(grid.get(Position::new(2, 0)), Some(&2));
        assert_eq!(grid.get(Position::new(0, 2)), Some(&3));
        assert_eq!(grid.get(Position::new(2, 2)), Some(&4));
        assert_eq!(grid.get(Position::new(1, 1)), Some(&0));
    }
}
