use advent_of_code::util::{Indexer, VecTable};
use nom::character::complete::{char, digit1, line_ending};
use nom::combinator::map_res;
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;

advent_of_code::solution!(22);

type CoordT = u32;
type Coord = [CoordT; 3];
type Brick = (Coord, Coord);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CoordIndexer {
    dimensions: [CoordT; 3],
}

impl Indexer<Coord> for CoordIndexer {
    fn len(&self) -> usize {
        self.dimensions.iter().product::<CoordT>() as usize
    }

    fn index_for(&self, key: &Coord) -> usize {
        let [x, y, z] = key;
        let [width, height, _] = self.dimensions;
        (z * height * width + y * width + x) as usize
    }
}

type Grid = VecTable<Coord, bool, CoordIndexer>;

fn parse_input(input: &str) -> IResult<&str, Vec<Brick>> {
    separated_list1(
        line_ending,
        separated_pair(parse_coord, char('~'), parse_coord),
    )(input)
}

fn parse_coord(input: &str) -> IResult<&str, Coord> {
    let (input, x) = map_res(digit1, str::parse)(input)?;
    let (input, y) = preceded(char(','), map_res(digit1, str::parse))(input)?;
    let (input, z) = preceded(char(','), map_res(digit1, str::parse))(input)?;
    Ok((input, [x, y, z]))
}

fn build_empty_grid(bricks: &[Brick]) -> Grid {
    let [max_x, max_y, max_z] = bricks.iter().fold(
        [0, 0, 0],
        |[max_x, max_y, max_z], ([x, y, z], [x2, y2, z2])| {
            [
                max_x.max(*x).max(*x2),
                max_y.max(*y).max(*y2),
                max_z.max(*z).max(*z2),
            ]
        },
    );

    let indexer = CoordIndexer {
        dimensions: [max_x + 1, max_y + 1, max_z + 1],
    };

    let mut grid = VecTable::new(indexer);

    // Mark floor as blocked
    for x in 0..=max_x {
        for y in 0..=max_y {
            grid[[x, y, 0]] = true;
        }
    }

    grid
}

fn is_resting(grid: &Grid, brick: Brick) -> bool {
    let ([x_lo, y_lo, z_lo], [x_hi, y_hi, _]) = brick;

    for x in x_lo..=x_hi {
        for y in y_lo..=y_hi {
            if grid[[x, y, z_lo - 1]] {
                return true;
            }
        }
    }

    false
}

fn mark_grid(grid: &mut Grid, brick: Brick, state: bool) {
    let ([x_lo, y_lo, z_lo], [x_hi, y_hi, z_hi]) = brick;

    for x in x_lo..=x_hi {
        for y in y_lo..=y_hi {
            for z in z_lo..=z_hi {
                grid[[x, y, z]] = state;
            }
        }
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, bricks) = parse_input(input).unwrap();

    let mut grid = build_empty_grid(&bricks);

    let mut falling_bricks = bricks;
    let mut resting_bricks = Vec::new();

    falling_bricks.sort_unstable_by_key(|&([_, _, z_lo], _)| z_lo);

    while !falling_bricks.is_empty() {
        // Remove resting bricks from falling bricks
        falling_bricks.retain(|&brick| {
            if is_resting(&grid, brick) {
                resting_bricks.push(brick);
                mark_grid(&mut grid, brick, true);

                false
            } else {
                true
            }
        });

        // Move falling bricks down
        for (brick_lo, brick_hi) in &mut falling_bricks {
            let [x_lo, y_lo, z_lo] = *brick_lo;
            let [x_hi, y_hi, z_hi] = *brick_hi;
            *brick_lo = [x_lo, y_lo, z_lo - 1];
            *brick_hi = [x_hi, y_hi, z_hi - 1];
        }
    }

    // Count resting bricks that can be removed, without then causing other bricks to fall
    let result = resting_bricks
        .iter()
        .filter(|&&brick| {
            let mut grid = grid.clone();
            mark_grid(&mut grid, brick, false);

            resting_bricks.iter().all(|&brick| is_resting(&grid, brick))
        })
        .count();

    Some(result as u32)
}

pub fn part_two(input: &str) -> Option<u32> {
    let (_, bricks) = parse_input(input).unwrap();

    let mut grid = build_empty_grid(&bricks);

    let mut falling_bricks = bricks;
    let mut resting_bricks = Vec::new();

    falling_bricks.sort_unstable_by_key(|&([_, _, z_lo], _)| z_lo);

    while !falling_bricks.is_empty() {
        // Remove resting bricks from falling bricks
        falling_bricks.retain(|&brick| {
            if is_resting(&grid, brick) {
                resting_bricks.push(brick);
                mark_grid(&mut grid, brick, true);

                false
            } else {
                true
            }
        });

        // Move falling bricks down
        for (brick_lo, brick_hi) in &mut falling_bricks {
            let [x_lo, y_lo, z_lo] = *brick_lo;
            let [x_hi, y_hi, z_hi] = *brick_hi;
            *brick_lo = [x_lo, y_lo, z_lo - 1];
            *brick_hi = [x_hi, y_hi, z_hi - 1];
        }
    }

    (0..resting_bricks.len())
        .map(|i| {
            let mut grid = build_empty_grid(&resting_bricks);

            let mut falling_bricks = resting_bricks.clone();
            falling_bricks.remove(i);

            // Count non-resting bricks
            falling_bricks
                .into_iter()
                .filter(|&brick| {
                    if is_resting(&grid, brick) {
                        mark_grid(&mut grid, brick, true);

                        false
                    } else {
                        true
                    }
                })
                .count() as u32
        })
        .sum::<u32>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(5));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(7));
    }
}
