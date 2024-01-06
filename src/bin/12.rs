use std::iter;

use advent_of_code::util::{Indexer, VecTable};
use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending, one_of, space1};
use nom::combinator::{map_opt, map_res};
use nom::multi::{many1, separated_list1};
use nom::sequence::separated_pair;
use nom::IResult;

advent_of_code::solution!(12);

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
enum SpringCondition {
    Operational,
    Damaged,
    Unknown,
}

impl SpringCondition {
    fn from_char(c: char) -> Option<Self> {
        match c {
            '.' => Some(Self::Operational),
            '#' => Some(Self::Damaged),
            '?' => Some(Self::Unknown),
            _ => None,
        }
    }
}

fn parse_input(input: &str) -> IResult<&str, Vec<(Vec<SpringCondition>, Vec<usize>)>> {
    separated_list1(
        line_ending,
        separated_pair(
            many1(map_opt(one_of(".#?"), SpringCondition::from_char)),
            space1,
            separated_list1(tag(","), map_res(digit1, str::parse)),
        ),
    )(input)
}

struct CacheIndexer {
    width: usize,
    height: usize,
}

impl Indexer<(usize, usize)> for CacheIndexer {
    fn len(&self) -> usize {
        self.width * self.height
    }

    fn index_for(&self, (x, y): &(usize, usize)) -> usize {
        y * self.width + x
    }
}

fn count_arrangements(row: &[SpringCondition], damaged_groups: &[usize]) -> usize {
    let mut cache = VecTable::with_default(
        usize::MAX,
        CacheIndexer {
            width: row.len() + 1,
            height: damaged_groups.len() + 1,
        },
    );

    for j in (0..=damaged_groups.len()).rev() {
        // Number of (potentially) damaged springs at the beginning of the row.
        let mut damaged_prefix = 0;

        for i in (0..=row.len()).rev() {
            let row = &row[i..];
            let damaged_groups = &damaged_groups[j..];

            if row.is_empty() {
                cache.insert(&(i, j), if damaged_groups.is_empty() { 1 } else { 0 });
                continue;
            }

            // Maintain the number of damaged springs at the beginning of the row.
            match row[0] {
                SpringCondition::Damaged | SpringCondition::Unknown => damaged_prefix += 1,
                SpringCondition::Operational => damaged_prefix = 0,
            }

            if damaged_groups.is_empty() {
                if row[0] == SpringCondition::Damaged {
                    cache.insert(&(i, j), 0);
                    continue;
                }
                cache.insert(&(i, j), *cache.get(&(i + 1, j)));
                continue;
            }

            if damaged_groups[0] > row.len() {
                // Not enough space for the damaged group.
                cache.insert(&(i, j), 0);
                continue;
            }

            let mut num_arrangements = 0;

            if damaged_prefix >= damaged_groups[0] {
                // A damaged group could start here.

                if row.len() == damaged_groups[0] {
                    // Damaged group spans the entire row.
                    cache.insert(&(i, j), *cache.get(&(i + damaged_groups[0], j + 1)));
                    continue;
                }

                if row[damaged_groups[0]] != SpringCondition::Damaged {
                    // Damaged group is followed by at least one operational spring.
                    num_arrangements += cache.get(&(i + damaged_groups[0] + 1, j + 1))
                }
            } else if row[0] == SpringCondition::Damaged {
                // First damaged spring must be part of the first damaged group.
                cache.insert(&(i, j), 0);
                continue;
            }

            if row[0] != SpringCondition::Damaged {
                num_arrangements += cache.get(&(i + 1, j));
            }

            cache.insert(&(i, j), num_arrangements);
        }
    }

    *cache.get(&(0, 0))
}

pub fn part_one(input: &str) -> Option<usize> {
    let (_, records) = parse_input(input).unwrap();

    records
        .into_iter()
        .map(|(row, damaged_groups)| count_arrangements(&row, &damaged_groups))
        .sum1()
}

pub fn part_two(input: &str) -> Option<usize> {
    let (_, records) = parse_input(input).unwrap();

    records
        .into_iter()
        .map(|(row, damaged_groups)| {
            (
                iter::repeat(row)
                    .take(5)
                    .intersperse(vec![SpringCondition::Unknown])
                    .flatten()
                    .collect_vec(),
                iter::repeat(damaged_groups).take(5).flatten().collect_vec(),
            )
        })
        .map(|(row, damaged_groups)| count_arrangements(&row, &damaged_groups))
        .sum1()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(21));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(525152));
    }
}
