#![feature(portable_simd)]

use std::ops::BitAnd;
use std::simd::prelude::*;

use arrayvec::ArrayVec;
use itertools::izip;
use petgraph::visit::EdgeRef;

use advent_of_code::util::{BitSet, Indexer, LinearIndexer, VecTable};

use crate::tile_grid::Tile;

advent_of_code::solution!(23);

pub fn part_one(input: &str) -> Option<Cost> {
    solve(input, false)
}

pub fn part_two(input: &str) -> Option<Cost> {
    solve(input, true)
}

type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;

type NodeIndex = u32;
type Cost = u32;

fn solve(input: &str, part_two: bool) -> Option<Cost> {
    let (adj_list, start_node, target_node) = build_trails_map(input, part_two);

    debug_assert!(adj_list.len() <= 34);
    debug_assert_eq!(start_node, adj_list.len() - 2);
    debug_assert_eq!(target_node, adj_list.len() - 1);

    // ADJACENCY LIST
    //
    // The adjacency list is optimized such that:
    //  - Start node has no incoming edges
    //  - Target node has no outgoing edges
    //  - There are `N <= 34` nodes, where nodes with indices:
    //     - `0..=N-3` are internal nodes,
    //     - `N-2` is the start node,
    //     - `N-1` is the target node.

    // BITSETS
    //
    // We use bitsets to represent various sets of (internal) nodes:
    //  - `visited`: The set of nodes visited by the current path.
    //  - `reachable`: The set of nodes reachable from the current node without re-visiting any nodes in `visited`.
    //  - `image[i]`: The set of nodes that have an edge incoming from node `i`. Used to compute `reachable`.
    //
    // Special care is taken to ensure that we can represent these sets in only 32 bits. In particular, the start and
    // target nodes are never included in any of these sets, because:
    //  - `visited`:
    //      - The start node is always visited.
    //      - The target node is never visited, because we finish the path as soon as we reach it.
    //  - `reachable`:
    //      - The start node is never reachable, because it is always visited.
    //      - The target node is always reachable, because otherwise we would pruned the path.
    //      - Instead of checking `reachable[target_node]` to see if the path is still viable,
    //        we can check `reachable[preimage(target_node)]`.
    //  - `image[i]`: Used to compute `reachable`, so has the same size as `reachable`.

    // Sort the trails by length, so DFS considers the longest trails first. (Note the list is sorted in increasing
    // order, but since the stack is LIFO, the longest trails will be considered first.)
    let adj_list = {
        let mut adj_list = adj_list;
        for trails in adj_list.values_mut() {
            trails.sort_unstable_by_key(|&(_, cost)| cost);
        }
        adj_list
    };

    let mut stack = Vec::new();
    let mut max_path_cost = 0;

    let mut cache = Cache::new(*adj_list.0.indexer());
    let compute_reachable = ComputeReachable::new(&adj_list);
    let target_preimage = adj_list.preimage(target_node);

    // Cannot push start node to stack here, because its index is out of bounds for the bitsets.
    stack.extend(
        adj_list
            .get(start_node)
            .iter()
            .map(|&(next_node, next_cost)| (next_node, next_cost, 0)),
    );

    while let Some((node, path_cost, mut visited)) = stack.pop() {
        if node == target_node {
            max_path_cost = max_path_cost.max(path_cost);
            continue;
        }

        debug_assert_ne!(node, start_node);
        debug_assert_ne!(node, target_node);

        if visited.get(node) {
            continue;
        }

        // Compute the set of nodes reachable from this node
        let reachable = compute_reachable.compute_reachable(node, &visited);

        // Prune the path if we can't reach the target node from this node
        if reachable.is_disjoint(&target_preimage) {
            continue;
        }

        // Prune the path if we've already found a path to this node that can still reach the same set (or a superset!)
        // of nodes at a better cost.
        if !cache.insert_if_max(node, reachable, path_cost) {
            continue;
        }

        visited.set(node);

        stack.extend(
            adj_list
                .get(node)
                .iter()
                .map(|&(next_node, next_cost)| (next_node, path_cost + next_cost, visited)),
        );
    }

    Some(max_path_cost)
}

