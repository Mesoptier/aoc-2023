use std::collections::HashMap;

use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, digit1, line_ending};
use nom::combinator::{map, map_res, value};
use nom::multi::{many1, separated_list1};
use nom::sequence::{delimited, preceded, separated_pair, tuple};
use nom::IResult;

advent_of_code::solution!(19);

#[derive(Debug)]
struct Workflow<'a> {
    rules: Vec<(Condition, Target<'a>)>,
    fallback: Target<'a>,
}

#[derive(Debug)]
enum Condition {
    Gt(usize, u32),
    Lt(usize, u32),
}

#[derive(Debug, Clone, Copy)]
enum Target<'a> {
    Workflow(&'a str),
    Accept,
    Reject,
}

type Part = [u32; 4];

fn parse_input(input: &str) -> IResult<&str, (Vec<(&str, Workflow)>, Vec<Part>)> {
    separated_pair(
        separated_list1(
            line_ending,
            tuple((alpha1, delimited(tag("{"), parse_workflow, tag("}")))),
        ),
        many1(line_ending),
        separated_list1(line_ending, delimited(tag("{"), parse_part, tag("}"))),
    )(input)
}

fn parse_workflow(input: &str) -> IResult<&str, Workflow> {
    let (input, rules) = separated_list1(
        tag(","),
        separated_pair(parse_condition, tag(":"), parse_target),
    )(input)?;
    let (input, fallback) = preceded(tag(","), parse_target)(input)?;
    Ok((input, Workflow { rules, fallback }))
}

fn parse_condition(input: &str) -> IResult<&str, Condition> {
    let (input, property) = alt((
        value(0, tag("x")),
        value(1, tag("m")),
        value(2, tag("a")),
        value(3, tag("s")),
    ))(input)?;
    alt((
        map(preceded(tag(">"), map_res(digit1, str::parse)), move |n| {
            Condition::Gt(property, n)
        }),
        map(preceded(tag("<"), map_res(digit1, str::parse)), move |n| {
            Condition::Lt(property, n)
        }),
    ))(input)
}

fn parse_target(input: &str) -> IResult<&str, Target> {
    alt((
        value(Target::Accept, tag("A")),
        value(Target::Reject, tag("R")),
        map(alpha1, Target::Workflow),
    ))(input)
}

fn parse_part(input: &str) -> IResult<&str, Part> {
    let (input, x) = preceded(tag("x="), map_res(digit1, str::parse))(input)?;
    let (input, m) = preceded(tag(",m="), map_res(digit1, str::parse))(input)?;
    let (input, a) = preceded(tag(",a="), map_res(digit1, str::parse))(input)?;
    let (input, s) = preceded(tag(",s="), map_res(digit1, str::parse))(input)?;
    Ok((input, [x, m, a, s]))
}

fn is_part_accepted(part: Part, workflows: &HashMap<&str, Workflow>) -> bool {
    let mut workflow = workflows.get("in").unwrap();
    loop {
        let mut target = None;

        for (condition, rule_target) in &workflow.rules {
            let matches_condition = match condition {
                Condition::Gt(property, value) => part[*property] > *value,
                Condition::Lt(property, value) => part[*property] < *value,
            };

            if matches_condition {
                target = Some(rule_target);
                break;
            }
        }

        let target = target.unwrap_or(&workflow.fallback);
        match target {
            Target::Workflow(label) => {
                workflow = workflows.get(label).unwrap();
            }
            Target::Accept => {
                return true;
            }
            Target::Reject => {
                return false;
            }
        }
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, (workflows, parts)) = parse_input(input).unwrap();
    let workflows = HashMap::<&str, Workflow>::from_iter(workflows);
    parts
        .into_iter()
        .filter(|part| is_part_accepted(*part, &workflows))
        .map(|part| part.iter().sum::<u32>())
        .sum1()
}

#[derive(Copy, Clone, Debug)]
struct Bound {
    gt: u32,
    lt: u32,
}

fn compute_accepted_combinations_for_bounds(bounds: [Bound; 4]) -> usize {
    bounds
        .iter()
        .map(|Bound { gt, lt }| ((lt - 1) - (gt + 1) + 1) as usize)
        .product::<usize>()
}

fn compute_accepted_combinations(
    workflows: &HashMap<&str, Workflow>,
    label: &str,
    mut bounds: [Bound; 4],
) -> usize {
    let mut result = 0;
    let workflow = workflows.get(label).unwrap();
    for (condition, target) in &workflow.rules {
        let mut rule_bounds = bounds;

        match condition {
            Condition::Gt(property, gt) => {
                if bounds[*property].lt - 1 < *gt + 1 {
                    // Rule cannot match, skip
                    continue;
                }
                rule_bounds[*property].gt = *gt;
                bounds[*property].lt = *gt + 1;
            }
            Condition::Lt(property, lt) => {
                if bounds[*property].gt + 1 > *lt - 1 {
                    // Rule cannot match, skip
                    continue;
                }
                rule_bounds[*property].lt = *lt;
                bounds[*property].gt = *lt - 1;
            }
        }

        result += match target {
            Target::Workflow(label) => compute_accepted_combinations(workflows, label, rule_bounds),
            Target::Accept => compute_accepted_combinations_for_bounds(rule_bounds),
            Target::Reject => 0,
        };
    }

    result += match workflow.fallback {
        Target::Workflow(label) => compute_accepted_combinations(workflows, label, bounds),
        Target::Accept => compute_accepted_combinations_for_bounds(bounds),
        Target::Reject => 0,
    };

    result
}

pub fn part_two(input: &str) -> Option<usize> {
    let (_, (workflows, _)) = parse_input(input).unwrap();
    let workflows = HashMap::<&str, Workflow>::from_iter(workflows);
    compute_accepted_combinations(&workflows, "in", [Bound { gt: 0, lt: 4001 }; 4]).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(19114));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(167409079868000));
    }
}
