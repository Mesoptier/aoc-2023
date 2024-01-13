use itertools::chain;
use std::collections::VecDeque;

use advent_of_code::util::coord::{
    Coord, CoordIndexer, DirectedCoord, DirectedCoordIndexer, Direction,
};
use advent_of_code::util::{Indexer, LinearIndexer, VecMap, VecSet, VecTable};

advent_of_code::solution!(16);

type NodeIndex = usize;

struct Node {
    coord: Coord,
    next: [Option<NodeIndex>; 2],
}

impl Node {
    fn empty() -> Self {
        Self {
            coord: Coord::new(usize::MAX, usize::MAX),
            next: [None; 2],
        }
    }
}

struct StartingCoordIndexer {
    width: usize,
    height: usize,
}

impl Indexer<DirectedCoord> for StartingCoordIndexer {
    fn len(&self) -> usize {
        self.width * 2 + self.height * 2
    }

    fn index_for(&self, key: &DirectedCoord) -> usize {
        match key.direction {
            Direction::Down => {
                assert_eq!(key.coord.y, 0);
                key.coord.x
            }
            Direction::Up => {
                assert_eq!(key.coord.y, self.height - 1);
                self.width + key.coord.x
            }
            Direction::Right => {
                assert_eq!(key.coord.x, 0);
                self.width * 2 + key.coord.y
            }
            Direction::Left => {
                assert_eq!(key.coord.x, self.width - 1);
                self.width * 2 + self.height + key.coord.y
            }
        }
    }
}

