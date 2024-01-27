#![feature(portable_simd)]

use std::collections::{HashMap, VecDeque};
use std::simd::prelude::*;

use arrayvec::ArrayVec;

use advent_of_code::util::coord::Direction;
use advent_of_code::util::{Indexer, LinearIndexer, VecMap, VecSet, VecTable};

use crate::tile_grid::Tile;

advent_of_code::solution!(23);

type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;
type Grid = VecTable<Coord, Tile, CoordIndexer>;

mod tile_grid {
    use advent_of_code::util::coord::Direction;
    use advent_of_code::util::CharGrid;

    use crate::{Coord, CoordT};

    #[derive(Copy, Clone, Eq, PartialEq, Debug)]
    pub enum Tile {
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

    pub struct TileGrid<'a> {
        char_grid: CharGrid<'a>,
    }

    impl<'a> TileGrid<'a> {
        pub fn new(input: &'a str) -> Self {
            TileGrid {
                char_grid: CharGrid::new(input),
            }
        }

        pub fn width(&self) -> CoordT {
            self.char_grid.width() as CoordT
        }

        pub fn height(&self) -> CoordT {
            self.char_grid.height() as CoordT
        }

        pub fn get(&self, coord: Coord) -> Option<Tile> {
            self.char_grid
                .get(coord.x as usize, coord.y as usize)
                .and_then(|c| Tile::try_from(c).ok())
        }
    }
}

mod graph {
    use std::collections::{HashMap, VecDeque};

    use arrayvec::ArrayVec;
    use petgraph::graph::{DiGraph, NodeIndex};
    use petgraph::graphmap::DiGraphMap;
    use petgraph::visit::IntoNodeReferences;

    use advent_of_code::util::coord::Direction;

    use crate::tile_grid::{Tile, TileGrid};
    use crate::Coord;

    pub type Cost = u32;

    pub fn build_graph(
        tile_grid: TileGrid,
        start_coord: Coord,
        target_coord: Coord,
        part_two: bool,
    ) -> DiGraphMap<Coord, Cost> {
        let mut graph = DiGraphMap::<Coord, Cost>::new();
        let mut queue = VecDeque::new();

        queue.push_back((start_coord, Direction::Down));

        while let Some((from_coord, direction)) = queue.pop_front() {
            let mut coord = from_coord.step_unchecked(direction);
            let mut prev_direction = direction;
            let mut cost = 1;

            // Follow path until we reach a node (intersection or target coord)
            let to_coord = loop {
                if coord == target_coord {
                    break coord;
                }

                let neighbors = [
                    Direction::Up,
                    Direction::Down,
                    Direction::Right,
                    Direction::Left,
                ]
                .into_iter()
                .filter(|direction| {
                    // Don't backtrack
                    *direction != prev_direction.opposite()
                })
                .filter_map(|direction| {
                    let next_coord = coord.step_unchecked(direction);
                    match tile_grid.get(next_coord) {
                        Some(Tile::Path) => Some((next_coord, direction)),
                        Some(Tile::Slope(slope_direction))
                            if slope_direction == direction || part_two =>
                        {
                            Some((next_coord, direction))
                        }
                        _ => None,
                    }
                })
                .collect::<ArrayVec<_, 4>>();

                match neighbors.len() {
                    0 => unreachable!("Invalid trail"),
                    1 => {
                        // Continue along the same path
                        (coord, prev_direction) = neighbors[0];
                        cost += 1;
                    }
                    _ => {
                        // Found an intersection
                        if !graph.contains_node(coord) {
                            // We haven't visited this intersection before, so we need to explore it
                            for (_, direction) in neighbors {
                                queue.push_back((coord, direction));
                            }
                        }

                        break coord;
                    }
                }
            };

            // Add edge from previous node to this node
            graph.add_edge(from_coord, to_coord, cost);

            if part_two {
                // Add edge from this node to previous node
                graph.add_edge(to_coord, from_coord, cost);
            }
        }

        graph
    }

