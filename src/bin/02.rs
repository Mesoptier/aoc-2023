use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending};
use nom::combinator::{map_res, opt, value};
use nom::multi::{fold_many0, separated_list0};
use nom::sequence::{delimited, preceded, separated_pair};
use nom::IResult;

advent_of_code::solution!(2);

fn parse_input(input: &str) -> IResult<&str, Vec<Vec<[u32; 3]>>> {
    separated_list0(
        line_ending,
        preceded(
            delimited(tag("Game "), digit1, tag(": ")),
            separated_list0(tag("; "), parse_set),
        ),
    )(input)
}

fn parse_set(input: &str) -> IResult<&str, [u32; 3]> {
    fold_many0(
        preceded(
            opt(tag(", ")),
            separated_pair(
                map_res(digit1, str::parse::<u32>),
                tag(" "),
                alt((
                    value(0, tag("red")),
                    value(1, tag("green")),
                    value(2, tag("blue")),
                )),
            ),
        ),
        || [0; 3],
        |mut rgb, (value, idx)| {
            rgb[idx] += value;
            rgb
        },
    )(input)
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, games) = parse_input(input).unwrap();

    let max_red = 12;
    let max_green = 13;
    let max_blue = 14;

    games
        .iter()
        .enumerate()
        .filter(|(_, game)| {
            game.iter().all(|[red, green, blue]| {
                *red <= max_red && *green <= max_green && *blue <= max_blue
            })
        })
        .map(|(index, _)| index as u32 + 1)
        .sum::<u32>()
        .into()
}

pub fn part_two(input: &str) -> Option<u32> {
    let (_, games) = parse_input(input).unwrap();

    games
        .iter()
        .map(|game| {
            let (max_red, max_green, max_blue) = game.iter().fold(
                (0, 0, 0),
                |(max_red, max_green, max_blue), [red, green, blue]| {
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
