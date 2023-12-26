use itertools::Itertools;
advent_of_code::solution!(10);

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
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

fn both_parts(input: &str) -> (Option<u32>, Option<u32>) {
    let mut map = Map::from_str(input);

    let (mut x, mut y) = map.start;

    // Find valid initial direction
    let start_directions = Direction::VALUES
        .into_iter()
        .filter(|direction| {
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
        .collect_tuple::<(_, _)>()
        .unwrap();

    // Replace start tile with pipe
    match start_directions {
        (Direction::North, Direction::South) => {
            map.tiles[y][x] = Tile::VerticalPipe;
        }
        (Direction::East, Direction::West) => {
            map.tiles[y][x] = Tile::HorizontalPipe;
        }
        (Direction::North, Direction::East) => {
            map.tiles[y][x] = Tile::NorthEastPipe;
        }
        (Direction::North, Direction::West) => {
            map.tiles[y][x] = Tile::NorthWestPipe;
        }
        (Direction::South, Direction::East) => {
            map.tiles[y][x] = Tile::SouthEastPipe;
        }
        (Direction::South, Direction::West) => {
            map.tiles[y][x] = Tile::SouthWestPipe;
        }
        _ => unreachable!("Invalid start"),
    }

    let map = map;
    let mut direction = start_directions.0;

    let mut is_tile_on_loop = vec![vec![false; map.tiles[0].len()]; map.tiles.len()];
    let mut steps = 0;
    loop {
        is_tile_on_loop[y][x] = true;

        (x, y) = match direction {
            Direction::North => (x, y - 1),
            Direction::South => (x, y + 1),
            Direction::East => (x + 1, y),
            Direction::West => (x - 1, y),
        };
        steps += 1;

        if x == map.start.0 && y == map.start.1 {
            break;
        }

        match map.tiles[y][x].step_from(direction) {
            Some(new_direction) => direction = new_direction,
            None => unreachable!("Invalid loop"),
        }
    }

    let mut covered_tiles = 0;

    for (y, row) in map.tiles.iter().enumerate() {
        let mut is_within_loop = false;
        let mut x = 0;

        while x < row.len() {
            if !is_tile_on_loop[y][x] {
                if is_within_loop {
                    covered_tiles += 1;
                }
                x += 1;
                continue;
            }

            let tile = row[x];
            match tile {
                Tile::VerticalPipe => {
                    covered_tiles += 1;
                    is_within_loop = !is_within_loop;
                }
                Tile::NorthEastPipe | Tile::SouthEastPipe => {
                    covered_tiles += 1;

                    // Walk east until we hit the next corner
                    while row[x] != Tile::NorthWestPipe && row[x] != Tile::SouthWestPipe {
                        covered_tiles += 1;
                        x += 1;
                    }

                    match (tile, row[x]) {
                        (Tile::NorthEastPipe, Tile::SouthWestPipe) => {
                            is_within_loop = !is_within_loop;
                        }
                        (Tile::SouthEastPipe, Tile::NorthWestPipe) => {
                            is_within_loop = !is_within_loop;
                        }
                        _ => {}
                    }
                }
                _ => unreachable!(),
            }

            x += 1;
        }
    }

    let part_one = steps / 2;
    let part_two = covered_tiles - steps;

    (Some(part_one), Some(part_two))
}

pub fn part_one(input: &str) -> Option<u32> {
    both_parts(input).0
}

pub fn part_two(input: &str) -> Option<u32> {
    both_parts(input).1
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
        let result = part_two(&advent_of_code::template::read_file_part(
            "examples", DAY, 3,
        ));
        assert_eq!(result, Some(4));

        let result = part_two(&advent_of_code::template::read_file_part(
            "examples", DAY, 4,
        ));
        assert_eq!(result, Some(8));

        let result = part_two(&advent_of_code::template::read_file_part(
            "examples", DAY, 5,
        ));
        assert_eq!(result, Some(10));
    }
}
