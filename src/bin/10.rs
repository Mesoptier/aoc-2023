advent_of_code::solution!(10);

#[derive(Eq, PartialEq, Copy, Clone)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    const VALUES: [Self; 4] = [Self::North, Self::South, Self::East, Self::West];

    fn opposite(&self) -> Self {
        match self {
            Self::North => Self::South,
            Self::South => Self::North,
            Self::East => Self::West,
            Self::West => Self::East,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Tile {
    VerticalPipe,
    HorizontalPipe,
    NorthEastPipe,
    NorthWestPipe,
    SouthEastPipe,
    SouthWestPipe,
    Ground,
    Start,
}

impl Tile {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '|' => Some(Self::VerticalPipe),
            '-' => Some(Self::HorizontalPipe),
            'L' => Some(Self::NorthEastPipe),
            'J' => Some(Self::NorthWestPipe),
            '7' => Some(Self::SouthWestPipe),
            'F' => Some(Self::SouthEastPipe),
            '.' => Some(Self::Ground),
            'S' => Some(Self::Start),
            _ => None,
        }
    }

    /// Maps from incoming direction to outgoing direction
    fn step_from(&self, incoming_direction: Direction) -> Option<Direction> {
        match (self, incoming_direction) {
            (Self::VerticalPipe, Direction::North) => Some(Direction::North),
            (Self::VerticalPipe, Direction::South) => Some(Direction::South),
            (Self::HorizontalPipe, Direction::East) => Some(Direction::East),
            (Self::HorizontalPipe, Direction::West) => Some(Direction::West),
            (Self::NorthEastPipe, Direction::South) => Some(Direction::East),
            (Self::NorthEastPipe, Direction::West) => Some(Direction::North),
            (Self::NorthWestPipe, Direction::South) => Some(Direction::West),
            (Self::NorthWestPipe, Direction::East) => Some(Direction::North),
            (Self::SouthEastPipe, Direction::North) => Some(Direction::East),
            (Self::SouthEastPipe, Direction::West) => Some(Direction::South),
            (Self::SouthWestPipe, Direction::North) => Some(Direction::West),
            (Self::SouthWestPipe, Direction::East) => Some(Direction::South),
            (_, _) => None,
        }
    }
}

struct Map {
    tiles: Vec<Vec<Tile>>,
    start: (usize, usize),
}

impl Map {
    fn from_str(input: &str) -> Self {
        let mut tiles = Vec::new();
        let mut start = None;
        for (y, line) in input.lines().enumerate() {
            let mut row = Vec::new();
            for (x, c) in line.chars().enumerate() {
                if let Some(tile) = Tile::from_char(c) {
                    row.push(tile);
                    if tile == Tile::Start {
                        start = Some((x, y));
                    }
                }
            }
            tiles.push(row);
        }
        Self {
            tiles,
            start: start.unwrap(),
        }
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let map = Map::from_str(input);

    let (mut x, mut y) = map.start;

    // Find valid initial direction
    let mut direction = Direction::VALUES
        .into_iter()
        .find(|direction| {
            let (nx, ny) = match direction {
                Direction::North => {
                    if y == 0 {
                        return false;
                    }
                    (x, y - 1)
                }
                Direction::South => {
                    if y == map.tiles.len() - 1 {
                        return false;
                    }
                    (x, y + 1)
                }
                Direction::East => {
                    if x == map.tiles[y].len() - 1 {
                        return false;
                    }
                    (x + 1, y)
                }
                Direction::West => {
                    if x == 0 {
                        return false;
                    }
                    (x - 1, y)
                }
            };

            map.tiles[ny][nx].step_from(*direction).is_some()
        })
        .unwrap();

    let mut steps = 0;
    loop {
        (x, y) = match direction {
            Direction::North => (x, y - 1),
            Direction::South => (x, y + 1),
            Direction::East => (x + 1, y),
            Direction::West => (x - 1, y),
        };
        steps += 1;

        if x == map.start.0 && y == map.start.1 {
            return Some(steps / 2);
        }

        match map.tiles[y][x].step_from(direction) {
            Some(new_direction) => direction = new_direction,
            None => unreachable!("Invalid loop"),
        }
    }
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file_part(
            "examples", DAY, 1,
        ));
        assert_eq!(result, Some(4));

        let result = part_one(&advent_of_code::template::read_file_part(
            "examples", DAY, 2,
        ));
        assert_eq!(result, Some(8));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
