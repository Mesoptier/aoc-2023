use std::collections::{HashSet, VecDeque};

advent_of_code::solution!(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

fn compute_energized_tiles(
    map: Vec<Vec<char>>,
    initial_beam_front: (usize, usize, Direction),
) -> u32 {
    let width = map[0].len();
    let height = map.len();

    let mut beam_fronts = VecDeque::<(usize, usize, Direction)>::new();
    beam_fronts.push_front(initial_beam_front);

    let mut energized = HashSet::<(usize, usize)>::new();
    let mut visited = HashSet::<(usize, usize, Direction)>::new();

    while let Some(beam_front) = beam_fronts.pop_front() {
        let (x, y, direction) = beam_front;

        energized.insert((x, y));
        visited.insert((x, y, direction));

        let next_directions = match (map[y][x], direction) {
            ('/', Direction::Up) => vec![Direction::Right],
            ('/', Direction::Right) => vec![Direction::Up],
            ('/', Direction::Down) => vec![Direction::Left],
            ('/', Direction::Left) => vec![Direction::Down],
            ('\\', Direction::Up) => vec![Direction::Left],
            ('\\', Direction::Left) => vec![Direction::Up],
            ('\\', Direction::Down) => vec![Direction::Right],
            ('\\', Direction::Right) => vec![Direction::Down],
            ('|', Direction::Left) | ('|', Direction::Right) => {
                vec![Direction::Up, Direction::Down]
            }
            ('-', Direction::Up) | ('-', Direction::Down) => {
                vec![Direction::Left, Direction::Right]
            }
            (_, direction) => vec![direction],
        };

        for next_direction in next_directions {
            let next_coord = match next_direction {
                Direction::Up if y > 0 => Some((x, y - 1)),
                Direction::Right if x + 1 < width => Some((x + 1, y)),
                Direction::Down if y + 1 < height => Some((x, y + 1)),
                Direction::Left if x > 0 => Some((x - 1, y)),
                _ => None,
            };

            if let Some((next_x, next_y)) = next_coord {
                if !visited.contains(&(next_x, next_y, next_direction)) {
                    beam_fronts.push_front((next_x, next_y, next_direction));
                }
            }
        }
    }

    energized.len() as u32
}

fn parse_input(input: &str) -> Vec<Vec<char>> {
    input
        .lines()
        .map(|line| line.chars().collect::<Vec<char>>())
        .collect::<Vec<Vec<char>>>()
}

pub fn part_one(input: &str) -> Option<u32> {
    let map = parse_input(input);
    compute_energized_tiles(map, (0, 0, Direction::Right)).into()
}

pub fn part_two(input: &str) -> Option<u32> {
    let map = parse_input(input);

    let width = map[0].len();
    let height = map.len();

    let mut initial_beam_fronts = vec![];
    for x in 0..width {
        initial_beam_fronts.push((x, 0, Direction::Down));
        initial_beam_fronts.push((x, height - 1, Direction::Up));
    }
    for y in 0..height {
        initial_beam_fronts.push((0, y, Direction::Right));
        initial_beam_fronts.push((width - 1, y, Direction::Left));
    }

    initial_beam_fronts
        .iter()
        .map(|&front| compute_energized_tiles(map.clone(), front))
        .max()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(46));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(51));
    }
}
