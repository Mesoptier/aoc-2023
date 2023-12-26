use nom::branch::alt;
use nom::character::complete::{char, line_ending};
use nom::combinator::value;
use nom::multi::{many1, separated_list1};
use nom::IResult;

advent_of_code::solution!(14);

#[derive(Copy, Clone, Eq, PartialEq)]
enum Tile {
    RoundedRock,
    CubeShapedRock,
    Empty,
}

fn parse_input(input: &str) -> IResult<&str, Vec<Vec<Tile>>> {
    separated_list1(
        line_ending,
        many1(alt((
            value(Tile::RoundedRock, char('O')),
            value(Tile::CubeShapedRock, char('#')),
            value(Tile::Empty, char('.')),
        ))),
    )(input)
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, mut map) = parse_input(input).unwrap();

    let mut total_load = 0;

    for y in 0..map.len() {
        for x in 0..map[y].len() {
            if map[y][x] == Tile::RoundedRock {
                map[y][x] = Tile::Empty;

                let mut ny = y;
                while ny > 0 && map[ny - 1][x] == Tile::Empty {
                    ny -= 1;
                }

                map[ny][x] = Tile::RoundedRock;

                let load = (map.len() - ny) as u32;
                total_load += load;
            }
        }
    }

    Some(total_load)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(136));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
