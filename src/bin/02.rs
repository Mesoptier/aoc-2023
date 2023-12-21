use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending};
use nom::combinator::map_res;
use nom::multi::separated_list0;
use nom::sequence::{delimited, preceded, separated_pair};
use nom::IResult;

advent_of_code::solution!(2);

fn parse_input(input: &str) -> IResult<&str, Vec<Vec<(u32, u32, u32)>>> {
    separated_list0(
        line_ending,
        preceded(
            delimited(tag("Game "), digit1, tag(": ")),
            separated_list0(tag("; "), parse_set),
        ),
    )(input)
}

fn parse_set(input: &str) -> IResult<&str, (u32, u32, u32)> {
    let (input, values) = separated_list0(
        tag(", "),
        separated_pair(
            map_res(digit1, str::parse::<u32>),
            tag(" "),
            alt((tag("red"), tag("green"), tag("blue"))),
        ),
    )(input)?;

    let mut red = 0;
    let mut green = 0;
    let mut blue = 0;

    for (value, color) in values {
        match color {
            "red" => red += value,
            "green" => green += value,
            "blue" => blue += value,
            _ => unreachable!(),
        }
    }

    Ok((input, (red, green, blue)))
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, games) = parse_input(input).ok()?;

    let max_red = 12;
    let max_green = 13;
    let max_blue = 14;

    games
        .iter()
        .enumerate()
        .filter(|(_, game)| {
            game.iter().all(|(red, green, blue)| {
                *red <= max_red && *green <= max_green && *blue <= max_blue
            })
        })
        .map(|(index, _)| index as u32 + 1)
        .sum::<u32>()
        .into()
}

pub fn part_two(input: &str) -> Option<u32> {
    let (_, games) = parse_input(input).ok()?;

    games
        .iter()
        .map(|game| {
            let (max_red, max_green, max_blue) = game.iter().fold(
                (0, 0, 0),
                |(max_red, max_green, max_blue), (red, green, blue)| {
                    (
                        max_red.max(*red),
                        max_green.max(*green),
                        max_blue.max(*blue),
                    )
                },
            );

            max_red * max_green * max_blue
        })
        .sum::<u32>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(8));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(2286));
    }
}