    fn merge_into_single_neighbor(graph: &mut DiGraphMap<Coord, Cost>, node: Coord) -> Coord {
        let neighbors = graph.neighbors(node).collect::<Vec<_>>();
        assert_eq!(neighbors.len(), 1);

        let neighbor = neighbors[0];

        // Remove edges between node and neighbor
        graph.remove_edge(node, neighbor).unwrap();
        let removed_cost = graph.remove_edge(neighbor, node).unwrap();

        // Add removed cost to that of neighbor's other edges
        let neighbor_neighbors = graph.neighbors(neighbor).collect::<Vec<_>>();
        for neighbor_neighbor in neighbor_neighbors {
            graph
                .edge_weight_mut(neighbor, neighbor_neighbor)
                .map(|cost| *cost += removed_cost)
                .unwrap();
            graph
                .edge_weight_mut(neighbor_neighbor, neighbor)
                .map(|cost| *cost += removed_cost)
                .unwrap();
        }

        // Remove node
        graph.remove_node(node);

        neighbor
    }

    fn remove_incoming_edges(graph: &mut DiGraphMap<Coord, Cost>, node: Coord) {
        let neighbors = graph
            .neighbors_directed(node, petgraph::Direction::Incoming)
            .collect::<Vec<_>>();
        for neighbor in neighbors {
            graph.remove_edge(neighbor, node).unwrap();
        }
    }

    fn remove_outgoing_edges(graph: &mut DiGraphMap<Coord, Cost>, node: Coord) {
        let neighbors = graph
            .neighbors_directed(node, petgraph::Direction::Outgoing)
            .collect::<Vec<_>>();
        for neighbor in neighbors {
            graph.remove_edge(node, neighbor).unwrap();
        }
    }

