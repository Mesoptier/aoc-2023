use std::iter;

use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, one_of, space1};
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

fn parse_line(input: &str, repeat: usize) -> IResult<&str, (Vec<SpringCondition>, Vec<usize>)> {
    let (input, (row, damaged_groups)) = separated_pair(
        many1(map_opt(one_of(".#?"), SpringCondition::from_char)),
        space1,
        separated_list1(tag(","), map_res(digit1, str::parse)),
    )(input)?;

    if repeat == 1 {
        return Ok((input, (row, damaged_groups)));
    }

    let row_len = (row.len() + 1) * repeat - 1;
    let row = row
        .into_iter()
        .chain(iter::once(SpringCondition::Unknown))
        .cycle()
        .take(row_len)
        .collect_vec();

    let damaged_groups_len = damaged_groups.len() * repeat;
    let damaged_groups = damaged_groups
        .into_iter()
        .cycle()
        .take(damaged_groups_len)
        .collect_vec();

    Ok((input, (row, damaged_groups)))
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

    for &damaged_group_len in damaged_groups {
        std::mem::swap(&mut cache_row, &mut prev_cache_row);

        // Initialize first column.
        cache_row[0] = 0;

        let mut last_undamaged_i = 0;

        for i in 1..=row.len() {
            cache_row[i] = 0;

            if row[i - 1] != SpringCondition::Damaged {
                cache_row[i] += cache_row[i - 1];
            }

            if row[i - 1] != SpringCondition::Operational {
                // Number of (potentially) damaged springs at the end of `row[..i]`.
                let damaged_suffix_len = i - last_undamaged_i;

                if damaged_suffix_len >= damaged_group_len {
                    // A damaged group could end here.

                    if damaged_group_len == i {
                        // Damaged group spans the entire row up to this point.
                        cache_row[i] += prev_cache_row[0];
                    } else if row[i - damaged_group_len - 1] != SpringCondition::Damaged {
                        // Damaged group is preceded by at least one operational spring.
                        cache_row[i] += prev_cache_row[i - damaged_group_len - 1];
                    }
                }
            } else {
                last_undamaged_i = i;
            }
        }
    }

    cache_row[row.len()]
}

fn solve(input: &str, repeat: usize) -> Option<usize> {
    input
        .lines()
        .map(|line| parse_line(line, repeat).unwrap().1)
        .map(|(row, damaged_groups)| count_arrangements(&row, &damaged_groups))
        .sum1()
}

pub fn part_one(input: &str) -> Option<usize> {
    solve(input, 1)
}

pub fn part_two(input: &str) -> Option<usize> {
    solve(input, 5)
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
