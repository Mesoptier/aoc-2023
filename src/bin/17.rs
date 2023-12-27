use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap};
advent_of_code::solution!(17);

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Copy, Clone)]
struct State {
    x: usize,
    y: usize,
    direction: Direction,
    direction_steps: usize,
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct Entry {
    cost: u32,
    state: State,
}

pub fn part_one(input: &str) -> Option<u32> {
    let map: Vec<Vec<u32>> = input
        .lines()
        .map(|line| line.chars().map(|c| c.to_digit(10).unwrap()).collect())
        .collect();

    let width = map[0].len();
    let height = map.len();

    let initial_state = State {
        x: 0,
        y: 0,
        direction: Direction::Down,
        direction_steps: 0,
    };

    let mut min_heap = BinaryHeap::<Reverse<Entry>>::new();
    min_heap.push(Reverse(Entry {
        cost: 0,
        state: initial_state,
    }));

    let mut best_costs = HashMap::<State, u32>::new();

    while let Some(Reverse(entry)) = min_heap.pop() {
        let Entry { cost, state } = entry;

        if state.x == width - 1 && state.y == height - 1 {
            // Found the destination
            return Some(cost);
        }

        if let Some(best_cost) = best_costs.get(&state) {
            if *best_cost <= cost {
                // Already found a better path to this state
                continue;
            }
        }

        // Update the best cost for this state
        best_costs.insert(state, cost);

        for direction in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            if direction == state.direction.opposite() {
                // Can't reverse direction
                continue;
            }

            let direction_steps = if direction == state.direction {
                state.direction_steps + 1
            } else {
                1
            };

            if direction_steps > 3 {
                // Can't go in the same direction for more than 3 steps
                continue;
            }

            let next_state = match direction {
                Direction::Up if state.y > 0 => Some(State {
                    x: state.x,
                    y: state.y - 1,
                    direction,
                    direction_steps,
                }),
                Direction::Right if state.x + 1 < width => Some(State {
                    x: state.x + 1,
                    y: state.y,
                    direction,
                    direction_steps,
                }),
                Direction::Down if state.y + 1 < height => Some(State {
                    x: state.x,
                    y: state.y + 1,
                    direction,
                    direction_steps,
                }),
                Direction::Left if state.x > 0 => Some(State {
                    x: state.x - 1,
                    y: state.y,
                    direction,
                    direction_steps,
                }),
                _ => None,
            };

            if let Some(next_state) = next_state {
                let next_cost = cost + map[next_state.y][next_state.x];
                min_heap.push(Reverse(Entry {
                    cost: next_cost,
                    state: next_state,
                }));
            }
        }
    }

    None
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
        assert_eq!(result, Some(102));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
