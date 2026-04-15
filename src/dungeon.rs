use rand::Rng;

use crate::types::{Grid, Position, Tile};

/// A rectangular room defined by its top-left corner and dimensions.
#[derive(Debug, Clone)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Room {
    /// Center position of the room.
    pub fn center(&self) -> Position {
        Position::new(self.x + self.width / 2, self.y + self.height / 2)
    }

    /// Whether this room overlaps another (with a 1-tile gap).
    pub fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.width
            && self.x + self.width >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }

    /// Iterate over all interior floor positions of this room.
    pub fn floor_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        for dy in 1..self.height - 1 {
            for dx in 1..self.width - 1 {
                positions.push(Position::new(self.x + dx, self.y + dy));
            }
        }
        positions
    }
}

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 20;
pub const TOTAL_LEVELS: u32 = 5;

/// Generate a complete dungeon level. Returns the map and the list of rooms.
pub fn generate_level(
    width: usize,
    height: usize,
    rng: &mut impl Rng,
) -> (Grid<Tile>, Vec<Room>) {
    let mut map = Grid::new(width, height, Tile::Wall);

    let rooms = generate_rooms(width, height, rng);

    // Carve rooms
    for room in &rooms {
        carve_room(&mut map, room);
    }

    // Connect rooms with L-shaped corridors
    for pair in rooms.windows(2) {
        carve_corridor(&mut map, pair[0].center(), pair[1].center(), rng);
    }

    // Place stairway in the last room
    if let Some(last_room) = rooms.last() {
        let stairway_pos = last_room.center();
        if let Some(tile) = map.get_mut(stairway_pos) {
            *tile = Tile::Stairway;
        }
    }

    (map, rooms)
}

/// Try to generate between 5 and 8 non-overlapping rooms.
fn generate_rooms(width: usize, height: usize, rng: &mut impl Rng) -> Vec<Room> {
    let target = rng.gen_range(5..=8);
    let mut rooms = Vec::new();
    let mut attempts = 0;
    let max_attempts = 200;

    while rooms.len() < target && attempts < max_attempts {
        let w = rng.gen_range(4..=12);
        let h = rng.gen_range(4..=8);
        // Ensure room fits within map borders (leave 1-tile border)
        let max_x = (width as i32) - w - 1;
        let max_y = (height as i32) - h - 1;
        if max_x < 1 || max_y < 1 {
            attempts += 1;
            continue;
        }
        let x = rng.gen_range(1..=max_x);
        let y = rng.gen_range(1..=max_y);

        let room = Room {
            x,
            y,
            width: w,
            height: h,
        };

        if !rooms.iter().any(|r: &Room| room.intersects(r)) {
            rooms.push(room);
        }
        attempts += 1;
    }

    // Ensure at least 2 rooms for player and stairway
    if rooms.len() < 2 {
        rooms.clear();
        rooms.push(Room {
            x: 1,
            y: 1,
            width: 8,
            height: 6,
        });
        rooms.push(Room {
            x: (width as i32) - 10,
            y: (height as i32) - 8,
            width: 8,
            height: 6,
        });
    }

    rooms
}

/// Carve the interior of a room into the map as floor tiles.
fn carve_room(map: &mut Grid<Tile>, room: &Room) {
    for y in room.y..room.y + room.height {
        for x in room.x..room.x + room.width {
            let pos = Position::new(x, y);
            if let Some(tile) = map.get_mut(pos) {
                *tile = Tile::Floor;
            }
        }
    }
}

/// Carve an L-shaped corridor between two points.
fn carve_corridor(map: &mut Grid<Tile>, from: Position, to: Position, rng: &mut impl Rng) {
    let horizontal_first = rng.gen_bool(0.5);

    if horizontal_first {
        carve_horizontal(map, from.x, to.x, from.y);
        carve_vertical(map, from.y, to.y, to.x);
    } else {
        carve_vertical(map, from.y, to.y, from.x);
        carve_horizontal(map, from.x, to.x, to.y);
    }
}

fn carve_horizontal(map: &mut Grid<Tile>, x1: i32, x2: i32, y: i32) {
    let (start, end) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    for x in start..=end {
        let pos = Position::new(x, y);
        if let Some(tile) = map.get_mut(pos) {
            if *tile == Tile::Wall {
                *tile = Tile::Floor;
            }
        }
    }
}

