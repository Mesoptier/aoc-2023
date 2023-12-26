use nom::branch::alt;
use nom::character::complete::{char, line_ending};
use nom::combinator::value;
use nom::multi::{many1, separated_list1};
use nom::IResult;
use std::collections::HashMap;

advent_of_code::solution!(14);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Tile {
    RoundedRock,
    CubeShapedRock,
    Empty,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Direction {
    North,
    West,
    South,
    East,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::West => Direction::East,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
        }
    }
}

struct CoordIter {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    direction: Direction,
}

impl CoordIter {
    fn new(width: usize, height: usize, direction: Direction) -> Self {
        Self {
            x: match direction {
                Direction::North | Direction::South => 0,
                Direction::West => width - 1,
                Direction::East => 0,
            },
            y: match direction {
                Direction::North => height - 1,
                Direction::West | Direction::East => 0,
                Direction::South => 0,
            },
            width,
            height,
            direction,
        }
    }
}

impl Iterator for CoordIter {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self.direction {
            Direction::North => {
                if self.x < self.width - 1 {
                    self.x += 1;
                } else if self.y > 0 {
                    self.y -= 1;
                    self.x = 0;
                } else {
                    return None;
                }
            }
            Direction::West => {
                if self.y < self.height - 1 {
                    self.y += 1;
                } else if self.x > 0 {
                    self.x -= 1;
                    self.y = 0;
                } else {
                    return None;
                }
            }
            Direction::South => {
                if self.x < self.width - 1 {
                    self.x += 1;
                } else if self.y < self.height - 1 {
                    self.y += 1;
                    self.x = 0;
                } else {
                    return None;
                }
            }
            Direction::East => {
                if self.y < self.height - 1 {
                    self.y += 1;
                } else if self.x < self.width - 1 {
                    self.x += 1;
                    self.y = 0;
                } else {
                    return None;
                }
            }
        }

        Some((self.x, self.y))
    }
}

fn parse_input(input: &str) -> IResult<&str, Vec<Vec<Tile>>> {
    separated_list1(
        line_ending,
        many1(alt((
            value(Tile::RoundedRock, char('O')),
            value(Tile::CubeShapedRock, char('#')),
            value(Tile::Empty, char('.')),
        ))),
    )(input)
}

fn tilt(map: &mut Vec<Vec<Tile>>, direction: Direction) {
    let coords = CoordIter::new(map[0].len(), map.len(), direction.opposite());
    for (x, y) in coords {
        if map[y][x] == Tile::RoundedRock {
            map[y][x] = Tile::Empty;

            let mut nx = x;
            let mut ny = y;

            match direction {
                Direction::North => {
                    while ny > 0 && map[ny - 1][nx] == Tile::Empty {
                        ny -= 1;
                    }
                }
                Direction::West => {
                    while nx > 0 && map[ny][nx - 1] == Tile::Empty {
                        nx -= 1;
                    }
                }
                Direction::South => {
                    while ny < map.len() - 1 && map[ny + 1][nx] == Tile::Empty {
                        ny += 1;
                    }
                }
                Direction::East => {
                    while nx < map[ny].len() - 1 && map[ny][nx + 1] == Tile::Empty {
                        nx += 1;
                    }
                }
            }

            map[ny][nx] = Tile::RoundedRock;
        }
    }
}

fn compute_total_load(map: &Vec<Vec<Tile>>) -> u32 {
    let mut total_load = 0;
    for y in 0..map.len() {
        for x in 0..map[y].len() {
            if map[y][x] == Tile::RoundedRock {
                let load = (map.len() - y) as u32;
                total_load += load;
            }
        }
    }
    total_load
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, mut map) = parse_input(input).unwrap();
    tilt(&mut map, Direction::North);
    Some(compute_total_load(&map))
}

pub fn part_two(input: &str) -> Option<u32> {
    let (_, mut map) = parse_input(input).unwrap();

    let mut directions = [
        Direction::North,
        Direction::West,
        Direction::South,
        Direction::East,
    ]
    .into_iter()
    .cycle();

    let mut steps = 0;
    let mut cache = HashMap::<(Vec<Vec<Tile>>, Direction), usize>::new();

    let mut cycle = None;

    for direction in directions.by_ref() {
        tilt(&mut map, direction);
        steps += 1;

        if let Some(prev_steps) = cache.get(&(map.clone(), direction)) {
            cycle = Some(steps - prev_steps);
            break;
        }
        cache.insert((map.clone(), direction), steps);
    }

    let steps_remaining = (4_000_000_000 - steps) % cycle.unwrap();
    for direction in directions.take(steps_remaining) {
        tilt(&mut map, direction);
    }

    Some(compute_total_load(&map))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(136));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(64));
    }
}
