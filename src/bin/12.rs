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
    let (input, (springs, damaged_groups)) = separated_pair(
        many1(map_opt(one_of(".#?"), SpringCondition::from_char)),
        space1,
        separated_list1(tag(","), map_res(digit1, str::parse)),
    )(input)?;

    let mut springs = springs;
    let old_springs_len = springs.len();
    let springs_len = (springs.len() + 1) * repeat - 1;
    springs.reserve_exact(springs_len - old_springs_len);

    let mut damaged_groups = damaged_groups;
    let old_damaged_groups_len = damaged_groups.len();
    let damaged_groups_len = damaged_groups.len() * repeat;
    damaged_groups.reserve_exact(damaged_groups_len - old_damaged_groups_len);

    for _ in 1..repeat {
        springs.push(SpringCondition::Unknown);
        springs.extend_from_within(..old_springs_len);
        damaged_groups.extend_from_within(..old_damaged_groups_len);
    }

    Ok((input, (springs, damaged_groups)))
}

fn count_arrangements(springs: &[SpringCondition], damaged_groups: &[usize]) -> usize {
    let mut cache_row = vec![0; springs.len() + 1];
    let mut prev_cache_row = vec![0; springs.len() + 1];

    // Initialize base case.
    cache_row[0] = 1;

    // Initialize first row.
    for i in 1..=springs.len() {
        if springs[i - 1] == SpringCondition::Damaged {
            break;
        }
        cache_row[i] = 1;
    }

    for &damaged_group_len in damaged_groups {
        std::mem::swap(&mut cache_row, &mut prev_cache_row);

        // Initialize first column.
        cache_row[0] = 0;

        let mut damaged_suffix_start = 0;

        for (i, &spring) in springs.iter().enumerate() {
            let mut num_arrangements = 0;

            if spring != SpringCondition::Damaged {
                num_arrangements += cache_row[i];
            }

            if spring != SpringCondition::Operational {
                // Number of (potentially) damaged springs at the end of `row[..i]`.
                let damaged_suffix_len = i + 1 - damaged_suffix_start;

                if damaged_suffix_len >= damaged_group_len {
                    // A damaged group could end here.

                    if damaged_group_len == i + 1 {
                        // Damaged group spans the entire row up to this point.
                        num_arrangements += prev_cache_row[0];
                    } else if springs[i - damaged_group_len] != SpringCondition::Damaged {
                        // Damaged group is preceded by at least one operational spring.
                        num_arrangements += prev_cache_row[i - damaged_group_len];
                    }
                }
            } else {
                damaged_suffix_start = i + 1;
            }

            cache_row[i + 1] = num_arrangements;
        }
    }

    cache_row[springs.len()]
}

fn solve(input: &str, repeat: usize) -> Option<usize> {
    input
        .lines()
        .map(|line| parse_line(line, repeat).unwrap().1)
        .map(|(springs, damaged_groups)| count_arrangements(&springs, &damaged_groups))
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
