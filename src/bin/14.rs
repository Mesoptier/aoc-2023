use std::collections::HashMap;
use std::hash::Hash;

use itertools::Itertools;

use advent_of_code::util::coord::{
    Coord, CoordIndexer, Direction, Down, FlippedCoordIndexer, Left, Right, Up,
};
use advent_of_code::util::{Indexer, VecTable};

advent_of_code::solution!(14);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Tile {
    RoundedRock,
    CubeShapedRock,
    Empty,
}

impl Tile {
    fn from_char(c: char) -> Option<Tile> {
        match c {
            '.' => Some(Tile::Empty),
            'O' => Some(Tile::RoundedRock),
            '#' => Some(Tile::CubeShapedRock),
            _ => None,
        }
    }
}

fn parse_input(input: &str) -> VecTable<Coord, Tile, CoordIndexer> {
    let mut width = None;
    let data = input
        .lines()
        .flat_map(|line| {
            if width.is_none() {
                width = Some(line.len());
            } else {
                debug_assert_eq!(width, Some(line.len()));
            }
            line.chars().map(|c| Tile::from_char(c).unwrap())
        })
        .collect_vec();
    let width = width.unwrap();
    let height = data.len() / width;
    let indexer = CoordIndexer::new(width, height);
    VecTable::from_vec(data, indexer)
}

fn slide_rounded_rocks<D>(
    grid: &mut VecTable<Coord, Tile, FlippedCoordIndexer<D>, &mut [Tile]>,
) -> usize
where
    FlippedCoordIndexer<D>: Indexer<Coord>,
{
    let width = grid.indexer().width();
    let height = grid.indexer().height();

    let mut total_load = 0;

    for x in 0..width {
        let mut slide_to_y = 0;
        for y in 0..height {
            match grid.get(&Coord { x, y }) {
                Tile::RoundedRock => {
                    grid.insert(&Coord { x, y }, Tile::Empty);
                    grid.insert(&Coord { x, y: slide_to_y }, Tile::RoundedRock);

                    let load = height - slide_to_y;
                    total_load += load;
                    slide_to_y += 1;
                }
                Tile::CubeShapedRock => {
                    slide_to_y = y + 1;
                }
                Tile::Empty => {}
            }
        }
    }

    total_load
}

fn flip_total_load(total_load: usize, height: usize, num_rounded_rocks: usize) -> usize {
    num_rounded_rocks * (height + 1) - total_load
}

#[derive(Clone, Eq, PartialEq)]
struct CacheKey {
    tiles: Vec<Tile>,
    direction: Direction,
    north_total_load: usize,
    west_total_load: usize,
}

impl Hash for CacheKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // self.tiles.hash(state);
        self.direction.hash(state);
        self.north_total_load.hash(state);
        self.west_total_load.hash(state);
    }
}

pub fn part_one(input: &str) -> Option<usize> {
    let mut grid = parse_input(input);
    let total_load =
        slide_rounded_rocks(&mut grid.view_mut(FlippedCoordIndexer::<Up>::new(*grid.indexer())));
    Some(total_load)
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut grid = parse_input(input);

    let num_rounded_rocks = grid
        .values()
        .filter(|&&tile| tile == Tile::RoundedRock)
        .count();

    let mut directions = [
        Direction::Up,
        Direction::Left,
        Direction::Down,
        Direction::Right,
    ]
    .into_iter()
    .cycle();

    let mut steps = 0;
    let mut cache = HashMap::<CacheKey, usize>::new();
    let mut total_loads = vec![];

    let mut north_total_load = 0;
    let mut west_total_load = 0;

    for direction in directions.by_ref() {
        match direction {
            Direction::Up => {
                let indexer = FlippedCoordIndexer::<Up>::new(*grid.indexer());
                north_total_load = slide_rounded_rocks(&mut grid.view_mut(indexer));
            }
            Direction::Left => {
                let indexer = FlippedCoordIndexer::<Left>::new(*grid.indexer());
                west_total_load = slide_rounded_rocks(&mut grid.view_mut(indexer));
            }
            Direction::Down => {
                let indexer = FlippedCoordIndexer::<Down>::new(*grid.indexer());
                let south_total_load = slide_rounded_rocks(&mut grid.view_mut(indexer));
                north_total_load =
                    flip_total_load(south_total_load, grid.indexer().height, num_rounded_rocks);
            }
            Direction::Right => {
                let indexer = FlippedCoordIndexer::<Right>::new(*grid.indexer());
                let east_total_load = slide_rounded_rocks(&mut grid.view_mut(indexer));
                west_total_load =
                    flip_total_load(east_total_load, grid.indexer().width, num_rounded_rocks);
            }
        }
        steps += 1;

        if let Some(prev_steps) = cache.insert(
            CacheKey {
                tiles: grid.clone().to_vec(),
                direction,
                north_total_load,
                west_total_load,
            },
            steps,
        ) {
            let cycle = steps - prev_steps;
            let steps_remaining = (4_000_000_000 - steps) % cycle;
            return Some(total_loads[total_loads.len() - cycle + steps_remaining]);
        }

        total_loads.push(north_total_load);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(136));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(64));
    }
}
