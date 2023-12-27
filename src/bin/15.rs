use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, digit1};
use nom::combinator::{map, map_res};
use nom::sequence::{preceded, tuple};
use nom::IResult;

advent_of_code::solution!(15);

fn hash(s: &str) -> u8 {
    s.chars().fold(0, |acc, c| {
        if c.is_ascii_whitespace() {
            acc
        } else {
            acc.wrapping_add(c as u8).wrapping_mul(17)
        }
    })
}

pub fn part_one(input: &str) -> Option<u32> {
    input.split(',').map(|s| hash(s) as u32).sum1()
}

enum Operation {
    Remove,
    Insert(u32),
}

fn parse_step(s: &str) -> IResult<&str, (&str, Operation)> {
    tuple((
        alpha1,
        alt((
            map(tag("-"), |_| Operation::Remove),
            map(
                preceded(tag("="), map_res(digit1, str::parse)),
                Operation::Insert,
            ),
        )),
    ))(s)
}

pub fn part_two(input: &str) -> Option<u32> {
    let mut hash_map: Vec<Vec<(&str, u32)>> = vec![vec![]; 256];

    input
        .split(',')
        .map(|s| parse_step(s).unwrap().1)
        .for_each(|(label, operation)| {
            let index = hash_map[hash(label) as usize]
                .iter()
                .position(|entry| entry.0 == label);

            match (operation, index) {
                (Operation::Remove, Some(index)) => {
                    hash_map[hash(label) as usize].remove(index);
                }
                (Operation::Insert(value), None) => {
                    hash_map[hash(label) as usize].push((label, value));
                }
                (Operation::Insert(value), Some(index)) => {
                    hash_map[hash(label) as usize][index].1 = value;
                }
                _ => {}
            }
        });

    hash_map
        .iter()
        .enumerate()
        .map(|(box_idx, slots)| {
            slots
                .iter()
                .enumerate()
                .map(|(slot_idx, (_, value))| (box_idx + 1) as u32 * (slot_idx + 1) as u32 * value)
                .sum::<u32>()
        })
        .sum1()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(1320));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(145));
    }
}
