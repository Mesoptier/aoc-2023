use std::collections::VecDeque;

use itertools::chain;

use advent_of_code::util::coord::Direction;
use advent_of_code::util::{Indexer, LinearIndexer, VecMap, VecSet, VecTable};

advent_of_code::solution!(16);

type NodeIndex = u32;
type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type DirectedCoord = advent_of_code::util::coord::DirectedCoord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;

struct Node {
    coord: Coord,
    next: [Option<NodeIndex>; 2],
}

impl Node {
    fn empty() -> Self {
        Self {
            coord: Coord::new(CoordT::MAX, CoordT::MAX),
            next: [None; 2],
        }
    }
}

struct StartingCoordIndexer {
    width: CoordT,
    height: CoordT,
}

impl Indexer<DirectedCoord> for StartingCoordIndexer {
    fn len(&self) -> usize {
        (self.width * 2 + self.height * 2) as usize
    }

    fn index_for(&self, key: &DirectedCoord) -> usize {
        (match key.direction {
            Direction::Down => {
                debug_assert_eq!(key.coord.y, 0);
                key.coord.x
            }
            Direction::Up => {
                debug_assert_eq!(key.coord.y, self.height - 1);
                self.width + key.coord.x
            }
            Direction::Right => {
                debug_assert_eq!(key.coord.x, 0);
                self.width * 2 + key.coord.y
            }
            Direction::Left => {
                debug_assert_eq!(key.coord.x, self.width - 1);
                self.width * 2 + self.height + key.coord.y
            }
        }) as usize
    }
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
    let width = width.unwrap() as CoordT;
    let height = (data.len() as CoordT) / width;
    let indexer = CoordIndexer::new(width, height);
    VecTable::from_vec(data, indexer)
}

