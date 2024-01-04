use arrayvec::ArrayVec;
use std::collections::{HashMap, HashSet, VecDeque};
advent_of_code::solution!(23);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    fn step_unchecked(&self, x: usize, y: usize) -> (usize, usize) {
        match self {
            Direction::North => (x, y - 1),
            Direction::South => (x, y + 1),
            Direction::East => (x + 1, y),
            Direction::West => (x - 1, y),
        }
    }
}

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
            '^' => Ok(Tile::Slope(Direction::North)),
            'v' => Ok(Tile::Slope(Direction::South)),
            '>' => Ok(Tile::Slope(Direction::East)),
            '<' => Ok(Tile::Slope(Direction::West)),
            _ => Err(()),
        }
    }
}

fn solve(input: &str, part_two: bool) -> Option<u32> {
    type Node = (usize, usize);
    type TrailsMap = HashMap<Node, ArrayVec<(Node, u32), 12>>;

    let grid = input
        .lines()
        .map(|line| {
            line.chars()
                .map(Tile::try_from)
                .map(Result::unwrap)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // Find the start and target nodes
    let start_node = (
        grid.first()
            .unwrap()
            .iter()
            .position(|&tile| tile == Tile::Path)
            .unwrap(),
        0,
    );
    let target_node = (
        grid.last()
            .unwrap()
            .iter()
            .position(|&tile| tile == Tile::Path)
            .unwrap(),
        grid.len() - 1,
    );

    // Gather all trails
    let mut trails_map = TrailsMap::new();

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();

    queue.push_back((start_node, Direction::South));
    visited.insert(start_node);

    while let Some((node, direction)) = queue.pop_front() {
        let (mut x, mut y) = node;
        let mut steps = 0;

        (x, y) = direction.step_unchecked(x, y);
        steps += 1;

        let mut incoming_direction = direction;

        loop {
            if (x, y) == target_node || (x, y) == start_node {
                trails_map.entry(node).or_default().push(((x, y), steps));
                break;
            }

            let paths = [
                Direction::North,
                Direction::South,
                Direction::East,
                Direction::West,
            ]
            .into_iter()
            .filter(|&direction| direction != incoming_direction.opposite())
            .filter_map(|direction| {
                let (x, y) = direction.step_unchecked(x, y);
                match grid[y][x] {
                    Tile::Path => Some((x, y, direction)),
                    Tile::Slope(slope_direction) if slope_direction == direction || part_two => {
                        Some((x, y, direction))
                    }
                    _ => None,
                }
            })
            .collect::<ArrayVec<_, 4>>();

            match paths.len() {
                0 => unreachable!("Invalid trail"),
                1 => {
                    // Continue on the same path
                    (x, y, incoming_direction) = paths[0];
                    steps += 1;
                }
                _ => {
                    // Node is an intersection
                    trails_map.entry(node).or_default().push(((x, y), steps));

                    if visited.insert((x, y)) {
                        // We haven't visited this node before, so we need to explore it
                        for (_, _, direction) in paths {
                            queue.push_back(((x, y), direction));
                        }

                        if part_two {
                            queue.push_back(((x, y), incoming_direction.opposite()));
                        }
                    }

                    break;
                }
            }
        }
    }

    // Find the longest hike from the start to the target, without visiting any node twice
    fn find_longest_hike(
        node: Node,
        target_node: Node,
        trails: &TrailsMap,
        visited: &mut HashSet<Node>,
    ) -> Option<u32> {
        if node == target_node {
            // We've reached the target node
            return Some(0);
        }

        if !visited.insert(node) {
            // We've already visited this node, so we can't continue exploring
            return None;
        }

        let result = trails[&node]
            .iter()
            .filter_map(|&(next_node, steps)| {
                find_longest_hike(next_node, target_node, trails, visited)
                    .map(|next_steps| next_steps + steps)
            })
            .max();

        visited.remove(&node);
        result
    }

    find_longest_hike(start_node, target_node, &trails_map, &mut HashSet::new())
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
