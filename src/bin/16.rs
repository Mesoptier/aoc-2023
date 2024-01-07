use itertools::chain;
use std::collections::VecDeque;

use advent_of_code::util::coord::{
    Coord, CoordIndexer, DirectedCoord, DirectedCoordIndexer, Direction,
};
use advent_of_code::util::{VecMap, VecSet, VecTable};

advent_of_code::solution!(16);

fn compute_energized_tiles(
    map: &VecTable<Coord, char, CoordIndexer>,
    initial_beam_front: DirectedCoord,
    cache: &mut VecMap<DirectedCoord, [Option<DirectedCoord>; 2], DirectedCoordIndexer>,
) -> u32 {
    let coord_indexer = *map.indexer();
    let directed_coord_indexer = DirectedCoordIndexer::from(coord_indexer);

    let mut queue = VecDeque::<DirectedCoord>::new();
    let mut visited = VecSet::new(directed_coord_indexer);
    queue.push_front(initial_beam_front);
    visited.insert(initial_beam_front);

    let mut energized_count = 0;
    let mut energized = VecSet::new(coord_indexer);
    energized.insert(initial_beam_front.coord);
    energized_count += 1;

    while let Some(node) = queue.pop_front() {
        let next_nodes = cache.entry(&node).get_or_insert_with(|| {
            let next_directions = match (map[node.coord], node.direction) {
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

            next_directions.map(|direction| {
                direction.and_then(|direction| {
                    let mut coord = coord_indexer.step(node.coord, direction)?;
                    while map[coord] == '.' {
                        if let Some(next_coord) = coord_indexer.step(coord, direction) {
                            coord = next_coord;
                        } else {
                            break;
                        }
                    }
                    Some(DirectedCoord { coord, direction })
                })
            })
        });

        for next_node in next_nodes.iter().flatten() {
            let min_x = next_node.coord.x.min(node.coord.x);
            let max_x = next_node.coord.x.max(node.coord.x);
            let min_y = next_node.coord.y.min(node.coord.y);
            let max_y = next_node.coord.y.max(node.coord.y);

            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    let coord = Coord { x, y };
                    if energized.insert(coord) {
                        energized_count += 1;
                    }
                }
            }

            if visited.insert(*next_node) {
                queue.push_back(*next_node);
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
    let indexer = CoordIndexer::new(width, height);
    VecTable::from_vec(data, indexer)
}

pub fn part_one(input: &str) -> Option<u32> {
    let map = parse_input(input);
    compute_energized_tiles(
        &map,
        DirectedCoord {
            coord: Coord { x: 0, y: 0 },
            direction: Direction::Right,
        },
        &mut VecMap::new(DirectedCoordIndexer::from(*map.indexer())),
    )
    .into()
}

pub fn part_two(input: &str) -> Option<u32> {
    let map = parse_input(input);

    let width = map.indexer().width;
    let height = map.indexer().height;

    let mut cache = VecMap::new(DirectedCoordIndexer::from(*map.indexer()));

    chain![
        (0..width).map(|x| DirectedCoord {
            coord: Coord { x, y: 0 },
            direction: Direction::Down,
        }),
        (0..width).map(|x| DirectedCoord {
            coord: Coord { x, y: height - 1 },
            direction: Direction::Up,
        }),
        (0..height).map(|y| DirectedCoord {
            coord: Coord { x: 0, y },
            direction: Direction::Right,
        }),
        (0..height).map(|y| DirectedCoord {
            coord: Coord { x: width - 1, y },
            direction: Direction::Left,
        }),
    ]
    .map(|beam_front| compute_energized_tiles(&map, beam_front, &mut cache))
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