fn build_nodes(
    map: &VecTable<Coord, char, CoordIndexer>,
) -> (
    Vec<Node>,
    VecMap<DirectedCoord, NodeIndex, StartingCoordIndexer>,
) {
    let width = map.indexer().width as CoordT;
    let height = map.indexer().height as CoordT;

    // Each special character (|, -, \, /) is represented by four nodes, one for each incoming direction.
    // Each node has one or two outgoing directions, the nodes for which are stored in the `next` array.
    let mut nodes = vec![];
    // Map from directed coordinates along the edge of the map to the index of the node that represents them.
    let mut starting_nodes = VecMap::new(StartingCoordIndexer { width, height });

    // A frontier represents a segment of empty space south/east of a special character (or the edge of the map).
    struct Frontier {
        // A beam traveling the segment in the reverse direction (north/west) goes to the node `in_index`.
        in_index: NodeIndex,
        // A beam traveling the segment in the forward direction (south/east) goes to the node `out_index`.
        // The actual node is only created when the frontier is closed.
        out_index: NodeIndex,
    }

    fn open_frontier(nodes: &mut Vec<Node>, coord: Coord) -> Frontier {
        let in_index = nodes.len() as NodeIndex;
        let in_node = Node {
            coord,
            next: [None; 2],
        };
        nodes.push(in_node);

        let out_index = nodes.len() as NodeIndex;
        let out_node = Node::empty(); // will be filled when frontier is closed
        nodes.push(out_node);

        Frontier {
            in_index,
            out_index,
        }
    }

    /// Opens a frontier starting at the edge of the map.
    fn open_first_frontier(
        coord: Coord,
        direction: Direction,
        nodes: &mut Vec<Node>,
        starting_nodes: &mut VecMap<DirectedCoord, NodeIndex, StartingCoordIndexer>,
    ) -> Frontier {
        let frontier = open_frontier(nodes, coord);

        // Add starting node from edge
        let node_index = nodes.len() as NodeIndex;
        nodes.push(Node {
            coord,
            next: [Some(frontier.out_index), None],
        });
        starting_nodes.insert(&DirectedCoord { coord, direction }, node_index);
        frontier
    }

    /// Closes a frontier at the edge of the map.
    fn close_last_frontier(
        frontier: Frontier,
        coord: Coord,
        direction: Direction,
        nodes: &mut Vec<Node>,
        starting_nodes: &mut VecMap<DirectedCoord, NodeIndex, StartingCoordIndexer>,
    ) {
        nodes[frontier.out_index as usize].coord = coord;

        // Add starting node from edge
        let node_index = nodes.len() as NodeIndex;
        nodes.push(Node {
            coord,
            next: [Some(frontier.in_index), None],
        });
        starting_nodes.insert(&DirectedCoord { coord, direction }, node_index);
    }

    let mut vertical_frontiers = Vec::with_capacity(width as usize);
    for x in 0..width {
        vertical_frontiers.push(open_first_frontier(
            Coord::new(x, 0),
            Direction::Down,
            &mut nodes,
            &mut starting_nodes,
        ));
    }

    for y in 0..height {
        let mut horizontal_frontier = open_first_frontier(
            Coord::new(0, y),
            Direction::Right,
            &mut nodes,
            &mut starting_nodes,
        );

        for x in 0..width {
            let coord = Coord::new(x, y);

            if *map.get(&coord) == '.' {
                continue;
            }

            let vertical_frontier = &mut vertical_frontiers[x as usize];

            let next_horizontal_frontier = open_frontier(&mut nodes, coord);
            let next_vertical_frontier = open_frontier(&mut nodes, coord);

            nodes[horizontal_frontier.out_index as usize].coord = coord;
            nodes[vertical_frontier.out_index as usize].coord = coord;

            match map.get(&coord) {
                '/' => {
                    nodes[horizontal_frontier.out_index as usize].next =
                        [Some(vertical_frontier.in_index), None];
                    nodes[vertical_frontier.out_index as usize].next =
                        [Some(horizontal_frontier.in_index), None];
                    nodes[next_horizontal_frontier.in_index as usize].next =
                        [Some(next_vertical_frontier.out_index), None];
                    nodes[next_vertical_frontier.in_index as usize].next =
                        [Some(next_horizontal_frontier.out_index), None];
                }
                '\\' => {
                    nodes[horizontal_frontier.out_index as usize].next =
                        [Some(next_vertical_frontier.out_index), None];
                    nodes[vertical_frontier.out_index as usize].next =
                        [Some(next_horizontal_frontier.out_index), None];
                    nodes[next_horizontal_frontier.in_index as usize].next =
                        [Some(vertical_frontier.in_index), None];
                    nodes[next_vertical_frontier.in_index as usize].next =
                        [Some(horizontal_frontier.in_index), None];
                }
                '|' => {
                    nodes[horizontal_frontier.out_index as usize].next = [
                        Some(vertical_frontier.in_index),
                        Some(next_vertical_frontier.out_index),
                    ];
                    nodes[vertical_frontier.out_index as usize].next =
                        [Some(next_vertical_frontier.out_index), None];
                    nodes[next_horizontal_frontier.in_index as usize].next = [
                        Some(vertical_frontier.in_index),
                        Some(next_vertical_frontier.out_index),
                    ];
                    nodes[next_vertical_frontier.in_index as usize].next =
                        [Some(vertical_frontier.in_index), None];
                }
                '-' => {
                    nodes[horizontal_frontier.out_index as usize].next =
                        [Some(next_horizontal_frontier.out_index), None];
                    nodes[vertical_frontier.out_index as usize].next = [
                        Some(horizontal_frontier.in_index),
                        Some(next_horizontal_frontier.out_index),
                    ];
                    nodes[next_horizontal_frontier.in_index as usize].next =
                        [Some(horizontal_frontier.in_index), None];
                    nodes[next_vertical_frontier.in_index as usize].next = [
                        Some(horizontal_frontier.in_index),
                        Some(next_horizontal_frontier.out_index),
                    ];
                }
                _ => unreachable!(),
            }

            horizontal_frontier = next_horizontal_frontier;
            *vertical_frontier = next_vertical_frontier;
        }

        // Close the last horizontal frontier.
        close_last_frontier(
            horizontal_frontier,
            Coord::new(width - 1, y),
            Direction::Left,
            &mut nodes,
            &mut starting_nodes,
        );
    }

    // Close the last vertical frontiers.
    for (x, vertical_frontier) in vertical_frontiers.into_iter().enumerate() {
        close_last_frontier(
            vertical_frontier,
            Coord::new(x as CoordT, height - 1),
            Direction::Up,
            &mut nodes,
            &mut starting_nodes,
        );
    }

    (nodes, starting_nodes)
}