fn build_trails_map(input: &str, part_two: bool) -> (AdjacencyList, NodeIndex, NodeIndex) {
    let tile_grid = tile_grid::TileGrid::new(input);

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

    let graph = graph::build_graph(tile_grid, start_coord, target_coord, part_two);

    let part = if part_two { 2 } else { 1 };
    let engine = if part_two { "neato" } else { "dot" };
    if cfg!(feature = "debug_output") {
        graph::print_graph(
            &graph.clone().into_graph().map(
                |_, coord| format!("({}, {})", coord.x, coord.y),
                |_, cost| *cost,
            ),
            &format!("23-{}", part),
            engine,
        );
    }

    let (graph, start_node, target_node) =
        graph::optimize_graph(graph, start_coord, target_coord, part_two);

    if cfg!(feature = "debug_output") {
        graph::print_graph(
            &graph.map(
                |node_index, _| format!("{}", node_index.index()),
                |_, cost| *cost,
            ),
            &format!("23-{}-opt", part),
            engine,
        );
    }

    // Convert to a VecTable
    let adj_list_data = graph
        .node_indices()
        .map(|node| {
            graph
                .edges(node)
                .map(|edge| (edge.target().index() as NodeIndex, *edge.weight()))
                .collect::<ArrayVec<_, MAX_DEGREE>>()
        })
        .collect::<Vec<_>>();
    let indexer = LinearIndexer::new(adj_list_data.len() as NodeIndex);
    let adj_list = VecTable::from_vec(adj_list_data, indexer);

    (
        AdjacencyList(adj_list),
        start_node.index() as NodeIndex,
        target_node.index() as NodeIndex,
    )
}

const MAX_DEGREE: usize = 4;
struct AdjacencyList(
    VecTable<NodeIndex, ArrayVec<(NodeIndex, Cost), MAX_DEGREE>, LinearIndexer<NodeIndex>>,
);

impl AdjacencyList {
    #[inline]
    fn len(&self) -> NodeIndex {
        self.0.indexer().len() as NodeIndex
    }

    #[inline]
    fn get(&self, node: NodeIndex) -> &ArrayVec<(NodeIndex, Cost), MAX_DEGREE> {
        &self.0[node]
    }

    #[inline]
    fn is_internal(&self, node: NodeIndex) -> bool {
        node < self.len() - 2
    }

    #[inline]
    fn values_mut(&mut self) -> impl Iterator<Item = &mut ArrayVec<(NodeIndex, Cost), MAX_DEGREE>> {
        self.0.values_mut()
    }

    /// Returns the set of internal nodes that have an incoming edge from `node`.
    #[inline]
    fn image(&self, node: NodeIndex) -> u32 {
        self.0[node]
            .iter()
            .filter(|(node, _)| self.is_internal(*node))
            .map(|(node, _)| 1 << node)
            .fold(0, |a, b| a | b)
    }