    /// Optimizes the graph such that:
    /// - Start node has no incoming edges
    /// - Target node has no outgoing edges
    /// - There are `N <= 34` nodes, where nodes with indices:
    ///     - `0..=N-3` are trail nodes,
    ///     - `N-2` is the start node,
    ///     - `N-1` is the target node.
    pub fn optimize_graph(
        mut graph: DiGraphMap<Coord, Cost>,
        start_coord: Coord,
        target_coord: Coord,
    ) -> (DiGraph<(), Cost>, NodeIndex, NodeIndex) {
        let start_coord = merge_into_single_neighbor(&mut graph, start_coord);
        remove_incoming_edges(&mut graph, start_coord);

        let target_coord = merge_into_single_neighbor(&mut graph, target_coord);
        remove_outgoing_edges(&mut graph, target_coord);

        let node_index_map = {
            // Sort nodes so start and target nodes are at the end
            let mut sorted_nodes = graph.nodes().collect::<Vec<_>>();
            sorted_nodes.sort_unstable_by_key(|node| match *node {
                node if node == start_coord => 1,
                node if node == target_coord => 2,
                _ => 0,
            });

            HashMap::<Coord, u32>::from_iter(
                sorted_nodes
                    .into_iter()
                    .enumerate()
                    .map(|(index, node)| (node, index as u32)),
            )
        };

        let graph = DiGraph::from_edges(
            graph
                .all_edges()
                .map(|(souce, target, cost)| {
                    (node_index_map[&souce], node_index_map[&target], cost)
                })
                .collect::<Vec<_>>(),
        );

        let start_node: NodeIndex = node_index_map[&start_coord].into();
        let target_node: NodeIndex = node_index_map[&target_coord].into();

        assert!(graph.node_count() <= 34);
        assert_eq!(start_node.index(), graph.node_count() - 2);
        assert_eq!(target_node.index(), graph.node_count() - 1);

        (graph, start_node, target_node)
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

struct ComputeReachable {
    neighbor_masks: u32x32,
    before_target_mask: u32,
}

impl ComputeReachable {
    fn new(trails_map: &TrailsMap, target_node: NodeIndex) -> Self {
        let mut neighbor_masks = [0; 32];
        trails_map
            .iter()
            .take(32)
            .map(|(_, neighbors)| {
                neighbors
                    .iter()
                    .filter(|(neighbor, _)| *neighbor < 32)
                    .map(|(neighbor, _)| 1 << *neighbor)
                    .fold(0, |a, b| a | b)
            })
            .enumerate()
            .for_each(|(i, mask)| neighbor_masks[i] = mask);
        let neighbor_masks = u32x32::from_array(neighbor_masks);

        let before_target_mask = trails_map
            .iter()
            .filter(|(_, neighbors)| {
                neighbors
                    .iter()
                    .any(|(neighbor, _)| *neighbor == target_node)
            })
            .map(|(node, _)| 1 << node)
            .fold(0, |a, b| a | b);

        ComputeReachable {
            neighbor_masks,
            before_target_mask,
        }
    }

    fn compute_reachable(&self, node: NodeIndex, visited: &BitSet) -> (BitSet, bool) {
        let not_visited = !visited.0;
        let mut reachable = 1 << node;

        loop {
            let next_reachable = reachable
                | mask32x32::from_bitmask(reachable as u64)
                    .select(self.neighbor_masks, u32x32::splat(0))
                    .reduce_or();

            // Can't re-visit nodes that have already been visited
            let next_reachable = next_reachable & not_visited;

            if next_reachable == reachable {
                break;
            }

            reachable = next_reachable;
        }

        (BitSet(reachable), reachable & self.before_target_mask != 0)
    }
}

fn solve(input: &str, part_two: bool, example: bool) -> Option<u32> {
    let debug = false;

    let (grid, start_coord, target_coord) = parse_input(input);

    let (trails_map, start_node, target_node) =
        gather_trails(grid, start_coord, target_coord, part_two);

    if debug && !example {
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

    let (mut trails_map, start_node, target_node) = if part_two && !example {
        optimize_trails_map(trails_map, start_node, target_node)
    } else {
        (trails_map, start_node, target_node)
    };

    if debug && part_two && !example {
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
    let compute_reachable = ComputeReachable::new(&trails_map, target_node);

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

    while let Some((node, steps, mut visited)) = stack.pop() {
        if node == target_node {
            max_steps = max_steps.max(steps);
            continue;
        }

        if visited.get(node) {
            continue;
        }

        // Compute the set of nodes reachable from this node
        let (reachable, is_target_node_reachable) =
            compute_reachable.compute_reachable(node, &visited);

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

        visited.set(node);

        for &(next_node, next_steps) in &trails_map[node] {
            stack.push((next_node, steps + next_steps, visited));
        }
    }

    Some(max_steps)
}

pub fn part_one(input: &str) -> Option<u32> {
    solve(input, false, false)
}

pub fn part_two(input: &str) -> Option<u32> {
    solve(input, true, false)
}

#[cfg(test)]
mod tests {
    use petgraph::dot::Dot;

    use super::*;

    #[test]
    fn debug() {
        let input = advent_of_code::template::read_file("inputs", DAY);

        let tile_grid = tile_grid::TileGrid::new(&input);

        // Start coord is the only path tile in the top row
        let start_coord = (0..tile_grid.width())
            .map(|x| Coord::new(x, 0))
            .find(|&coord| tile_grid.get(coord) == Some(Tile::Path))
            .unwrap();

        // Target coord is the only path tile in the bottom row
        let target_coord = (0..tile_grid.width())
            .map(|x| Coord::new(x, tile_grid.height() - 1))
            .find(|&coord| tile_grid.get(coord) == Some(Tile::Path))
            .unwrap();

        let part_two = true;
        let graph = graph::build_graph(tile_grid, start_coord, target_coord, part_two);
        let (graph, start_node, target_node) =
            graph::optimize_graph(graph, start_coord, target_coord);

        println!(
            "{}",
            Dot::new(&graph.map(
                |node, _| match node {
                    node if node == start_node => "Start".to_string(),
                    node if node == target_node => "Target".to_string(),
                    _ => node.index().to_string(),
                },
                |_, cost| *cost,
            ))
        );
    }

    #[test]
    fn test_part_one() {
        let result = solve(
            &advent_of_code::template::read_file("examples", DAY),
            false,
            true,
        );
        assert_eq!(result, Some(94));
    }

    #[test]
    fn test_part_two() {
        let result = solve(
            &advent_of_code::template::read_file("examples", DAY),
            true,
            true,
        );
        assert_eq!(result, Some(154));
    }
}