/// Computes the number of tiles beams will travel through if they start at the given node, excluding loops.
/// This functions as an upper bound on the number of energized tiles.
fn build_length_remaining(nodes: &[Node]) -> VecMap<NodeIndex, u32, LinearIndexer<NodeIndex>> {
    let mut length_remaining = VecMap::new(LinearIndexer::new(nodes.len() as NodeIndex));

    for node_index in 0..(nodes.len() as NodeIndex) {
        if length_remaining.contains_key(&node_index) {
            continue;
        }

        build_length_remaining_inner(
            nodes,
            node_index,
            &mut length_remaining,
            &mut VecSet::new(LinearIndexer::new(nodes.len() as NodeIndex)),
        );
    }

    length_remaining
}

fn build_length_remaining_inner(
    nodes: &[Node],
    node_index: NodeIndex,
    length_remaining: &mut VecMap<NodeIndex, u32, LinearIndexer<NodeIndex>>,
    visited: &mut VecSet<NodeIndex, LinearIndexer<NodeIndex>>,
) -> u32 {
    if !visited.insert(node_index) {
        return 0;
    }

    if let Some(length) = length_remaining.get(&node_index) {
        return *length;
    }

    let node = &nodes[node_index as usize];
    let length = node
        .next
        .iter()
        .flatten()
        .map(|next_node_index| {
            let next_node = &nodes[*next_node_index as usize];
            let dist =
                node.coord.x.abs_diff(next_node.coord.x) + node.coord.y.abs_diff(next_node.coord.y);

            build_length_remaining_inner(nodes, *next_node_index, length_remaining, visited) + dist
        })
        .sum::<u32>();
    length_remaining.insert(&node_index, length);
    length
}

fn compute_energized_tiles(
    nodes: &[Node],
    node_index: NodeIndex,
    indexer: CoordIndexer,
    length_remaining: &VecMap<NodeIndex, u32, LinearIndexer<NodeIndex>>,
    current_max_energized_tiles: u32,
) -> u32 {
    let mut queue = VecDeque::<NodeIndex>::new();
    let mut visited = VecSet::new(LinearIndexer::new(nodes.len() as NodeIndex));
    queue.push_front(node_index);
    visited.insert(node_index);

    let mut energized_count = 0;
    let mut energized = VecSet::new(indexer);
    energized.insert(nodes[node_index as usize].coord);
    energized_count += 1;

    while let Some(node_index) = queue.pop_front() {
        if queue.is_empty()
            && energized_count + length_remaining.get(&node_index).unwrap()
                <= current_max_energized_tiles
        {
            return energized_count;
        }

        let node = &nodes[node_index as usize];

        for next_node_index in node.next.iter().flatten() {
            let next_node = &nodes[*next_node_index as usize];

            let min_x = next_node.coord.x.min(node.coord.x);
            let max_x = next_node.coord.x.max(node.coord.x);
            let min_y = next_node.coord.y.min(node.coord.y);
            let max_y = next_node.coord.y.max(node.coord.y);

            debug_assert!(min_x == max_x || min_y == max_y);

            for x in min_x..=max_x {
                for y in min_y..=max_y {
                    let coord = Coord { x, y };
                    if energized.insert(coord) {
                        energized_count += 1;
                    }
                }
            }

            if visited.insert(*next_node_index) {
                queue.push_back(*next_node_index);
            }
        }
    }

    energized_count
}

pub fn part_one(input: &str) -> Option<u32> {
    let map = parse_input(input);
    let (nodes, starting_nodes) = build_nodes(&map);

    let length_remaining_map = build_length_remaining(&nodes);

    compute_energized_tiles(
        &nodes,
        *starting_nodes
            .get(&DirectedCoord {
                coord: Coord { x: 0, y: 0 },
                direction: Direction::Right,
            })
            .unwrap(),
        *map.indexer(),
        &length_remaining_map,
        0,
    )
    .into()
}

pub fn part_two(input: &str) -> Option<u32> {
    let map = parse_input(input);
    let (nodes, starting_nodes) = build_nodes(&map);

    let length_remaining_map = build_length_remaining(&nodes);

    let width = map.indexer().width;
    let height = map.indexer().height;

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
    .fold(0, |current_max_energized_count, beam_front| {
        compute_energized_tiles(
            &nodes,
            *starting_nodes.get(&beam_front).unwrap(),
            *map.indexer(),
            &length_remaining_map,
            current_max_energized_count,
        )
        .max(current_max_energized_count)
    })
    .into()
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