    /// Returns the set of internal nodes that have an outgoing edge to `node`.
    #[inline]
    fn preimage(&self, node: NodeIndex) -> u32 {
        self.0
            .iter()
            .filter(|(_, neighbors)| neighbors.iter().any(|(neighbor, _)| *neighbor == node))
            .filter(|(node, _)| self.is_internal(*node))
            .map(|(node, _)| 1 << node)
            .fold(0, |a, b| a | b)
    }
}

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
    use itertools::Itertools;
    use petgraph::graph::{DiGraph, NodeIndex};
    use petgraph::graphmap::DiGraphMap;

    use advent_of_code::util::coord::Direction;

    use crate::tile_grid::{Tile, TileGrid};
    use crate::{Coord, Cost};

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
        part_two: bool,
    ) -> (DiGraph<(), Cost>, NodeIndex, NodeIndex) {
        let start_coord = if part_two {
            // Merge start node into the first intersection node
            merge_into_single_neighbor(&mut graph, start_coord)
        } else {
            start_coord
        };
        // Remove incoming edges to start node, since the path can never return to it
        remove_incoming_edges(&mut graph, start_coord);

        let target_coord = if part_two {
            // Merge target node into the last intersection node
            merge_into_single_neighbor(&mut graph, target_coord)
        } else {
            target_coord
        };
        // Remove outgoing edges from target node, since the path can never leave it
        remove_outgoing_edges(&mut graph, target_coord);

        if part_two {
            for node in graph
                .neighbors_directed(start_coord, petgraph::Direction::Outgoing)
                .collect_vec()
            {
                // Follow the perimeter of the graph, removing all edges moving away from the target node
                let mut node = node;
                while node != target_coord {
                    let next_node = graph
                        .neighbors_directed(node, petgraph::Direction::Outgoing)
                        .find(|&neighbor| graph.neighbors(neighbor).count() <= 3)
                        .unwrap();
                    graph.remove_edge(next_node, node);
                    node = next_node;
                }
            }
        }

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
                .map(|(source, target, cost)| {
                    (node_index_map[&source], node_index_map[&target], cost)
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

    pub fn print_graph(graph: &DiGraph<String, Cost>, name: &str, engine: &str) {
        use std::io::Write;

        let mut file = std::fs::File::create(format!("data/viz/{}.dot", name)).unwrap();
        write!(file, "{}", petgraph::dot::Dot::new(&graph)).unwrap();

        // Run dot to generate SVG
        std::process::Command::new("dot")
            .args([
                "-Tsvg",
                format!("data/viz/{}.dot", name).as_str(),
                "-o",
                format!("data/viz/{}.svg", name).as_str(),
                format!("-K{}", engine).as_str(),
            ])
            .output()
            .unwrap();
    }
}

struct Cache {
    cache: VecTable<NodeIndex, (Vec<u32>, Vec<u32>), LinearIndexer<NodeIndex>>,
}

impl Cache {
    fn new(indexer: LinearIndexer<NodeIndex>) -> Self {
        Cache {
            cache: VecTable::new(indexer),
        }
    }

    /// Inserts a new (node, bitset)-value pair into the cache if a pair with a superset bitset and a higher value is
    /// not already present.
    ///
    /// Returns `true` if the new value was inserted.
    fn insert_if_max(&mut self, node: NodeIndex, query_bitset: u32, query_value: u32) -> bool {
        let (bitsets, values) = &self.cache[node];
        assert_eq!(bitsets.len(), values.len());

        const LANES: usize = 32;

        // Process existing entries in reverse order so newer (and thus superseding) entries are processed first.
        // TODO: Could we instead replace superseded entries with the new entry?
        let bitsets = bitsets.rchunks_exact(LANES);
        let values = values.rchunks_exact(LANES);

        let bitsets_remainder = bitsets.remainder();
        let values_remainder = values.remainder();

        let query_bitsets = Simd::<u32, LANES>::splat(query_bitset);
        let query_values = Simd::<u32, LANES>::splat(query_value);

        for (bitsets, values) in izip!(bitsets, values) {
            let bitsets = Simd::<u32, LANES>::from_slice(bitsets);
            let values = Simd::<u32, LANES>::from_slice(values);

            // bitset & query_bitset == query_bitset (i.e. query_bitset is a subset of bitset)
            let mask = (bitsets & query_bitsets).simd_eq(query_bitsets);

            // value >= query_value
            let mask = mask & values.simd_ge(query_values);

            if mask.any() {
                return false;
            }
        }

        for (&bitset, &value) in izip!(bitsets_remainder, values_remainder) {
            if bitset & query_bitset == query_bitset && value >= query_value {
                return false;
            }
        }

        // Insert the new pair
        let (bitsets, values) = &mut self.cache[node];
        bitsets.push(query_bitset);
        values.push(query_value);
        true
    }
}

struct ComputeReachable {
    /// `image[i]` is the set of nodes that have an edge incoming from node `i`.
    image: u32x32,
}

impl ComputeReachable {
    fn new(adj_list: &AdjacencyList) -> Self {
        let mut image = [0; 32];
        for node in 0..adj_list.len() - 2 {
            image[node as usize] = adj_list.image(node);
        }
        let image = u32x32::from_array(image);

        ComputeReachable { image }
    }

    fn compute_reachable(&self, node: NodeIndex, visited: &u32) -> u32 {
        let mut reachable = 0;
        // Start search from `node`
        reachable.set(node);

        // Filter out nodes in advance that have already been visited
        let unvisited_image = self.image.bitand(u32x32::splat(!visited));

        loop {
            // For each node `i` in `reachable`, select the set of unvisited nodes that have an edge incoming from `i`,
            // and add them to the `reachable` set.
            let next_reachable = reachable
                | mask32x32::from_bitmask(reachable as u64)
                    .select(unvisited_image, u32x32::splat(0))
                    .reduce_or();

            if next_reachable == reachable {
                // Didn't reach any new nodes, so we're done
                return reachable;
            }

            reachable = next_reachable;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = solve(&advent_of_code::template::read_file("examples", DAY), false);
        assert_eq!(result, Some(94));
    }

    #[test]
    fn test_part_two() {
        let result = solve(&advent_of_code::template::read_file("examples", DAY), true);
        assert_eq!(result, Some(154));
    }
}