fn build_nodes(
    map: &VecTable<Coord, char, CoordIndexer>,
) -> (
    Vec<Node>,
    VecMap<DirectedCoord, NodeIndex, StartingCoordIndexer>,
) {
    let width = map.indexer().width;
    let height = map.indexer().height;

    let mut nodes = vec![];
    let mut starting_nodes = VecMap::new(StartingCoordIndexer { width, height });

    struct Frontier {
        in_index: NodeIndex,
        out_index: NodeIndex,
    }

    impl Frontier {
        fn next(nodes: &mut Vec<Node>, coord: Coord) -> Frontier {
            let in_index = nodes.len();
            let in_node = Node {
                coord,
                next: [None; 2],
            };
            nodes.push(in_node);

            let out_index = nodes.len();
            let out_node = Node::empty(); // will be filled when frontier is closed
            nodes.push(out_node);

            Frontier {
                in_index,
                out_index,
            }
        }
    }

    let mut vertical_frontiers = Vec::with_capacity(width);
    for x in 0..width {
        let coord = Coord::new(x, 0);
        let vertical_frontier = Frontier::next(&mut nodes, coord);
        let starting_node_index = nodes.len();
        nodes.push(Node {
            coord,
            next: [Some(vertical_frontier.out_index), None],
        });
        starting_nodes.insert(
            &DirectedCoord {
                coord,
                direction: Direction::Down,
            },
            starting_node_index,
        );
        vertical_frontiers.push(vertical_frontier);
    }

    for y in 0..height {
        let coord = Coord::new(0, y);
        let mut horizontal_frontier = Frontier::next(&mut nodes, coord);
        let starting_node_index = nodes.len();
        nodes.push(Node {
            coord,
            next: [Some(horizontal_frontier.out_index), None],
        });
        starting_nodes.insert(
            &DirectedCoord {
                coord,
                direction: Direction::Right,
            },
            starting_node_index,
        );

        for x in 0..width {
            let coord = Coord::new(x, y);

            if *map.get(&coord) == '.' {
                continue;
            }

            let vertical_frontier = &mut vertical_frontiers[x];

            let next_horizontal_frontier = Frontier::next(&mut nodes, coord);
            let next_vertical_frontier = Frontier::next(&mut nodes, coord);

            nodes[horizontal_frontier.out_index].coord = coord;
            nodes[vertical_frontier.out_index].coord = coord;

            match map.get(&coord) {
                '/' => {
                    nodes[horizontal_frontier.out_index].next =
                        [Some(vertical_frontier.in_index), None];
                    nodes[vertical_frontier.out_index].next =
                        [Some(horizontal_frontier.in_index), None];
                    nodes[next_horizontal_frontier.in_index].next =
                        [Some(next_vertical_frontier.out_index), None];
                    nodes[next_vertical_frontier.in_index].next =
                        [Some(next_horizontal_frontier.out_index), None];
                }
                '\\' => {
                    nodes[horizontal_frontier.out_index].next =
                        [Some(next_vertical_frontier.out_index), None];
                    nodes[vertical_frontier.out_index].next =
                        [Some(next_horizontal_frontier.out_index), None];
                    nodes[next_horizontal_frontier.in_index].next =
                        [Some(vertical_frontier.in_index), None];
                    nodes[next_vertical_frontier.in_index].next =
                        [Some(horizontal_frontier.in_index), None];
                }
                '|' => {
                    nodes[horizontal_frontier.out_index].next = [
                        Some(vertical_frontier.in_index),
                        Some(next_vertical_frontier.out_index),
                    ];
                    nodes[vertical_frontier.out_index].next =
                        [Some(next_vertical_frontier.out_index), None];
                    nodes[next_horizontal_frontier.in_index].next = [
                        Some(vertical_frontier.in_index),
                        Some(next_vertical_frontier.out_index),
                    ];
                    nodes[next_vertical_frontier.in_index].next =
                        [Some(vertical_frontier.in_index), None];
                }
                '-' => {
                    nodes[horizontal_frontier.out_index].next =
                        [Some(next_horizontal_frontier.out_index), None];
                    nodes[vertical_frontier.out_index].next = [
                        Some(horizontal_frontier.in_index),
                        Some(next_horizontal_frontier.out_index),
                    ];
                    nodes[next_horizontal_frontier.in_index].next =
                        [Some(horizontal_frontier.in_index), None];
                    nodes[next_vertical_frontier.in_index].next = [
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
        nodes[horizontal_frontier.out_index].coord = Coord::new(width - 1, y);

        let node_index = nodes.len();
        nodes.push(Node {
            coord: Coord::new(width - 1, y),
            next: [Some(horizontal_frontier.in_index); 2],
        });
        starting_nodes.insert(
            &DirectedCoord::new(width - 1, y, Direction::Left),
            node_index,
        );
    }

    // Close the last vertical frontiers.
    for (x, vertical_frontier) in vertical_frontiers.into_iter().enumerate() {
        nodes[vertical_frontier.out_index].coord = Coord::new(x, height - 1);

        let node_index = nodes.len();
        nodes.push(Node {
            coord: Coord::new(x, height - 1),
            next: [Some(vertical_frontier.in_index); 2],
        });
        starting_nodes.insert(
            &DirectedCoord::new(x, height - 1, Direction::Up),
            node_index,
        );
    }

    (nodes, starting_nodes)
}

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

fn foo(nodes: &[Node], node_index: NodeIndex, indexer: CoordIndexer) -> u32 {
    let mut queue = VecDeque::<NodeIndex>::new();
    let mut visited = VecSet::new(LinearIndexer::new(nodes.len()));
    queue.push_front(node_index);
    visited.insert(node_index);

    let mut energized_count = 0;
    let mut energized = VecSet::new(indexer);
    energized.insert(nodes[node_index].coord);
    energized_count += 1;

    while let Some(node_index) = queue.pop_front() {
        let node = &nodes[node_index];

        for next_node_index in node.next.iter().flatten() {
            let next_node = &nodes[*next_node_index];

            let min_x = next_node.coord.x.min(node.coord.x);
            let max_x = next_node.coord.x.max(node.coord.x);
            let min_y = next_node.coord.y.min(node.coord.y);
            let max_y = next_node.coord.y.max(node.coord.y);

            assert!(min_x == max_x || min_y == max_y);

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
    let (nodes, starting_nodes) = build_nodes(&map);

    foo(
        &nodes,
        *starting_nodes
            .get(&DirectedCoord {
                coord: Coord { x: 0, y: 0 },
                direction: Direction::Right,
            })
            .unwrap(),
        *map.indexer(),
    )
    .into()

    // compute_energized_tiles(
    //     &map,
    //     DirectedCoord {
    //         coord: Coord { x: 0, y: 0 },
    //         direction: Direction::Right,
    //     },
    //     &mut VecMap::new(DirectedCoordIndexer::from(*map.indexer())),
    // )
    // .into()
}

pub fn part_two(input: &str) -> Option<u32> {
    let map = parse_input(input);
    let (nodes, starting_nodes) = build_nodes(&map);

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
    .map(|beam_front| {
        foo(
            &nodes,
            *starting_nodes.get(&beam_front).unwrap(),
            *map.indexer(),
        )
    })
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
