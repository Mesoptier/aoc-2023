use advent_of_code::util::coord::Direction;
use advent_of_code::util::{shortest_path, Indexer, VecTable};

use advent_of_code::util::shortest_path::Problem;

advent_of_code::solution!(17);

type CoordT = u32;
type Coord = advent_of_code::util::coord::Coord<CoordT>;
type CoordIndexer = advent_of_code::util::coord::CoordIndexer<CoordT>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn orthogonal(&self) -> Axis {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }

    fn directions(&self) -> [Direction; 2] {
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

struct ClumsyCrucibleProblem {
    grid: VecTable<Coord, u32, CoordIndexer>,
    width: CoordT,
    height: CoordT,
    source_coord: Coord,
    target_coord: Coord,
    min_steps: CoordT,
    max_steps: CoordT,
}

impl Problem for ClumsyCrucibleProblem {
    type State = State;
    type Cost = u32;

    fn sources(&self) -> impl IntoIterator<Item = Self::State> {
        [Axis::Horizontal, Axis::Vertical].map(move |axis| State {
            coord: self.source_coord,
            axis,
        })
    }

    fn is_target(&self, state: &Self::State) -> bool {
        state.coord == self.target_coord
    }

    #[inline]
    fn neighbors(&self, state: &Self::State, mut callback: impl FnMut(Self::State, Self::Cost)) {
        let State {
            coord: Coord { x, y },
            axis,
        } = *state;

        for direction in axis.directions() {
            let steps_to_edge = match direction {
                Direction::Up => y,
                Direction::Right => self.width - x - 1,
                Direction::Down => self.height - y - 1,
                Direction::Left => x,
            };
            if steps_to_edge < self.min_steps {
                // Not enough space to move in this direction
                continue;
            }

            let (dx, dy) = match direction {
                Direction::Up => (0, (-1i32) as u32),
                Direction::Right => (1, 0),
                Direction::Down => (0, 1),
                Direction::Left => ((-1i32) as u32, 0),
            };

            let mut next_cost = 0;
            let mut x = x;
            let mut y = y;

            for _ in 1..self.min_steps {
                x = x.wrapping_add(dx);
                y = y.wrapping_add(dy);

                let next_coord = Coord::new(x, y);
                next_cost += self.grid.get(&next_coord);
            }

            for _ in self.min_steps..=self.max_steps.min(steps_to_edge) {
                x = x.wrapping_add(dx);
                y = y.wrapping_add(dy);

                let next_coord = Coord::new(x, y);
                next_cost += self.grid.get(&next_coord);

                let next_state = State {
                    coord: next_coord,
                    axis: axis.orthogonal(),
                };

                callback(next_state, next_cost);
            }
        }
    }

    fn heuristic(&self, state: &Self::State) -> Self::Cost {
        // Manhattan distance to the target coord
        let State { coord, .. } = state;
        self.target_coord.x - coord.x + self.target_coord.y - coord.y
    }

    fn cost_to_index(cost: Self::Cost) -> usize {
        cost as usize
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

    let problem = ClumsyCrucibleProblem {
        grid,
        width,
        height,
        source_coord,
        target_coord,
        min_steps,
        max_steps,
    };
    shortest_path::a_star(problem, StateIndexer::new(width, height))
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
