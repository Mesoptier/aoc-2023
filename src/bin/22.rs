use std::cmp::Ordering;
use std::collections::VecDeque;

use nom::character::complete::{char, digit1, line_ending};
use nom::combinator::map_res;
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;

use advent_of_code::util::{Indexer, LinearIndexer, VecSet, VecTable};

advent_of_code::solution!(22);

type CoordT = u32;
type Coord = [CoordT; 3];
type Brick = (Coord, Coord);
type BrickIndex = u32;

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

fn get_floor_dimensions(bricks: &[Brick]) -> [CoordT; 2] {
    let [max_x, max_y] = bricks
        .iter()
        .fold([0, 0], |[max_x, max_y], ([_, _, _], [x_hi, y_hi, _])| {
            [max_x.max(*x_hi), max_y.max(*y_hi)]
        });
    [max_x + 1, max_y + 1]
}

type AdjacencyList = VecTable<BrickIndex, Vec<BrickIndex>, LinearIndexer<BrickIndex>>;

/// Returns two adjacency lists:
/// 1. Brick -> bricks supporting it
/// 2. Brick -> bricks supported by it
fn build_supporting_graph(input: &str) -> (AdjacencyList, AdjacencyList) {
    let (_, mut bricks) = parse_input(input).unwrap();

    let floor_dimensions = get_floor_dimensions(&bricks);

    bricks.sort_unstable_by_key(|&([_, _, z_lo], _)| z_lo);

    // (z, brick_index) of top layer
    let mut top_layer = VecTable::<Coord, (CoordT, Option<BrickIndex>), CoordIndexer>::with_default(
        (0, None),
        CoordIndexer {
            dimensions: [floor_dimensions[0], floor_dimensions[1], 1],
        },
    );

    // Brick -> bricks supporting it
    let mut supported_by = AdjacencyList::new(LinearIndexer::new(bricks.len() as BrickIndex));
    // Brick -> bricks supported by it
    let mut supporting = AdjacencyList::new(LinearIndexer::new(bricks.len() as BrickIndex));

    for (brick_index, brick) in bricks.into_iter().enumerate() {
        let brick_index = brick_index as BrickIndex;
        let ([x_lo, y_lo, z_lo], [x_hi, y_hi, z_hi]) = brick;

        let mut next_z_lo = 0;
        let supported_by = supported_by.get_mut(&brick_index);

        for x in x_lo..=x_hi {
            for y in y_lo..=y_hi {
                let (z, supported_by_brick) = top_layer[[x, y, 0]];
                let z = z + 1;
                match z.cmp(&next_z_lo) {
                    Ordering::Less => {}
                    Ordering::Equal => {
                        // Brick is supported by another brick on the same layer
                        if let Some(supported_by_brick) = supported_by_brick {
                            supported_by.push(supported_by_brick);
                        }
                    }
                    Ordering::Greater => {
                        // Brick is supported by a brick on a higher layer
                        next_z_lo = z;
                        if let Some(supported_by_brick) = supported_by_brick {
                            supported_by.clear();
                            supported_by.push(supported_by_brick);
                        }
                    }
                }
            }
        }

        // Remove duplicates. Note that we don't need to sort the list first, because bricks are contiguous,
        // and so any duplicates will be contiguous as well.
        supported_by.dedup();

        // Update inverse adjacency list
        for supported_by_brick in supported_by.iter() {
            supporting.get_mut(supported_by_brick).push(brick_index);
        }

        // Update top layer
        let next_z_hi = next_z_lo + (z_hi - z_lo);
        for x in x_lo..=x_hi {
            for y in y_lo..=y_hi {
                top_layer[[x, y, 0]] = (next_z_hi, Some(brick_index));
            }
        }
    }

    (supported_by, supporting)
}

pub fn part_one(input: &str) -> Option<usize> {
    let (supported_by, supporting) = build_supporting_graph(input);
    supporting
        .values()
        .filter(|supported_bricks| {
            supported_bricks
                .iter()
                .all(|brick_index| supported_by[*brick_index].len() > 1)
        })
        .count()
        .into()
}

pub fn part_two(input: &str) -> Option<usize> {
    let (supported_by, supporting) = build_supporting_graph(input);

    let num_bricks = supported_by.indexer().len() as BrickIndex;
    (0..num_bricks)
        .map(|brick_index| {
            let mut queue = VecDeque::new();
            queue.push_back(brick_index);

            let mut removed_count = 0;
            let mut removed = VecSet::new(LinearIndexer::new(num_bricks));

            while let Some(brick_index) = queue.pop_front() {
                removed.insert(brick_index);
                removed_count += 1;

                for supported_brick in supporting[brick_index].iter() {
                    // If all bricks supporting this brick have been removed, add it to the queue to be removed
                    if supported_by[*supported_brick]
                        .iter()
                        .all(|brick_index| removed.contains(brick_index))
                    {
                        queue.push_back(*supported_brick);
                    }
                }
            }

            removed_count - 1
        })
        .sum::<usize>()
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
