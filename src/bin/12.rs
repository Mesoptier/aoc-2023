use std::collections::HashMap;
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

type Cache<'a> = HashMap<(&'a [SpringCondition], &'a [usize]), usize>;

fn count_arrangements<'a>(
    row: &'a [SpringCondition],
    damaged_groups: &'a [usize],
    cache: &mut Cache<'a>,
) -> usize {
    if let Some(&count) = cache.get(&(row, damaged_groups)) {
        return count;
    }

    if row.is_empty() {
        return if damaged_groups.is_empty() { 1 } else { 0 };
    }
    if damaged_groups.is_empty() {
        return if row
            .iter()
            .all(|&condition| condition != SpringCondition::Damaged)
        {
            1
        } else {
            0
        };
    }
    if row.len() < damaged_groups.iter().sum::<usize>() + damaged_groups.len() - 1 {
        return 0;
    }

    let mut sum = 0;
    let mut final_iteration = false;

    'outer: for (start, &condition) in row.iter().enumerate() {
        if final_iteration {
            break 'outer;
        }
        if damaged_groups[0] > row.len() - start {
            // Not enough space for the damaged group.
            break 'outer;
        }
        if condition == SpringCondition::Damaged {
            // First damaged spring must be part of the first damaged group, so
            // any iterations after this one would be invalid.
            final_iteration = true;
        }

        // Check if a damaged group of the correct size starts at `start`.
        let mut span = 0;
        while span < damaged_groups[0] {
            if row[start + span] == SpringCondition::Operational {
                continue 'outer;
            }
            span += 1;
        }

        // Damaged group must be followed by at least one operational spring (or the end of the row).
        if start + span < row.len() {
            if row[start + span] == SpringCondition::Damaged {
                continue 'outer;
            }
            span += 1;
        }

        sum += count_arrangements(&row[start + span..], &damaged_groups[1..], cache);
    }

    cache.insert((row, damaged_groups), sum);

    sum
}

pub fn part_one(input: &str) -> Option<usize> {
    let (_, records) = parse_input(input).unwrap();

    records
        .into_iter()
        .map(|(row, damaged_groups)| count_arrangements(&row, &damaged_groups, &mut HashMap::new()))
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
        .map(|(row, damaged_groups)| count_arrangements(&row, &damaged_groups, &mut HashMap::new()))
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
