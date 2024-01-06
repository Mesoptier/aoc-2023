use advent_of_code::util::{Indexer, VecSet, VecTable};
use rayon::prelude::*;
use std::collections::VecDeque;

advent_of_code::solution!(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Copy, Clone)]
struct Coord {
    x: usize,
    y: usize,
}

#[derive(Copy, Clone)]
struct CoordIndexer {
    width: usize,
    height: usize,
}
impl Indexer<Coord> for CoordIndexer {
    fn len(&self) -> usize {
        self.width * self.height
    }

    fn index_for(&self, coord: &Coord) -> usize {
        coord.y * self.width + coord.x
    }
}

impl CoordIndexer {
    fn step(&self, coord: Coord, direction: Direction) -> Option<Coord> {
        let Coord { x, y } = coord;
        match direction {
            Direction::Up if y > 0 => Some(Coord { x, y: y - 1 }),
            Direction::Right if x + 1 < self.width => Some(Coord { x: x + 1, y }),
            Direction::Down if y + 1 < self.height => Some(Coord { x, y: y + 1 }),
            Direction::Left if x > 0 => Some(Coord { x: x - 1, y }),
            _ => None,
        }
    }
}

#[derive(Copy, Clone)]
struct DirectedCoord {
    coord: Coord,
    direction: Direction,
}
struct DirectedCoordIndexer {
    width: usize,
    height: usize,
}
impl Indexer<DirectedCoord> for DirectedCoordIndexer {
    fn len(&self) -> usize {
        self.width * self.height * 4
    }

    fn index_for(&self, directed_coord: &DirectedCoord) -> usize {
        let DirectedCoord {
            coord: Coord { x, y },
            direction,
        } = *directed_coord;
        let direction_index = match direction {
            Direction::Up => 0,
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Left => 3,
        };
        (y * self.width + x) * 4 + direction_index
    }
}

fn compute_energized_tiles(
    map: VecTable<Coord, char, CoordIndexer>,
    initial_beam_front: DirectedCoord,
) -> u32 {
    let mut beam_fronts = VecDeque::<DirectedCoord>::new();
    beam_fronts.push_front(initial_beam_front);

    let coord_indexer = *map.indexer();
    let directed_coord_indexer = DirectedCoordIndexer {
        width: coord_indexer.width,
        height: coord_indexer.height,
    };

    let mut energized_count = 0;
    let mut energized = VecSet::new(coord_indexer);
    let mut visited = VecSet::new(directed_coord_indexer);

    while let Some(beam) = beam_fronts.pop_front() {
        if energized.insert(beam.coord) {
            energized_count += 1;
        }
        visited.insert(beam);

        let next_directions = match (map[beam.coord], beam.direction) {
            ('/', Direction::Up) => [Some(Direction::Right), None],
            ('/', Direction::Right) => [Some(Direction::Up), None],
            ('/', Direction::Down) => [Some(Direction::Left), None],
            ('/', Direction::Left) => [Some(Direction::Down), None],
            ('\\', Direction::Up) => [Some(Direction::Left), None],
            ('\\', Direction::Left) => [Some(Direction::Up), None],
            ('\\', Direction::Down) => [Some(Direction::Right), None],
            ('\\', Direction::Right) => [Some(Direction::Down), None],
            ('|', Direction::Left) | ('|', Direction::Right) => {
                [Some(Direction::Up), Some(Direction::Down)]
            }
            ('-', Direction::Up) | ('-', Direction::Down) => {
                [Some(Direction::Left), Some(Direction::Right)]
            }
            (_, direction) => [Some(direction), None],
        };

        for direction in next_directions.into_iter().flatten() {
            if let Some(coord) = coord_indexer.step(beam.coord, direction) {
                let next_beam = DirectedCoord { coord, direction };
                if !visited.contains(&next_beam) {
                    beam_fronts.push_front(next_beam);
                }
            }
        }
    }

    energized_count
}

fn parse_input(input: &str) -> VecTable<Coord, char, CoordIndexer> {
    let mut width = None;
    let data = input
        .lines()
        .flat_map(|line| {
            if width.is_none() {
                width = Some(line.len());
            } else {
                debug_assert_eq!(width, Some(line.len()));
            }
            line.chars()
        })
        .collect::<Vec<char>>();
    let width = width.unwrap();
    let height = data.len() / width;
    let indexer = CoordIndexer { width, height };
    VecTable::from_vec(data, indexer)
}

pub fn part_one(input: &str) -> Option<u32> {
    let map = parse_input(input);
    compute_energized_tiles(
        map,
        DirectedCoord {
            coord: Coord { x: 0, y: 0 },
            direction: Direction::Right,
        },
    )
    .into()
}

pub fn part_two(input: &str) -> Option<u32> {
    let map = parse_input(input);

    let width = map.indexer().width;
    let height = map.indexer().height;

    let mut initial_beam_fronts = vec![];
    for x in 0..width {
        initial_beam_fronts.push(DirectedCoord {
            coord: Coord { x, y: 0 },
            direction: Direction::Down,
        });
        initial_beam_fronts.push(DirectedCoord {
            coord: Coord { x, y: height - 1 },
            direction: Direction::Up,
        });
    }
    for y in 0..height {
        initial_beam_fronts.push(DirectedCoord {
            coord: Coord { x: 0, y },
            direction: Direction::Right,
        });
        initial_beam_fronts.push(DirectedCoord {
            coord: Coord { x: width - 1, y },
            direction: Direction::Left,
        });
    }

    initial_beam_fronts
        .par_iter()
        .map(|&front| compute_energized_tiles(map.clone(), front))
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(46));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(51));
    }
}
