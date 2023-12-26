use itertools::Itertools;
use nom::branch::alt;
use nom::character::complete::{char, line_ending};
use nom::combinator::value;
use nom::multi::{many1, separated_list1};
use nom::IResult;
use std::iter;
advent_of_code::solution!(13);

fn parse_input(input: &str) -> IResult<&str, Vec<Vec<Vec<bool>>>> {
    separated_list1(
        many1(line_ending),
        separated_list1(
            line_ending,
            many1(alt((value(false, char('.')), value(true, char('#'))))),
        ),
    )(input)
}

fn transpose_map(map: &Vec<Vec<bool>>) -> Vec<Vec<bool>> {
    let mut result = vec![vec![false; map.len()]; map[0].len()];
    for (x, row) in map.iter().enumerate() {
        for (y, &value) in row.iter().enumerate() {
            result[y][x] = value;
        }
    }
    result
}

fn find_horizontal_reflection_line(map: &Vec<Vec<bool>>, target_smudges: usize) -> Option<usize> {
    (1..map.len()).find(|&num_rows_above| {
        let num_rows_below = map.len() - num_rows_above;
        let max_offset = usize::min(num_rows_above - 1, num_rows_below - 1);

        let mut smudges = 0;

        for offset in 0..=max_offset {
            let row_above = &map[num_rows_above - offset - 1];
            let row_below = &map[num_rows_above + offset];

            iter::zip(row_above, row_below)
                .filter(|(&a, &b)| a != b)
                .for_each(|_| smudges += 1);

            if smudges > target_smudges {
                return false;
            }
        }

        smudges == target_smudges
    })
}

fn solve(input: &str, smudges: usize) -> Option<usize> {
    let (_, maps) = parse_input(input).unwrap();

    maps.into_iter()
        .map(|map| {
            if let Some(line) = find_horizontal_reflection_line(&map, smudges) {
                line * 100
            } else if let Some(line) =
                find_horizontal_reflection_line(&transpose_map(&map), smudges)
            {
                line
            } else {
                unreachable!();
            }
        })
        .sum1()
}

pub fn part_one(input: &str) -> Option<usize> {
    solve(input, 0)
}

pub fn part_two(input: &str) -> Option<usize> {
    solve(input, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(405));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(400));
    }
}
