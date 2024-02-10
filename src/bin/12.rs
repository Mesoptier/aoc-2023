use std::iter;

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

fn count_arrangements(row: &[SpringCondition], damaged_groups: &[usize]) -> usize {
    let mut cache_row = vec![0; row.len() + 1];
    let mut prev_cache_row = vec![0; row.len() + 1];

    // Initialize base case.
    cache_row[0] = 1;

    // Initialize first row.
    for i in 1..=row.len() {
        if row[i - 1] == SpringCondition::Damaged {
            break;
        }
        cache_row[i] = 1;
    }

    for &cur_damaged_group_len in damaged_groups {
        std::mem::swap(&mut cache_row, &mut prev_cache_row);

        // Initialize first column.
        cache_row[0] = 0;

        // Number of (potentially) damaged springs at the end of `row[..i]`.
        let mut damaged_suffix = 0;

        for i in 1..=row.len() {
            let cur_spring = row[i - 1];

            // Maintain the `damaged_suffix` count.
            match cur_spring {
                SpringCondition::Damaged | SpringCondition::Unknown => damaged_suffix += 1,
                SpringCondition::Operational => damaged_suffix = 0,
            }

            if cur_damaged_group_len > i {
                // Not enough space for the damaged group.
                cache_row[i] = 0;
                continue;
            }

            let mut num_arrangements = 0;

            if damaged_suffix >= cur_damaged_group_len {
                // A damaged group could end here.

                if cur_damaged_group_len == i {
                    // Damaged group spans the entire row up to this point.
                    cache_row[i] = prev_cache_row[0];
                    continue;
                }

                if row[i - cur_damaged_group_len - 1] != SpringCondition::Damaged {
                    // Damaged group is preceded by at least one operational spring.
                    num_arrangements += prev_cache_row[i - cur_damaged_group_len - 1];
                }
            } else if cur_spring == SpringCondition::Damaged {
                // Damaged spring must be part of a damaged group, but no damaged group ends here.
                cache_row[i] = 0;
                continue;
            }

            if cur_spring != SpringCondition::Damaged {
                num_arrangements += cache_row[i - 1];
            }

            cache_row[i] = num_arrangements;
        }
    }

    cache_row[row.len()]
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
