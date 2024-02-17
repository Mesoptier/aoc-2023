use bucket_queue::{BucketQueue, LastInFirstOutQueue};

use advent_of_code::util::coord::{CoordStepper, Direction};
use advent_of_code::util::shortest_path::{CostMap, OpenSet, Problem};
use advent_of_code::util::{shortest_path, Indexer, VecMap, VecSet, VecTable};

advent_of_code::solution!(17);

type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;

type Cost = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    const fn orthogonal(&self) -> Axis {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }

    const fn directions(&self) -> [Direction; 2] {
        match self {
            Axis::Horizontal => [Direction::Left, Direction::Right],
            Axis::Vertical => [Direction::Up, Direction::Down],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct State {
    coord: Coord,
    axis: Axis,
}

#[derive(Clone, Copy)]
struct StateIndexer {
    coord_indexer: CoordIndexer,
}
impl StateIndexer {
    fn new(width: CoordT, height: CoordT) -> Self {
        Self {
            coord_indexer: CoordIndexer::new(width, height),
        }
    }
}
impl Indexer<State> for StateIndexer {
    fn len(&self) -> usize {
        self.coord_indexer.len() * 2
    }

    fn index_for(&self, key: &State) -> usize {
        let State { coord, axis } = key;
        let coord_index = self.coord_indexer.index_for(coord);
        let axis_index = match axis {
            Axis::Horizontal => 0,
            Axis::Vertical => 1,
        };
        coord_index * 2 + axis_index
    }
}

fn parse_input(input: &str) -> VecTable<Coord, u8, CoordIndexer> {
    let mut width = None;
    let data = input
        .lines()
        .flat_map(|line| {
            if width.is_none() {
                width = Some(line.len());
            } else {
                debug_assert_eq!(width, Some(line.len()));
            }
            line.chars().map(|c| (c as u8) - b'0')
        })
        .collect::<Vec<_>>();
    let width = width.unwrap();
    let height = data.len() / width;
    let indexer = CoordIndexer::new(width as CoordT, height as CoordT);
    VecTable::from_vec(data, indexer)
}

struct ClumsyCrucibleProblem {
    grid: VecTable<Coord, u8, CoordIndexer>,
    width: CoordT,
    height: CoordT,
    source_coord: Coord,
    target_coord: Coord,
    min_steps: CoordT,
    max_steps: CoordT,
}

impl Problem for ClumsyCrucibleProblem {
    type State = State;
    type Cost = Cost;

    fn sources(&self) -> impl IntoIterator<Item = Self::State> {
        [Axis::Horizontal, Axis::Vertical].map(move |axis| State {
            coord: self.source_coord,
            axis,
        })
    }

    fn is_target(&self, state: &Self::State) -> bool {
        state.coord == self.target_coord
    }

    fn successors(
        &self,
        state: &Self::State,
    ) -> impl IntoIterator<Item = (Self::State, Self::Cost)> {
        let State { coord, axis } = *state;

        axis.directions()
            .into_iter()
            .filter_map(move |direction| {
                let steps_to_edge = match direction {
                    Direction::Up => coord.y,
                    Direction::Right => self.width - coord.x - 1,
                    Direction::Down => self.height - coord.y - 1,
                    Direction::Left => coord.x,
                };
                if steps_to_edge < self.min_steps {
                    // Not enough space to move in this direction
                    return None;
                }

                Some((direction, steps_to_edge))
            })
            .flat_map(move |(direction, steps_to_edge)| {
                let coord_stepper = CoordStepper::<CoordT>::from_direction(direction);

                let mut next_coord = coord;
                let mut next_cost = 0;

                let num_pre_steps = self.min_steps - 1;
                let num_steps = self.max_steps.min(steps_to_edge) - num_pre_steps;

                (0..num_pre_steps).for_each(|_| {
                    next_coord = coord_stepper.step(next_coord);
                    next_cost += self.grid.get(&next_coord);
                });

                (0..num_steps).map(move |_| {
                    next_coord = coord_stepper.step(next_coord);
                    next_cost += self.grid.get(&next_coord);

                    let next_state = State {
                        coord: next_coord,
                        axis: axis.orthogonal(),
                    };

                    (next_state, next_cost as Cost)
                })
            })
    }

    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        // Manhattan distance to the target coord
        let State { coord, .. } = state;
        self.target_coord.x.abs_diff(coord.x) + self.target_coord.y.abs_diff(coord.y)
    }
}

struct MyOpenSet {
    queue: BucketQueue<Vec<State>>,
    visited: VecSet<State, StateIndexer>,
}

impl MyOpenSet {
    fn new(state_indexer: StateIndexer) -> Self {
        Self {
            queue: BucketQueue::new(),
            visited: VecSet::new(state_indexer),
        }
    }
}

impl OpenSet<State, Cost> for MyOpenSet {
    #[inline]
    fn insert(&mut self, state: State, cost: Cost) {
        self.queue.push(state, cost as usize)
    }

    #[inline]
    fn pop_min(&mut self) -> Option<State> {
        while let Some(state) = self.queue.pop_min() {
            if self.visited.insert(state) {
                return Some(state);
            }
        }
        None
    }
}

struct MyCostMap {
    map: VecMap<State, Cost, StateIndexer>,
}

impl MyCostMap {
    fn new(state_indexer: StateIndexer) -> Self {
        Self {
            map: VecMap::new(state_indexer),
        }
    }
}

impl CostMap<State, Cost> for MyCostMap {
    fn get(&self, state: &State) -> Option<Cost> {
        self.map.get(state).copied()
    }

    fn insert(&mut self, state: State, cost: Cost) -> bool {
        match self.map.entry(&state) {
            Some(prev_cost) if *prev_cost <= cost => false,
            entry => {
                *entry = Some(cost);
                true
            }
        }
    }
}

fn solve(input: &str, ultra: bool) -> Option<u32> {
    let grid = parse_input(input);

    let min_steps = if ultra { 4 } else { 1 };
    let max_steps = if ultra { 10 } else { 3 };

    let coord_indexer = *grid.indexer();
    let width = coord_indexer.width;
    let height = coord_indexer.height;

    let source_coord = Coord::new(0, 0);
    let target_coord = Coord::new(width - 1, height - 1);

    let state_indexer = StateIndexer::new(width, height);

    let problem = ClumsyCrucibleProblem {
        grid,
        width,
        height,
        source_coord,
        target_coord,
        min_steps,
        max_steps,
    };
    shortest_path::a_star(
        problem,
        MyOpenSet::new(state_indexer),
        MyCostMap::new(state_indexer),
    )
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
