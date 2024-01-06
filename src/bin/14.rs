use nom::branch::alt;
use nom::character::complete::{char, line_ending};
use nom::combinator::value;
use nom::multi::{many1, separated_list1};
use nom::IResult;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

advent_of_code::solution!(14);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
enum Tile {
    RoundedRock,
    CubeShapedRock,
    Empty,
}

trait TileGrid {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn get_unchecked(&self, x: usize, y: usize) -> Tile;
    fn set_unchecked(&mut self, x: usize, y: usize, tile: Tile);
}

#[derive(Clone, Eq, PartialEq)]
struct Grid {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
    total_load: usize,
}

impl Grid {
    fn new(tiles: Vec<Vec<Tile>>) -> Self {
        let width = tiles[0].len();
        let height = tiles.len();

        let mut total_load = 0;
        for y in 0..height {
            for x in 0..width {
                if tiles[y][x] == Tile::RoundedRock {
                    let load = height - y;
                    total_load += load;
                }
            }
        }

        Self {
            width,
            height,
            tiles: tiles.into_iter().flatten().collect(),
            total_load,
        }
    }

    fn total_load(&self) -> usize {
        self.total_load
    }
}

impl TileGrid for Grid {
    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn get_unchecked(&self, x: usize, y: usize) -> Tile {
        self.tiles[y * self.width + x]
    }

    fn set_unchecked(&mut self, x: usize, y: usize, tile: Tile) {
        match (self.get_unchecked(x, y), tile) {
            (Tile::RoundedRock, Tile::Empty) => {
                let load = self.height - y;
                self.total_load -= load;
            }
            (Tile::Empty, Tile::RoundedRock) => {
                let load = self.height - y;
                self.total_load += load;
            }
            _ => {}
        }

        self.tiles[y * self.width + x] = tile;
    }
}

impl Hash for Grid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.total_load.hash(state);
    }
}

/// A view into a grid that flips the x and y axis such that the `direction` side of the original grid is now the top.
struct FlippedGridView<'a, G> {
    grid: &'a mut G,
    direction: Direction,
}

impl<'a, G> FlippedGridView<'a, G> {
    fn new(grid: &'a mut G, direction: Direction) -> Self {
        Self { grid, direction }
    }
}

impl<'a, G: TileGrid> TileGrid for FlippedGridView<'a, G> {
    fn width(&self) -> usize {
        match self.direction {
            Direction::North | Direction::South => self.grid.width(),
            Direction::West | Direction::East => self.grid.height(),
        }
    }

    fn height(&self) -> usize {
        match self.direction {
            Direction::North | Direction::South => self.grid.height(),
            Direction::West | Direction::East => self.grid.width(),
        }
    }

    fn get_unchecked(&self, x: usize, y: usize) -> Tile {
        match self.direction {
            Direction::North => self.grid.get_unchecked(x, y),
            Direction::West => self.grid.get_unchecked(y, x),
            Direction::South => self.grid.get_unchecked(x, self.grid.height() - 1 - y),
            Direction::East => self.grid.get_unchecked(self.grid.width() - 1 - y, x),
        }
    }

    fn set_unchecked(&mut self, x: usize, y: usize, tile: Tile) {
        match self.direction {
            Direction::North => self.grid.set_unchecked(x, y, tile),
            Direction::West => self.grid.set_unchecked(y, x, tile),
            Direction::South => self.grid.set_unchecked(x, self.grid.height() - 1 - y, tile),
            Direction::East => self.grid.set_unchecked(self.grid.width() - 1 - y, x, tile),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum Direction {
    North,
    West,
    South,
    East,
}

fn parse_input(input: &str) -> IResult<&str, Vec<Vec<Tile>>> {
    separated_list1(
        line_ending,
        many1(alt((
            value(Tile::RoundedRock, char('O')),
            value(Tile::CubeShapedRock, char('#')),
            value(Tile::Empty, char('.')),
        ))),
    )(input)
}

fn slide_rounded_rocks<G: TileGrid>(grid: &mut G) {
    for x in 0..grid.width() {
        let mut slide_to_y = 0;
        for y in 0..grid.height() {
            match grid.get_unchecked(x, y) {
                Tile::RoundedRock => {
                    grid.set_unchecked(x, y, Tile::Empty);
                    grid.set_unchecked(x, slide_to_y, Tile::RoundedRock);
                    slide_to_y += 1;
                }
                Tile::CubeShapedRock => {
                    slide_to_y = y + 1;
                }
                Tile::Empty => {}
            }
        }
    }
}

pub fn part_one(input: &str) -> Option<usize> {
    let (_, map) = parse_input(input).unwrap();
    let mut grid = Grid::new(map);

    slide_rounded_rocks(&mut FlippedGridView::new(&mut grid, Direction::North));

    Some(grid.total_load())
}

pub fn part_two(input: &str) -> Option<usize> {
    let (_, map) = parse_input(input).unwrap();
    let mut grid = Grid::new(map);

    let mut directions = [
        Direction::North,
        Direction::West,
        Direction::South,
        Direction::East,
    ]
    .into_iter()
    .cycle();

    let mut steps = 0;
    let mut cache = HashMap::<(Grid, Direction), usize>::new();
    let mut total_loads = vec![];

    for direction in directions.by_ref() {
        slide_rounded_rocks(&mut FlippedGridView::new(&mut grid, direction));
        steps += 1;

        if let Some(prev_steps) = cache.insert((grid.clone(), direction), steps) {
            let cycle = steps - prev_steps;
            let steps_remaining = (4_000_000_000 - steps) % cycle;
            return Some(total_loads[total_loads.len() - cycle + steps_remaining]);
        }

        total_loads.push(grid.total_load());
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
