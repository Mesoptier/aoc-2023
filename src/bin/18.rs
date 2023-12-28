use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{char, digit1, hex_digit1, space1};
use nom::combinator::{all_consuming, map, map_res};
use nom::sequence::{delimited, terminated, tuple};
use nom::IResult;

advent_of_code::solution!(18);

#[derive(Debug, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn parse_entry(s: &str) -> IResult<&str, (Direction, usize, u32)> {
    all_consuming(tuple((
        terminated(
            alt((
                map(char('U'), |_| Direction::Up),
                map(char('D'), |_| Direction::Down),
                map(char('L'), |_| Direction::Left),
                map(char('R'), |_| Direction::Right),
            )),
            space1,
        ),
        terminated(map_res(digit1, str::parse), space1),
        delimited(
            tag("(#"),
            map_res(hex_digit1, |s| u32::from_str_radix(s, 16)),
            char(')'),
        ),
    )))(s)
}

pub fn part_one(input: &str) -> Option<usize> {
    let dig_plan = input
        .lines()
        .map(|s| parse_entry(s).unwrap().1)
        .collect_vec();

    // Determine the bounds of the grid
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (0, 0, 0, 0);
    let (mut x, mut y) = (0, 0);
    for (dir, len, _) in &dig_plan {
        match dir {
            Direction::Up => {
                y -= *len as i32;
                min_y = min_y.min(y);
            }
            Direction::Down => {
                y += *len as i32;
                max_y = max_y.max(y);
            }
            Direction::Left => {
                x -= *len as i32;
                min_x = min_x.min(x);
            }
            Direction::Right => {
                x += *len as i32;
                max_x = max_x.max(x);
            }
        }
    }
    let (width, height) = ((max_x - min_x + 1) as usize, (max_y - min_y + 1) as usize);
    let (x0, y0) = (-min_x as usize, -min_y as usize);

    // Add padding to the bounds, so we can easily flood fill the exterior
    let (width, height) = (width + 2, height + 2);
    let (x0, y0) = (x0 + 1, y0 + 1);

    // Create the grid
    let mut grid = vec![vec![false; width]; height];

    // Draw the path
    let (mut x, mut y) = (x0, y0);
    for (dir, len, _color) in dig_plan {
        let (dx, dy) = match dir {
            Direction::Up => (0, usize::MAX),
            Direction::Down => (0, 1),
            Direction::Left => (usize::MAX, 0),
            Direction::Right => (1, 0),
        };
        for _ in 0..len {
            grid[y][x] = true;
            x = x.wrapping_add(dx);
            y = y.wrapping_add(dy);
        }
    }

    // Flood fill the exterior cells
    let mut queue = vec![(0, 0)];
    let mut exterior = vec![vec![false; width]; height];
    let mut num_exterior = 0;

    while let Some((x, y)) = queue.pop() {
        if exterior[y][x] {
            continue;
        }
        exterior[y][x] = true;
        num_exterior += 1;
        if x > 0 && !grid[y][x - 1] {
            queue.push((x - 1, y));
        }
        if x + 1 < width && !grid[y][x + 1] {
            queue.push((x + 1, y));
        }
        if y > 0 && !grid[y - 1][x] {
            queue.push((x, y - 1));
        }
        if y + 1 < height && !grid[y + 1][x] {
            queue.push((x, y + 1));
        }
    }

    Some(width * height - num_exterior)
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
        assert_eq!(result, Some(62));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
