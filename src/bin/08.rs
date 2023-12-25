use std::collections::HashMap;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, line_ending};
use nom::combinator::value;
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;
use num::integer::lcm;

advent_of_code::solution!(8);

#[derive(Debug, Copy, Clone)]
enum Instruction {
    Left,
    Right,
}

fn parse_input(input: &str) -> IResult<&str, (Vec<Instruction>, Vec<(&str, (&str, &str))>)> {
    separated_pair(
        many1(alt((
            value(Instruction::Left, tag("L")),
            value(Instruction::Right, tag("R")),
        ))),
        many1(line_ending),
        separated_list1(
            line_ending,
            separated_pair(
                alphanumeric1,
                tag(" = "),
                delimited(
                    tag("("),
                    separated_pair(alphanumeric1, tag(", "), alphanumeric1),
                    tag(")"),
                ),
            ),
        ),
    )(input)
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, (instructions, map)) = parse_input(input).ok()?;

    let map = HashMap::<&str, (&str, &str)>::from_iter(map);
    let mut node = "AAA";
    let mut steps = 0;

    for instruction in instructions.iter().cycle() {
        node = match (instruction, map.get(node).unwrap()) {
            (Instruction::Left, (left, _)) => left,
            (Instruction::Right, (_, right)) => right,
        };
        steps += 1;

        if node == "ZZZ" {
            return Some(steps);
        }
    }

    unreachable!()
}

pub fn part_two(input: &str) -> Option<usize> {
    let (_, (instructions, map)) = parse_input(input).ok()?;

    let map = HashMap::<&str, (&str, &str)>::from_iter(map);
    let nodes = map.keys().filter(|s| s.ends_with('A'));
    let steps = nodes.map(|node| {
        let mut node = node;
        let mut steps = 0;

        for instruction in instructions.iter().cycle() {
            node = match (instruction, map.get(node).unwrap()) {
                (Instruction::Left, (left, _)) => left,
                (Instruction::Right, (_, right)) => right,
            };
            steps += 1;

            if node.ends_with('Z') {
                return steps;
            }
        }

        unreachable!()
    });
    steps.reduce(lcm)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file_part(
            "examples", DAY, 1,
        ));
        assert_eq!(result, Some(2));
        let result = part_one(&advent_of_code::template::read_file_part(
            "examples", DAY, 2,
        ));
        assert_eq!(result, Some(6));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file_part(
            "examples", DAY, 3,
        ));
        assert_eq!(result, Some(6));
    }
}
