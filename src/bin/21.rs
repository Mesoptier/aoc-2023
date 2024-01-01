use std::collections::HashSet;
advent_of_code::solution!(21);

fn solve_part_one(input: &str, steps: u32) -> Option<u32> {
    let mut start = None;
    let grid = input
        .lines()
        .enumerate()
        .map(|(y, line)| {
            line.chars()
                .enumerate()
                .map(|(x, c)| {
                    if c == 'S' {
                        start = Some((x, y));
                    }
                    c == '#'
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let start = start.unwrap();

    let mut frontier = HashSet::new();
    frontier.insert(start);

    for _ in 0..steps {
        let mut new_frontier = HashSet::new();

        for (x, y) in frontier {
            let neighbors = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];

            for (nx, ny) in neighbors {
                if !grid[ny][nx] {
                    new_frontier.insert((nx, ny));
                }
            }
        }

        frontier = new_frontier;
    }

    Some(frontier.len() as u32)
}

pub fn part_one(input: &str) -> Option<u32> {
    solve_part_one(input, 64)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = solve_part_one(&advent_of_code::template::read_file("examples", DAY), 6);
        assert_eq!(result, Some(16));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
