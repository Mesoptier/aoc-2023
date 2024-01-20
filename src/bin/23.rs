use std::collections::{HashMap, VecDeque};

use arrayvec::ArrayVec;

use advent_of_code::util::coord::Direction;
use advent_of_code::util::{LinearIndexer, VecMap, VecSet, VecTable};

advent_of_code::solution!(23);

type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;
type Grid = VecTable<Coord, Tile, CoordIndexer>;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Tile {
    Path,
    Forest,
    Slope(Direction),
}

impl TryFrom<char> for Tile {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Tile::Path),
            '#' => Ok(Tile::Forest),
            '^' => Ok(Tile::Slope(Direction::Up)),
            'v' => Ok(Tile::Slope(Direction::Down)),
            '>' => Ok(Tile::Slope(Direction::Right)),
            '<' => Ok(Tile::Slope(Direction::Left)),
            _ => Err(()),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
struct BitSet(u64);

impl BitSet {
    fn new() -> Self {
        BitSet(0)
    }

    fn get(&self, index: u32) -> bool {
        debug_assert!(index < 64);
        (self.0 & (1 << index)) != 0
    }

    fn set(&mut self, index: u32) {
        debug_assert!(index < 64);
        self.0 |= 1 << index;
    }
}

fn parse_input(input: &str) -> (Grid, Coord, Coord) {
    // Parse the grid
    let mut width = None;
    let data = input
        .lines()
        .flat_map(|line| {
            if width.is_none() {
                width = Some(line.len());
            } else {
                debug_assert_eq!(width.unwrap(), line.len());
            }
            line.chars().map(Tile::try_from).map(Result::unwrap)
        })
        .collect::<Vec<_>>();
    let width = width.unwrap();
    let height = data.len() / width;
    let indexer = CoordIndexer::new(width as CoordT, height as CoordT);
    let grid = Grid::from_vec(data, indexer);

    // Find the start and target nodes
    let start_node = (0..indexer.width)
        .map(|x| Coord::new(x, 0))
        .find(|&c| grid[c] == Tile::Path)
        .unwrap();
    let target_node = (0..indexer.width)
        .map(|x| Coord::new(x, indexer.height - 1))
        .find(|&c| grid[c] == Tile::Path)
        .unwrap();

    (grid, start_node, target_node)
}

type NodeIndex = u32;
type TrailsMap = VecTable<NodeIndex, ArrayVec<(NodeIndex, CoordT), 4>, LinearIndexer<NodeIndex>>;
fn gather_trails(
    grid: Grid,
    start_coord: Coord,
    target_coord: Coord,
    part_two: bool,
) -> (TrailsMap, NodeIndex, NodeIndex) {
    let mut index_map = VecMap::<Coord, NodeIndex, CoordIndexer>::new(*grid.indexer());
    let mut trails_data = vec![];

    let mut queue = VecDeque::new();
    let mut visited = VecSet::<Coord, CoordIndexer>::new(*grid.indexer());
    queue.push_back((start_coord, Direction::Down));
    visited.insert(start_coord);

    trails_data.push(ArrayVec::new());
    index_map.insert(&start_coord, 0);

    while let Some((from_coord, direction)) = queue.pop_front() {
        let mut steps = 0;

        let from_index = *index_map.get(&from_coord).unwrap();
        let mut coord = from_coord.step_unchecked(direction);
        let mut incoming_direction = direction;
        steps += 1;

        loop {
            if coord == target_coord || coord == start_coord {
                let index = *index_map.entry(&coord).get_or_insert_with(|| {
                    let index = trails_data.len() as NodeIndex;
                    trails_data.push(ArrayVec::new());
                    index
                });
                trails_data[from_index as usize].push((index, steps));
                break;
            }

            let paths = [
                Direction::Up,
                Direction::Down,
                Direction::Right,
                Direction::Left,
            ]
            .into_iter()
            .filter(|&direction| direction != incoming_direction.opposite())
            .filter_map(|direction| {
                let coord = coord.step_unchecked(direction);
                match grid[coord] {
                    Tile::Path => Some((coord, direction)),
                    Tile::Slope(slope_direction) if slope_direction == direction || part_two => {
                        Some((coord, direction))
                    }
                    _ => None,
                }
            })
            .collect::<ArrayVec<_, 4>>();

            match paths.len() {
                0 => unreachable!("Invalid trail"),
                1 => {
                    // Continue on the same path
                    (coord, incoming_direction) = paths[0];
                    steps += 1;
                }
                _ => {
                    // Current coord is an intersection
                    let index = *index_map.entry(&coord).get_or_insert_with(|| {
                        let index = trails_data.len() as NodeIndex;
                        trails_data.push(ArrayVec::new());
                        index
                    });
                    trails_data[from_index as usize].push((index, steps));

                    if visited.insert(coord) {
                        // We haven't visited this coord before, so we need to explore it
                        for (_, direction) in paths {
                            queue.push_back((coord, direction));
                        }

                        if part_two {
                            queue.push_back((coord, incoming_direction.opposite()));
                        }
                    }

                    break;
                }
            }
        }
    }

    let start_index = *index_map.get(&start_coord).unwrap();
    let target_index = *index_map.get(&target_coord).unwrap();
    let indexer = LinearIndexer::new(trails_data.len() as NodeIndex);
    (
        TrailsMap::from_vec(trails_data, indexer),
        start_index,
        target_index,
    )
}

fn solve(input: &str, part_two: bool) -> Option<u32> {
    let (grid, start_coord, target_coord) = parse_input(input);

    let (mut trails_map, start_node, target_node) =
        gather_trails(grid, start_coord, target_coord, part_two);

    for trails in trails_map.values_mut() {
        // Sort the trails by length, so DFS considers the longest trails first. (Note the list is sorted in increasing
        // order, but since the stack is LIFO, the longest trails will be considered first.)
        trails.sort_unstable_by_key(|&(_, steps)| steps);
    }

    let mut stack = Vec::new();
    let mut max_steps = 0;

    let mut cache = HashMap::<(NodeIndex, BitSet), u32>::new();

    stack.push((start_node, 0, BitSet::new()));

    while let Some((node, steps, mut visited)) = stack.pop() {
        if node == target_node {
            max_steps = max_steps.max(steps);
            continue;
        }

        if visited.get(node) {
            continue;
        }

        visited.set(node);

        let reachable = {
            let mut reachable = BitSet::new();
            let mut queue = VecDeque::new();
            queue.push_back(node);

            while let Some(node) = queue.pop_front() {
                reachable.set(node);

                for &(next_node, _) in &trails_map[node] {
                    if visited.get(next_node) {
                        continue;
                    }
                    if reachable.get(next_node) {
                        continue;
                    }

                    queue.push_back(next_node);
                }
            }

            reachable
        };

        // Prune the path if we've already found a path to this node that's at least as long and can still reach the
        // same set of nodes.
        let cache_key = (node, reachable);
        match cache.get(&cache_key) {
            Some(&cached_steps) if cached_steps >= steps => continue,
            _ => cache.insert(cache_key, steps),
        };

        for &(next_node, next_steps) in &trails_map[node] {
            stack.push((next_node, steps + next_steps, visited));
        }
    }

    Some(max_steps)
}

pub fn part_one(input: &str) -> Option<u32> {
    solve(input, false)
}

pub fn part_two(input: &str) -> Option<u32> {
    solve(input, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(94));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(154));
    }
}