fn carve_vertical(map: &mut Grid<Tile>, y1: i32, y2: i32, x: i32) {
    let (start, end) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    for y in start..=end {
        let pos = Position::new(x, y);
        if let Some(tile) = map.get_mut(pos) {
            if *tile == Tile::Wall {
                *tile = Tile::Floor;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use std::collections::HashSet;

    #[test]
    fn test_room_center() {
        let room = Room {
            x: 10,
            y: 10,
            width: 6,
            height: 4,
        };
        assert_eq!(room.center(), Position::new(13, 12));
    }

    #[test]
    fn test_room_intersects() {
        let r1 = Room {
            x: 5,
            y: 5,
            width: 5,
            height: 5,
        };
        let r2 = Room {
            x: 8,
            y: 8,
            width: 5,
            height: 5,
        };
        assert!(r1.intersects(&r2));
    }

    #[test]
    fn test_room_no_intersect() {
        let r1 = Room {
            x: 1,
            y: 1,
            width: 4,
            height: 4,
        };
        let r2 = Room {
            x: 20,
            y: 20,
            width: 4,
            height: 4,
        };
        assert!(!r1.intersects(&r2));
    }

    #[test]
    fn test_generate_level_has_rooms() {
        let mut rng = StdRng::seed_from_u64(42);
        let (map, rooms) = generate_level(MAP_WIDTH, MAP_HEIGHT, &mut rng);

        assert!(rooms.len() >= 2, "Too few rooms: {}", rooms.len());
        assert!(rooms.len() <= 8, "Too many rooms: {}", rooms.len());

        // First room center should be floor
        let first_center = rooms[0].center();
        assert_eq!(map.get(first_center), Some(&Tile::Floor));
    }

    #[test]
    fn test_generate_level_has_stairway() {
        let mut rng = StdRng::seed_from_u64(42);
        let (map, rooms) = generate_level(MAP_WIDTH, MAP_HEIGHT, &mut rng);

        let last_center = rooms.last().unwrap().center();
        assert_eq!(map.get(last_center), Some(&Tile::Stairway));
    }

    #[test]
    fn test_generate_level_rooms_connected() {
        let mut rng = StdRng::seed_from_u64(42);
        let (map, rooms) = generate_level(MAP_WIDTH, MAP_HEIGHT, &mut rng);

        // BFS from first room center should reach last room center
        let start = rooms[0].center();
        let goal = rooms.last().unwrap().center();

        let mut visited = HashSet::new();
        let mut queue = vec![start];
        visited.insert(start);

        while let Some(pos) = queue.pop() {
            if pos == goal {
                return; // success
            }
            for dir in &[
                crate::types::Direction::Up,
                crate::types::Direction::Down,
                crate::types::Direction::Left,
                crate::types::Direction::Right,
            ] {
                let next = pos.step(*dir);
                if !visited.contains(&next) {
                    if let Some(tile) = map.get(next) {
                        if tile.is_walkable() {
                            visited.insert(next);
                            queue.push(next);
                        }
                    }
                }
            }
        }

        panic!("First room is not connected to last room!");
    }

    #[test]
    fn test_generate_level_deterministic() {
        let make_map = |seed: u64| {
            let mut rng = StdRng::seed_from_u64(seed);
            let (map, _) = generate_level(MAP_WIDTH, MAP_HEIGHT, &mut rng);
            // Check a few positions
            (0..10)
                .map(|i| *map.get(Position::new(i, 0)).unwrap())
                .collect::<Vec<_>>()
        };
        assert_eq!(make_map(99), make_map(99));
    }

    #[test]
    fn test_room_floor_positions() {
        let room = Room {
            x: 5,
            y: 5,
            width: 4,
            height: 4,
        };
        let positions = room.floor_positions();
        // Interior of 4x4 room is 2x2 = 4 floor tiles
        assert_eq!(positions.len(), 4);
        assert!(positions.contains(&Position::new(6, 6)));
        assert!(positions.contains(&Position::new(7, 7)));
    }
}
