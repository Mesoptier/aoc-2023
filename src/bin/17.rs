use std::cmp::Ordering;
use std::collections::BinaryHeap;

use advent_of_code::util::coord::Direction;
use advent_of_code::util::{VecMap, VecSet, VecTable};

advent_of_code::solution!(17);

type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type DirectedCoord = advent_of_code::util::coord::DirectedCoord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;
type DirectedCoordIndexer = advent_of_code::util::coord::DirectedCoordIndexer<CoordT>;

type State = DirectedCoord;

struct Entry {
    estimated_cost: u32,
    state: State,
}
impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.estimated_cost.eq(&other.estimated_cost)
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
        self.estimated_cost.cmp(&other.estimated_cost).reverse()
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
    let indexer = CoordIndexer::new(width as CoordT, height as CoordT);
    VecTable::from_vec(data, indexer)
}

fn solve(input: &str, ultra: bool) -> Option<u32> {
    let grid = parse_input(input);

    let min_steps = if ultra { 4 } else { 1 };
    let max_steps = if ultra { 10 } else { 3 };

    let coord_indexer = *grid.indexer();
    let width = coord_indexer.width;
    let height = coord_indexer.height;

    let destination = Coord::new(width - 1, height - 1);

    let heuristic = |state: &State| -> u32 {
        // Manhattan distance to the destination
        let State { coord, .. } = state;
        destination.x - coord.x + destination.y - coord.y
    };

    let mut min_heap = BinaryHeap::<Entry>::new();
    let mut best_costs: VecMap<DirectedCoord, u32, DirectedCoordIndexer> =
        VecMap::new(DirectedCoordIndexer::new(width, height));
    let mut visited = VecSet::new(DirectedCoordIndexer::new(width, height));

    let state = State::new(0, 0, Direction::Down);
    best_costs.insert(&state, 0);
    min_heap.push(Entry {
        estimated_cost: heuristic(&state),
        state,
    });

    let state = State::new(0, 0, Direction::Right);
    best_costs.insert(&state, 0);
    min_heap.push(Entry {
        estimated_cost: heuristic(&state),
        state,
    });

    while let Some(entry) = min_heap.pop() {
        let Entry { state, .. } = entry;
        let State {
            coord: Coord { x, y },
            direction,
        } = state;

        if !visited.insert(state) {
            // Already visited this state
            continue;
        }

        let cost = *best_costs.get(&state).unwrap();

        if x == destination.x && y == destination.y {
            // Found the destination
            return Some(cost);
        }

        let steps_to_edge = match direction {
            Direction::Up => y,
            Direction::Right => width - x - 1,
            Direction::Down => height - y - 1,
            Direction::Left => x,
        };
        if steps_to_edge < min_steps {
            // Not enough space to move in this direction
            continue;
        }

        let (dx, dy) = match direction {
            Direction::Up => (0, (-1i32) as u32),
            Direction::Right => (1, 0),
            Direction::Down => (0, 1),
            Direction::Left => ((-1i32) as u32, 0),
        };

        let mut next_cost = cost;
        let mut x = x;
        let mut y = y;

        for _ in 1..min_steps {
            x = x.wrapping_add(dx);
            y = y.wrapping_add(dy);

            let next_coord = Coord::new(x, y);
            next_cost += grid.get(&next_coord);
        }

        for _ in min_steps..=max_steps.min(steps_to_edge) {
            x = x.wrapping_add(dx);
            y = y.wrapping_add(dy);

            let next_coord = Coord::new(x, y);
            next_cost += grid.get(&next_coord);

            for next_direction in direction.orthogonal() {
                let next_state = State {
                    coord: next_coord,
                    direction: next_direction,
                };

                match best_costs.entry(&next_state) {
                    Some(best_cost) if *best_cost <= next_cost => {
                        // If we've already found a better path to this state, skip it
                        continue;
                    }
                    entry => {
                        // Otherwise, update the best cost and add the state to the queue
                        *entry = Some(next_cost);
                    }
                }

                min_heap.push(Entry {
                    estimated_cost: next_cost + heuristic(&next_state),
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
