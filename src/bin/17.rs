use std::cmp::Ordering;
use std::collections::BinaryHeap;

use advent_of_code::util::coord::{
    Coord, CoordIndexer, DirectedCoord, DirectedCoordIndexer, Direction,
};
use advent_of_code::util::{LinearIndexer, VecMap, VecTable};

advent_of_code::solution!(17);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct State {
    directed_coord: DirectedCoord,
    direction_steps: usize,
}

impl State {
    fn new(x: usize, y: usize, direction: Direction, direction_steps: usize) -> Self {
        Self {
            directed_coord: DirectedCoord::new(x, y, direction),
            direction_steps,
        }
    }

    fn coord(&self) -> Coord {
        self.directed_coord.coord
    }
}

struct Entry {
    cost: u32,
    state: State,
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.cost.eq(&other.cost)
    }
}
impl Eq for Entry {}
impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cost.cmp(&other.cost).reverse()
    }
}

fn parse_input(input: &str) -> VecTable<Coord, u32, CoordIndexer> {
    let mut width = None;
    let data = input
        .lines()
        .flat_map(|line| {
            if width.is_none() {
                width = Some(line.len());
            } else {
                debug_assert_eq!(width, Some(line.len()));
            }
            line.chars().map(|c| c.to_digit(10).unwrap())
        })
        .collect::<Vec<_>>();
    let width = width.unwrap();
    let height = data.len() / width;
    let indexer = CoordIndexer::new(width, height);
    VecTable::from_vec(data, indexer)
}

fn solve(input: &str, ultra: bool) -> Option<u32> {
    let grid = parse_input(input);

    let coord_indexer = *grid.indexer();
    let width = coord_indexer.width;
    let height = coord_indexer.height;

    let mut min_heap = BinaryHeap::<Entry>::new();
    min_heap.push(Entry {
        cost: 0,
        state: State::new(0, 0, Direction::Down, 0),
    });
    min_heap.push(Entry {
        cost: 0,
        state: State::new(0, 0, Direction::Right, 0),
    });

    let mut best_costs: VecMap<
        DirectedCoord,
        VecMap<usize, u32, LinearIndexer>,
        DirectedCoordIndexer,
    > = VecMap::new(DirectedCoordIndexer::new(width, height));

    while let Some(entry) = min_heap.pop() {
        let Entry { cost, state } = entry;
        let State {
            directed_coord:
                DirectedCoord {
                    coord: Coord { x, y },
                    direction,
                },
            direction_steps,
        } = state;

        if x == width - 1 && y == height - 1 && (!ultra || direction_steps >= 4) {
            // Found the destination
            return Some(cost);
        }

        // Check if we already found a better path to this state, and if not, update the best cost
        match best_costs
            .entry(&state.directed_coord)
            .get_or_insert_with(|| VecMap::new(LinearIndexer::new(if ultra { 10 } else { 4 })))
            .entry(&state.direction_steps)
        {
            Some(best_cost) if *best_cost <= cost => {
                // Already found a better path to this state
                continue;
            }
            entry => {
                *entry = Some(cost);
            }
        }

        for next_direction in [
            Direction::Up,
            Direction::Right,
            Direction::Down,
            Direction::Left,
        ] {
            if next_direction == direction.opposite() {
                // Can't reverse direction
                continue;
            }

            let next_direction_steps = if next_direction == direction {
                direction_steps + 1
            } else {
                1
            };

            if !ultra {
                if next_direction_steps > 3 {
                    // Can't go in the same direction for more than 3 steps
                    continue;
                }
            } else {
                if next_direction_steps > 10 {
                    // Can't go in the same direction for more than 10 steps
                    continue;
                }
                if direction_steps < 4 && next_direction != direction {
                    // Can't change direction before 3 steps
                    continue;
                }
            }

            let next_state = match next_direction {
                Direction::Up if y > 0 => {
                    Some(State::new(x, y - 1, next_direction, next_direction_steps))
                }
                Direction::Right if x + 1 < width => {
                    Some(State::new(x + 1, y, next_direction, next_direction_steps))
                }
                Direction::Down if y + 1 < height => {
                    Some(State::new(x, y + 1, next_direction, next_direction_steps))
                }
                Direction::Left if x > 0 => {
                    Some(State::new(x - 1, y, next_direction, next_direction_steps))
                }
                _ => None,
            };

            if let Some(next_state) = next_state {
                let next_cost = cost + grid.get(&next_state.coord());
                min_heap.push(Entry {
                    cost: next_cost,
                    state: next_state,
                });
            }
        }
    }

    None
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
        assert_eq!(result, Some(102));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(94));
    }
}
