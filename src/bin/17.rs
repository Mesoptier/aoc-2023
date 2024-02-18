use bucket_queue::{BucketQueue, LastInFirstOutQueue};

use advent_of_code::util::coord::Direction;
use advent_of_code::util::shortest_path::{CostMap, OpenSet, Problem};
use advent_of_code::util::{shortest_path, Indexer, VecMap, VecSet, VecTable};

advent_of_code::solution!(17);

type CoordT = u16;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;

type CoordIndex = u16;
type Cost = u16;

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

#[derive(Clone, Copy, PartialEq, Eq)]
struct State {
    coord_index: CoordIndex,
    axis: Axis,
}

#[derive(Clone, Copy)]
struct StateIndexer {
    grid_len: usize,
}
impl StateIndexer {
    fn new(grid_len: usize) -> Self {
        Self { grid_len }
    }
}
impl Indexer<State> for StateIndexer {
    fn len(&self) -> usize {
        self.grid_len * 2
    }

    fn index_for(&self, key: &State) -> usize {
        let coord_index = key.coord_index as usize;
        let axis = match key.axis {
            Axis::Horizontal => 0,
            Axis::Vertical => 1,
        };
        (coord_index << 1) + axis
    }
}

fn parse_input(input: &str) -> VecTable<Coord, Cost, CoordIndexer> {
    let mut width = None;
    let data = input
        .lines()
        .flat_map(|line| {
            if width.is_none() {
                width = Some(line.len());
            } else {
                debug_assert_eq!(width, Some(line.len()));
            }
            line.chars().map(|c| ((c as u8) - b'0') as Cost)
        })
        .collect::<Vec<_>>();
    let width = width.unwrap();
    let height = data.len() / width;
    let indexer = CoordIndexer::new(width as CoordT, height as CoordT);
    VecTable::from_vec(data, indexer)
}

struct ClumsyCrucibleProblem {
    grid: Box<[Cost]>,
    grid_width: CoordT,
    grid_height: CoordT,
    min_steps: CoordT,
    max_steps: CoordT,
}

impl ClumsyCrucibleProblem {
    fn source_index(&self) -> CoordIndex {
        (self.grid.len() - 1) as CoordIndex
    }
    fn target_index(&self) -> CoordIndex {
        0
    }
}

impl Problem for ClumsyCrucibleProblem {
    type State = State;
    type Cost = Cost;

    fn sources(&self) -> impl IntoIterator<Item = Self::State> {
        [Axis::Horizontal, Axis::Vertical].map(move |axis| {
            let coord_index = self.source_index();
            State { coord_index, axis }
        })
    }

    fn is_target(&self, state: &Self::State) -> bool {
        state.coord_index == self.target_index()
    }

    fn successors(
        &self,
        state: &Self::State,
    ) -> impl IntoIterator<Item = (Self::State, Self::Cost)> {
        let coord_index = state.coord_index;
        let axis = state.axis;

        axis.directions()
            .into_iter()
            .filter_map(move |direction| {
                let x = (coord_index % self.grid_width) as CoordT;
                let y = (coord_index / self.grid_width) as CoordT;
                let steps_to_edge = match direction {
                    Direction::Up => y,
                    Direction::Right => self.grid_width - x - 1,
                    Direction::Down => self.grid_height - y - 1,
                    Direction::Left => x,
                };
                if steps_to_edge < self.min_steps {
                    // Not enough space to move in this direction
                    return None;
                }
                Some((direction, steps_to_edge))
            })
            .flat_map(move |(direction, steps_to_edge)| {
                let coord_step = match direction {
                    Direction::Up => (0 as CoordT).wrapping_sub(self.grid_width),
                    Direction::Right => 1,
                    Direction::Down => self.grid_width,
                    Direction::Left => (0 as CoordT).wrapping_sub(1),
                };

                let mut next_coord_index = coord_index;
                let mut next_cost = 0;

                let num_pre_steps = self.min_steps - 1;
                let num_steps = self.max_steps.min(steps_to_edge) - num_pre_steps;

                (0..num_pre_steps).for_each(|_| {
                    next_cost += self.grid[next_coord_index as usize];
                    next_coord_index = next_coord_index.wrapping_add(coord_step);
                });

                (0..num_steps).map(move |_| {
                    next_cost += self.grid[next_coord_index as usize];
                    next_coord_index = next_coord_index.wrapping_add(coord_step);

                    let next_state = State {
                        coord_index: next_coord_index,
                        axis: axis.orthogonal(),
                    };
                    (next_state, next_cost as Cost)
                })
            })
    }

    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        // Manhattan distance to the target coord
        let x = state.coord_index % self.grid_width;
        let y = state.coord_index / self.grid_width;

        let target_x = self.target_index() % self.grid_width;
        let target_y = self.target_index() / self.grid_width;

        target_x.abs_diff(x) + target_y.abs_diff(y)
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

fn solve(input: &str, ultra: bool) -> Option<Cost> {
    let grid = parse_input(input);

    let min_steps = if ultra { 4 } else { 1 };
    let max_steps = if ultra { 10 } else { 3 };

    let coord_indexer = *grid.indexer();
    let width = coord_indexer.width;
    let height = coord_indexer.height;

    let grid = grid.to_vec().into_boxed_slice();
    let state_indexer = StateIndexer::new(grid.len());

    let problem = ClumsyCrucibleProblem {
        grid,
        grid_width: width,
        grid_height: height,
        min_steps,
        max_steps,
    };
    shortest_path::a_star(
        problem,
        MyOpenSet::new(state_indexer),
        MyCostMap::new(state_indexer),
    )
}

pub fn part_one(input: &str) -> Option<Cost> {
    solve(input, false)
}

pub fn part_two(input: &str) -> Option<Cost> {
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
