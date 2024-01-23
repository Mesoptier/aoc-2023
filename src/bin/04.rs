use nom::bytes::complete::tag;
use nom::character::complete::{digit1, space1};
use nom::combinator::map_res;
use nom::multi::separated_list0;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::IResult;

advent_of_code::solution!(4);

fn parse_line(input: &str) -> IResult<&str, (Vec<u32>, Vec<u32>)> {
    preceded(
        tuple((tag("Card"), space1, digit1, tag(":"), space1)),
        separated_pair(
            parse_number_list,
            tuple((space1, tag("|"), space1)),
            parse_number_list,
        ),
    )(input)
}

fn parse_number_list(input: &str) -> IResult<&str, Vec<u32>> {
    separated_list0(space1, map_res(digit1, str::parse))(input)
}

fn iter_wins(input: &str) -> impl Iterator<Item = u32> + '_ {
    input.lines().map(|line| {
        let (_, (winning_numbers, numbers)) = parse_line(line).unwrap();

        let mut winning_mask = [false; 100];
        for number in winning_numbers {
            winning_mask[number as usize] = true;
        }

        let mut result = 0u32;
        for number in numbers {
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
