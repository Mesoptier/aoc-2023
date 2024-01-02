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

pub fn part_one(input: &str) -> Option<u32> {
    let grid = input
        .lines()
        .map(|line| {
            line.chars()
                .map(Tile::try_from)
                .map(Result::unwrap)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

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

    // Gather all trails and their lengths
    let mut trails = HashMap::<(usize, usize), Vec<((usize, usize), usize)>>::new();
    let mut visited = HashSet::<(usize, usize)>::new();

    let mut queue = VecDeque::new();
    queue.push_back((start_node, Direction::South));

    while let Some((node, mut direction)) = queue.pop_front() {
        let (mut x, mut y) = node;
        let mut steps = 0;

        // Step in current direction
        (x, y) = direction.step_unchecked(x, y);
        steps += 1;

        loop {
            if (x, y) == target_node {
                trails.entry(node).or_default().push((target_node, steps));
                break;
            }

            match grid[y][x] {
                Tile::Path => {
                    for new_direction in [
                        Direction::North,
                        Direction::South,
                        Direction::East,
                        Direction::West,
                    ] {
                        if new_direction == direction.opposite() {
                            // Don't go back the way we came
                            continue;
                        }

                        let (new_x, new_y) = new_direction.step_unchecked(x, y);
                        if grid[new_y][new_x] != Tile::Forest {
                            x = new_x;
                            y = new_y;
                            direction = new_direction;
                            steps += 1;
                            break;
                        }
                    }
                }
                Tile::Slope(slope_direction) => {
                    // Insert trail to incoming node on this intersection
                    trails.entry(node).or_default().push(((x, y), steps));
                    let node = (x, y);

                    // Step to middle path of this intersection
                    let (mid_x, mid_y) = slope_direction.step_unchecked(x, y);

                    for direction in [
                        Direction::North,
                        Direction::South,
                        Direction::East,
                        Direction::West,
                    ] {
                        let (out_x, out_y) = direction.step_unchecked(mid_x, mid_y);
                        match grid[out_y][out_x] {
                            Tile::Slope(slope_direction) if slope_direction == direction => {
                                trails.entry(node).or_default().push(((out_x, out_y), 2));

                                if visited.insert((out_x, out_y)) {
                                    queue.push_back(((out_x, out_y), direction));
                                }
                            }
                            Tile::Path => {
                                unreachable!("Path should not be adjacent to intersection")
                            }
                            _ => {}
                        }
                    }

                    // Completed this hiking trail
                    break;
                }
                Tile::Forest => unreachable!("Cannot step into forest"),
            }
        }
    }

    // Find longest path from start_node to target_node using BFS
    let mut queue = VecDeque::new();
    queue.push_back((start_node, 0));

    let mut max_steps = 0;
    while let Some((node, steps)) = queue.pop_front() {
        if node == target_node {
            max_steps = max_steps.max(steps);
            continue;
        }

        for (next_node, trail_length) in trails.get(&node).unwrap() {
            queue.push_back((*next_node, steps + trail_length));
        }
    }

    Some(max_steps as u32)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
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
        assert_eq!(result, None);
    }
}
