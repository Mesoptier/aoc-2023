use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete::{digit1, line_ending, space1};
use nom::combinator::map_res;
use nom::multi::{fold_many1, separated_list1};
use nom::sequence::{preceded, tuple};
use nom::IResult;
advent_of_code::solution!(6);

fn parse_input_part_one(input: &str) -> IResult<&str, Vec<(usize, usize)>> {
    let (input, times) = preceded(
        tuple((tag("Time:"), space1)),
        separated_list1(space1, map_res(digit1, str::parse)),
    )(input)?;
    let (input, _) = line_ending(input)?;
    let (input, distances) = preceded(
        tuple((tag("Distance:"), space1)),
        separated_list1(space1, map_res(digit1, str::parse)),
    )(input)?;

    Ok((input, times.into_iter().zip(distances).collect()))
}

fn parse_badly_kerned_number(input: &str) -> IResult<&str, usize> {
    let (input, number_str) =
        fold_many1(preceded(space1, digit1), String::new, |mut acc, item| {
            acc.push_str(item);
            acc
        })(input)?;
    let number = number_str.parse().unwrap();
    Ok((input, number))
}

fn parse_input_part_two(input: &str) -> IResult<&str, (usize, usize)> {
    let (input, time) = preceded(tag("Time:"), parse_badly_kerned_number)(input)?;
    let (input, _) = line_ending(input)?;
    let (input, distance) = preceded(tag("Distance:"), parse_badly_kerned_number)(input)?;

    Ok((input, (time, distance)))
}

/// Counts the number of ways there are to beat the record distance.
fn solve_race(race_time: usize, record_distance: usize) -> usize {
    (0..=race_time)
        .map(|pressed_time| (race_time - pressed_time) * pressed_time)
        .filter(|&distance| distance > record_distance)
        .count()
}

pub fn part_one(input: &str) -> Option<usize> {
    let (_, races) = parse_input_part_one(input).unwrap();

    races
        .into_iter()
        .map(|(race_time, record_distance)| solve_race(race_time, record_distance))
        .product1()
}

pub fn part_two(input: &str) -> Option<usize> {
    let (_, (race_time, record_distance)) = parse_input_part_two(input).unwrap();
    Some(solve_race(race_time, record_distance))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(288));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(71503));
    }
}
