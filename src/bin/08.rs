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

fn solve(input: &str, part_two: bool) -> Option<usize> {
    let (_, (instructions, map)) = parse_input(input).unwrap();

    let (map, starting_nodes, target_node_mask) = {
        let node_to_index = map
            .iter()
            .enumerate()
            .map(|(i, (node, _))| (*node, i as u32))
            .collect::<HashMap<&str, u32>>();

        let mut starting_nodes = vec![];
        let mut target_node_mask = vec![false; map.len()];
        let map = map
            .iter()
            .map(|(node, (left, right))| {
                if match part_two {
                    false => *node == "AAA",
                    true => node.ends_with('A'),
                } {
                    starting_nodes.push(node_to_index[node]);
                }
                if match part_two {
                    false => *node == "ZZZ",
                    true => node.ends_with('Z'),
                } {
                    target_node_mask[node_to_index[node] as usize] = true;
                }

                let left = node_to_index[left];
                let right = node_to_index[right];
                (left, right)
            })
            .collect::<Vec<_>>();

        (map, starting_nodes, target_node_mask)
    };

    starting_nodes
        .into_iter()
        .map(|starting_node| {
            let mut node = starting_node;
            let mut steps = 0;

            for instruction in instructions.iter().cycle() {
                node = match (instruction, map[node as usize]) {
                    (Instruction::Left, (left, _)) => left,
                    (Instruction::Right, (_, right)) => right,
                };
                steps += 1;

                if target_node_mask[node as usize] {
                    return steps;
                }
            }

            unreachable!()
        })
        .reduce(lcm)
}

pub fn part_one(input: &str) -> Option<usize> {
    solve(input, false)
}

pub fn part_two(input: &str) -> Option<usize> {
    solve(input, true)
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
