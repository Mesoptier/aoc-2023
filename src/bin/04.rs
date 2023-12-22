use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending, space1};
use nom::combinator::map_res;
use nom::multi::separated_list0;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::IResult;
use std::cmp::Ordering;

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

fn count_wins(mut winning_numbers: Vec<u32>, mut numbers: Vec<u32>) -> usize {
    winning_numbers.sort();
    numbers.sort();

    let mut result = 0;
    let mut i = 0;
    let mut j = 0;
    while i < winning_numbers.len() && j < numbers.len() {
        match winning_numbers[i].cmp(&numbers[j]) {
            Ordering::Less => i += 1,
            Ordering::Greater => j += 1,
            Ordering::Equal => {
                result += 1;
                i += 1;
                j += 1;
            }
        }
    }
    result
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, cards) = parse_input(input).ok()?;

    let result = cards
        .into_iter()
        .map(|(winning_numbers, numbers)| count_wins(winning_numbers, numbers))
        .filter(|&num_winners| num_winners > 0)
        .map(|num_winners| 2u32.pow(num_winners as u32 - 1))
        .sum();

    Some(result)
}

pub fn part_two(input: &str) -> Option<u32> {
    let (_, cards) = parse_input(input).ok()?;

    let original_card_wins = cards
        .into_iter()
        .map(|(winning_numbers, numbers)| count_wins(winning_numbers, numbers))
        .collect::<Vec<_>>();

    let mut num_cards = vec![1; original_card_wins.len()];
    for i in 0..original_card_wins.len() {
        for di in 0..original_card_wins[i] {
            num_cards[i + di + 1] += num_cards[i];
        }
    }

    Some(num_cards.iter().sum())
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
        assert_eq!(result, Some(30));
    }
}
