use std::collections::{HashMap, VecDeque};

use arrayvec::ArrayVec;

use advent_of_code::util::coord::Direction;
use advent_of_code::util::{Indexer, LinearIndexer, VecMap, VecSet, VecTable};

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

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash, Ord, PartialOrd)]
struct BitSet(u32);

impl BitSet {
    fn new() -> Self {
        BitSet(0)
    }

    fn get(&self, index: u32) -> bool {
        debug_assert!(index < 32);
        (self.0 & (1 << index)) != 0
    }

    fn set(&mut self, index: u32) {
        debug_assert!(index < 32);
        self.0 |= 1 << index;
    }
}

struct Cache {
    cache: VecTable<NodeIndex, HashMap<u32, u32>, LinearIndexer<NodeIndex>>,
}

impl Cache {
    fn new(indexer: LinearIndexer<NodeIndex>) -> Self {
        Cache {
            cache: VecTable::new(indexer),
        }
    }

    fn get(&self, node: NodeIndex, reachable: BitSet) -> Option<u32> {
        self.cache[node].get(&reachable.0).copied()
    }

    fn insert(&mut self, node: NodeIndex, reachable: BitSet, steps: u32) {
        self.cache[node].insert(reachable.0, steps);
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

/// Optimizes trails map for part two.
///
/// Produces a graph that looks as follows:
/// ```text
///     X … X → T
///   ⁄ |   |   ↑
/// X — X … X — X
/// ⋮   ⋮   ⋮   ⋮
/// X — X … X — X
/// ↑   |   | ⁄
/// S → X … X
/// ```
/// Where `S = 32` is the start node, `T = 33` is the target node, and `X = 0..=31` are trail nodes.
/// The start node has no incoming edges, and the target node has no outgoing edges. All other edges are bidirectional.
fn optimize_trails_map(
    trails_map: TrailsMap,
    start_node: NodeIndex,
    target_node: NodeIndex,
) -> (TrailsMap, NodeIndex, NodeIndex) {
    debug_assert_eq!(trails_map.indexer().len(), 36);
    debug_assert_eq!(trails_map[start_node].len(), 1);
    debug_assert_eq!(trails_map[target_node].len(), 0);

    let mut trails_data = trails_map.to_vec();

    fn merge_node(
        trails_data: &mut [ArrayVec<(NodeIndex, CoordT), 4>],
        node: NodeIndex,
    ) -> NodeIndex {
        // Find neighbor that has an edge to this node
        let (neighbor, steps) = trails_data
            .iter()
            .enumerate()
            .find_map(|(index, neighbors)| {
                neighbors
                    .iter()
                    .find(|(neighbor, _)| *neighbor == node)
                    .map(|(_, steps)| (index as NodeIndex, *steps))
            })
            .unwrap();

        // Remove this node from data
        trails_data[node as usize].clear();

        // Remove edge from neighbor to this node
        trails_data[neighbor as usize].retain(|(neighbor, _)| *neighbor != node);

        // Update cost of edges from neighbor to other nodes
        for (_, other_steps) in &mut trails_data[neighbor as usize] {
            *other_steps += steps;
        }
        // Update cost of edges from other nodes to neighbor
        for other_neighbors in trails_data.iter_mut() {
            for (other_node, other_steps) in other_neighbors {
                if *other_node == neighbor {
                    *other_steps += steps;
                }
            }
        }

        neighbor
    }

    fn delete_incoming_edges(
        trails_data: &mut [ArrayVec<(NodeIndex, CoordT), 4>],
        node: NodeIndex,
    ) {
        // Remove edges from other nodes to this node
        for neighbors in trails_data.iter_mut() {
            neighbors.retain(|(neighbor, _)| *neighbor != node);
        }
    }

    fn delete_outgoing_edges(
        trails_data: &mut [ArrayVec<(NodeIndex, CoordT), 4>],
        node: NodeIndex,
    ) {
        // Remove edges from this node to other nodes
        trails_data[node as usize].clear();
    }

    let new_start_node = merge_node(&mut trails_data, start_node);
    delete_incoming_edges(&mut trails_data, new_start_node);

    let new_target_node = merge_node(&mut trails_data, target_node);
    delete_outgoing_edges(&mut trails_data, new_target_node);

    // Re-index the data so:
    // - Start and target nodes are at the end of the table
    // - Trail nodes have indices 0..=31
    let new_start_node_neighbors = trails_data[new_start_node as usize].clone();
    let mut new_trails_data = Vec::from_iter(trails_data.into_iter().skip(2).take(32).map(
        |mut neighbors| {
            for (neighbor, _) in &mut neighbors {
                *neighbor -= 2;
            }
            neighbors
        },
    ));

    let new_target_node = new_trails_data.len() as NodeIndex;
    new_trails_data.push(ArrayVec::new());

    let new_start_node = new_trails_data.len() as NodeIndex;
    new_trails_data.push({
        let mut neighbors = new_start_node_neighbors;
        for (neighbor, _) in &mut neighbors {
            *neighbor -= 2;
        }
        neighbors
    });

    let indexer = LinearIndexer::new(new_trails_data.len() as NodeIndex);
    (
        TrailsMap::from_vec(new_trails_data, indexer),
        new_start_node,
        new_target_node,
    )
}

/// Writes the trails map to a file for debugging with Graphviz.
fn write_trails_map_to_file(
    trails_map: &TrailsMap,
    start_node: NodeIndex,
    target_node: NodeIndex,
    filename: &str,
) {
    use std::io::Write;

    let mut file = std::fs::File::create(filename).unwrap();
    writeln!(file, "digraph {{").unwrap();
    for (from_node, trails) in trails_map.iter() {
        if from_node == start_node {
            writeln!(
                file,
                "  {} [label=\"{} (start)\", color=green]",
                from_node, from_node
            )
            .unwrap();
        } else if from_node == target_node {
            writeln!(
                file,
                "  {} [label=\"{} (target)\", color=red]",
                from_node, from_node
            )
            .unwrap();
        } else {
            writeln!(file, "  {} [label=\"{}\"]", from_node, from_node).unwrap();
        }

        for &(to_node, steps) in trails {
            writeln!(file, "  {} -> {} [label=\"{}\"]", from_node, to_node, steps).unwrap();
        }
    }
    writeln!(file, "}}").unwrap();
}

fn solve(input: &str, part_two: bool) -> Option<u32> {
    let debug = false;

    let (grid, start_coord, target_coord) = parse_input(input);

    let (trails_map, start_node, target_node) =
        gather_trails(grid, start_coord, target_coord, part_two);

    if debug {
        write_trails_map_to_file(
            &trails_map,
            start_node,
            target_node,
            if !part_two {
                "data/viz/23-1.dot"
            } else {
                "data/viz/23-2.dot"
            },
        );
    }

    let (mut trails_map, start_node, target_node) = if part_two {
        optimize_trails_map(trails_map, start_node, target_node)
    } else {
        (trails_map, start_node, target_node)
    };

    if debug && part_two {
        write_trails_map_to_file(
            &trails_map,
            start_node,
            target_node,
            "data/viz/23-2-opt.dot",
        );
    }

    for trails in trails_map.values_mut() {
        // Sort the trails by length, so DFS considers the longest trails first. (Note the list is sorted in increasing
        // order, but since the stack is LIFO, the longest trails will be considered first.)
        trails.sort_unstable_by_key(|&(_, steps)| steps);
    }

    let mut stack = Vec::new();
    let mut max_steps = 0;

    let mut cache = Cache::new(*trails_map.indexer());

    // A note on 32-bit bitsets:
    // - Part one: There are fewer than 32 nodes, so we can use the first 32 bits of the bitset as a cache key.
    // - Part two:
    //      - Node 33 is the start node and is always visited and never reachable.
    //      - Node 32 is the target node and is never visited and always reachable (otherwise we'd prune the path).
    //      - Remaining nodes 0..=31 are trail nodes, so we can use the first 32 bits of the bitset as a cache key.

    // Cannot push start node to stack here, because its index is out of bounds for the bitsets.
    for &(next_node, next_steps) in &trails_map[start_node] {
        stack.push((next_node, next_steps, BitSet::new()));
    }

    let mut inner_queue = VecDeque::with_capacity(36);

    while let Some((node, steps, mut visited)) = stack.pop() {
        if node == target_node {
            max_steps = max_steps.max(steps);
            continue;
        }

        if visited.get(node) {
            continue;
        }

        visited.set(node);

        // Compute the set of nodes reachable from this node
        let (reachable, is_target_node_reachable) = {
            let mut reachable = BitSet::new();
            let mut is_target_node_reachable = false;

            inner_queue.clear();
            inner_queue.push_back(node);

            while let Some(node) = inner_queue.pop_front() {
                reachable.set(node);

                for &(next_node, _) in &trails_map[node] {
                    if next_node == target_node {
                        is_target_node_reachable = true;
                        continue;
                    }
                    if visited.get(next_node) || reachable.get(next_node) {
                        continue;
                    }
                    inner_queue.push_back(next_node);
                }
            }

            (reachable, is_target_node_reachable)
        };

        // Prune the path if we can't reach the target node from this node
        if !is_target_node_reachable {
            continue;
        }

        // Prune the path if we've already found a path to this node that's at least as long and can still reach the
        // same set of nodes.
        match cache.get(node, reachable) {
            Some(cached_steps) if cached_steps >= steps => continue,
            _ => cache.insert(node, reachable, steps),
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
