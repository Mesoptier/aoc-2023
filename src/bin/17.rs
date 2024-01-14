use std::cmp::Ordering;
use std::collections::BinaryHeap;

use advent_of_code::util::coord::Direction;
use advent_of_code::util::{VecMap, VecTable};

advent_of_code::solution!(17);

type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type DirectedCoord = advent_of_code::util::coord::DirectedCoord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;
type DirectedCoordIndexer = advent_of_code::util::coord::DirectedCoordIndexer<CoordT>;

type State = DirectedCoord;

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

    let mut min_heap = BinaryHeap::<Entry>::new();
    min_heap.push(Entry {
        cost: 0,
        state: State::new(0, 0, Direction::Down),
    });
    min_heap.push(Entry {
        cost: 0,
        state: State::new(0, 0, Direction::Right),
    });

    let mut best_costs: VecMap<DirectedCoord, u32, DirectedCoordIndexer> =
        VecMap::new(DirectedCoordIndexer::new(width, height));

    while let Some(entry) = min_heap.pop() {
        let Entry { cost, state } = entry;
        let State {
            coord: Coord { x, y },
            direction,
        } = state;

        if x == width - 1 && y == height - 1 {
            // Found the destination
            return Some(cost);
        }

        // Check if we already found a better path to this state, and if not, update the best cost
        match best_costs.entry(&state) {
            Some(best_cost) if *best_cost <= cost => {
                // Already found a better path to this state
                continue;
            }
            entry => {
                *entry = Some(cost);
            }
        }

        let mut next_cost = cost;
        for steps in 1..=max_steps {
            let next_coord = match direction {
                Direction::Up if y >= steps => Coord::new(x, y - steps),
                Direction::Right if x + steps < width => Coord::new(x + steps, y),
                Direction::Down if y + steps < height => Coord::new(x, y + steps),
                Direction::Left if x >= steps => Coord::new(x - steps, y),
                _ => break,
            };
            next_cost += grid.get(&next_coord);

            if steps >= min_steps {
                for next_direction in direction.orthogonal() {
                    let next_state = State::new(next_coord.x, next_coord.y, next_direction);
                    min_heap.push(Entry {
                        cost: next_cost,
                        state: next_state,
                    });
                }
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
