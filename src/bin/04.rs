use nom::bytes::complete::tag;
use nom::character::complete::{digit1, space0, space1};
use nom::combinator::{map_res, value};
use nom::sequence::{terminated, tuple};
use nom::IResult;

advent_of_code::solution!(4);

fn parse_prefix(input: &str) -> IResult<&str, ()> {
    value((), tuple((tag("Card"), space1, digit1, tag(":"), space1)))(input)
}

fn parse_number(input: &str) -> IResult<&str, u32> {
    terminated(map_res(digit1, str::parse), space0)(input)
}

fn parse_separator(input: &str) -> IResult<&str, ()> {
    value((), tuple((tag("|"), space1)))(input)
}

fn iter_wins(input: &str) -> impl Iterator<Item = u32> + '_ {
    input.lines().map(|line| {
        // Parse "Card #: " prefix
        let (mut line, _) = parse_prefix(line).unwrap();

        // Parse winning numbers, and store them in a mask
        let mut winning_mask = [false; 100];
        while let Ok((next_line, number)) = parse_number(line) {
            line = next_line;
            winning_mask[number as usize] = true;
        }

        // Parse "| " separator
        let (mut line, _) = parse_separator(line).unwrap();

        // Parse card numbers, and count the number of wins
        let mut result = 0u32;
        while let Ok((next_line, number)) = parse_number(line) {
            line = next_line;
            if winning_mask[number as usize] {
                result += 1;
            }
        }

        result
    })
}

pub fn part_one(input: &str) -> Option<u32> {
    iter_wins(input)
        .filter_map(|wins| {
            if wins == 0 {
                None
            } else {
                Some(2u32.pow(wins - 1))
            }
        })
        .sum::<u32>()
        .into()
}

pub fn part_two(input: &str) -> Option<u32> {
    let original_card_wins = iter_wins(input).collect::<Vec<_>>();

    let mut num_cards = vec![1; original_card_wins.len()];
    for i in 0..original_card_wins.len() {
        for di in 0..original_card_wins[i] {
            num_cards[i + (di as usize) + 1] += num_cards[i];
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
