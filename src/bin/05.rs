use itertools::Itertools;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, digit1, line_ending, multispace1, space1};
use nom::combinator::{map, map_res};
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair, tuple};
use nom::IResult;
advent_of_code::solution!(5);

type Map = Vec<MapEntry>;

#[derive(Debug)]
struct MapEntry {
    destination_range_start: usize,
    source_range_start: usize,
    range_length: usize,
}

fn parse_input(input: &str) -> IResult<&str, (Vec<usize>, Vec<Map>)> {
    separated_pair(
        preceded(
            tag("seeds: "),
            separated_list1(space1, map_res(digit1, str::parse)),
        ),
        multispace1,
        separated_list1(multispace1, parse_map),
    )(input)
}

fn parse_map(input: &str) -> IResult<&str, Map> {
    let (input, _) = tuple((alpha1, tag("-to-"), alpha1, tag(" map:"), line_ending))(input)?;
    separated_list1(
        line_ending,
        map(
            tuple((
                map_res(digit1, str::parse),
                space1,
                map_res(digit1, str::parse),
                space1,
                map_res(digit1, str::parse),
            )),
            |(destination_range_start, _, source_range_start, _, range_length)| MapEntry {
                destination_range_start,
                source_range_start,
                range_length,
            },
        ),
    )(input)
}

pub fn part_one(input: &str) -> Option<usize> {
    let (_, (seeds, mut maps)) = parse_input(input).unwrap();

    for map in &mut maps {
        map.sort_unstable_by_key(|entry| entry.source_range_start);
    }

    seeds
        .into_iter()
        .map(|seed| {
            let mut current = seed;
            for map in &maps {
                for entry in map {
                    let MapEntry {
                        source_range_start: src_start,
                        range_length: len,
                        destination_range_start: dest_start,
                    } = *entry;

                    if src_start <= current && current < src_start + len {
                        current = dest_start + (current - src_start);
                        break;
                    }
                }
            }
            current
        })
        .min()
}

pub fn part_two(input: &str) -> Option<usize> {
    let (_, (seeds, mut maps)) = parse_input(input).unwrap();

    for map in &mut maps {
        map.sort_unstable_by_key(|entry| entry.source_range_start);
    }

    let mut current_ranges = seeds.into_iter().tuples::<(_, _)>().collect_vec();

    for map in maps {
        current_ranges.sort_unstable_by_key(|(start, _)| *start);

        let mut map_entry_index = 0;
        let mut new_ranges = Vec::new();

        for (mut cur_start, mut cur_len) in current_ranges {
            while map_entry_index < map.len() {
                let MapEntry {
                    destination_range_start: dest_start,
                    source_range_start: src_start,
                    range_length: len,
                } = map[map_entry_index];

                if cur_start < src_start {
                    if cur_start + cur_len <= src_start {
                        // Current range is entirely before the source range
                        new_ranges.push((cur_start, cur_len));
                        cur_start += cur_len;
                        cur_len = 0;
                    } else {
                        // First part of current range is before the source range
                        new_ranges.push((cur_start, src_start - cur_start));
                        cur_len -= src_start - cur_start;
                        cur_start = src_start;
                    }
                } else if cur_start < src_start + len {
                    if cur_start + cur_len <= src_start + len {
                        // Current range is entirely inside the source range
                        new_ranges.push((dest_start + (cur_start - src_start), cur_len));
                        cur_start += cur_len;
                        cur_len = 0;
                    } else {
                        // First part of current range is inside the source range
                        new_ranges.push((
                            dest_start + (cur_start - src_start),
                            src_start + len - cur_start,
                        ));
                        cur_len -= src_start + len - cur_start;
                        cur_start = src_start + len;
                    }
                }

                if cur_len == 0 {
                    break;
                }

                map_entry_index += 1;
            }

            if cur_len > 0 {
                // Current range is after all source ranges
                new_ranges.push((cur_start, cur_len));
            }
        }

        current_ranges = new_ranges;
    }

    current_ranges
        .iter()
        .map(|(range_start, _)| *range_start)
        .min()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(35));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(46));
    }
}
