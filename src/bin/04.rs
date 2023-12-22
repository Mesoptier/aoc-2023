use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending, space1};
use nom::combinator::map_res;
use nom::multi::separated_list0;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::IResult;
use std::collections::HashSet;

advent_of_code::solution!(4);

fn parse_input(input: &str) -> IResult<&str, Vec<(Vec<u32>, Vec<u32>)>> {
    separated_list0(
        line_ending,
        preceded(
            tuple((tag("Card"), space1, digit1, tag(":"), space1)),
            separated_pair(
                parse_number_list,
                tuple((space1, tag("|"), space1)),
                parse_number_list,
            ),
        ),
    )(input)
}

fn parse_number_list(input: &str) -> IResult<&str, Vec<u32>> {
    separated_list0(space1, map_res(digit1, str::parse))(input)
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, cards) = parse_input(input).ok()?;

    let mut result = 0;

    for (winning_numbers, numbers) in cards {
        let winning_numbers = HashSet::<u32>::from_iter(winning_numbers);
        let numbers = HashSet::<u32>::from_iter(numbers);

        let num_winners = winning_numbers.intersection(&numbers).count();
        if num_winners > 0 {
            result += 2u32.pow(num_winners as u32 - 1);
        }
    }

    Some(result)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(13));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
