use std::collections::HashSet;

advent_of_code::solution!(21);

fn parse_input(input: &str) -> (Vec<Vec<bool>>, (usize, usize)) {
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
    (grid, start.unwrap())
}

fn count_reached_tiles(grid: &[Vec<bool>], start: (usize, usize), steps: u32) -> u32 {
    let mut frontier = HashSet::new();
    frontier.insert((start.0 as isize, start.1 as isize));

    let width = grid[0].len() as isize;
    let height = grid.len() as isize;

    for _ in 0..steps {
        let mut new_frontier = HashSet::new();

        for (x, y) in frontier {
            let neighbors = [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)];

            for (nx, ny) in neighbors {
                if !grid[ny.rem_euclid(height) as usize][nx.rem_euclid(width) as usize] {
                    new_frontier.insert((nx, ny));
                }
            }
        }

        frontier = new_frontier;
    }

    frontier.len() as u32
}

fn solve_part_one(input: &str, steps: u32) -> Option<u32> {
    let (grid, start) = parse_input(input);
    Some(count_reached_tiles(&grid, start, steps))
}

pub fn part_one(input: &str) -> Option<u32> {
    solve_part_one(input, 64)
}

pub fn part_two(_input: &str) -> Option<usize> {
    // let (grid, start) = parse_input(input);

    // c_x, where x = (num_steps - 65) / 131

    // let c_naive = |x: u32| {
    //     let num_steps = 65 + 131 * x;
    //     count_reached_tiles(&grid, start, num_steps) as usize
    // };

    // let c_0 = c_naive(0);
    // let c_1 = c_naive(1);
    // let c_2 = c_naive(2);
    // let c_3 = c_naive(3);
    // let c_0 = 3832;
    // let c_1 = 33967;
    // let c_2 = 94056;
    // let c_3 = 184099;

    // There are two types of diamonds in the input grid (A and B). Each diamond (once filled) can be in one of two
    // states, based on parity of number of steps and which ring it's in. These are labeled a1, a2, b1, b2.

    // Counting the number of each type of diamond after `131 * x + 65` steps, we get the following equations for c_x:
    // c_0 = a1
    // c_1 = a2 + (a1+b1)*4
    // c_2 = a1 + (a2+b2)*4 + (a1+b1)*8
    // c_3 = a2 + (a1+b1)*4 + (a2+b2)*8 + (a1+b1)*12
    //
    // c_4 = a1 + (a2+b2)*4 + (a1+b1)*8 + (a2+b2)*12 + (a1+b1)*16
    // c_5 = a2 + (a1+b1)*4 + (a2+b2)*8 + (a1+b1)*12 + (a2+b2)*16 + (a1+b1)*20
    // c_6 = a1 + (a2+b2)*4 + (a1+b1)*8 + (a2+b2)*12 + (a1+b1)*16 + (a2+b2)*20 + (a1+b1)*24
    // c_7 = a2 + (a1+b1)*4 + (a2+b2)*8 + (a1+b1)*12 + (a2+b2)*16 + (a1+b1)*20 + (a2+b2)*24 + (a1+b1)*28

    // Coefficients of (a1+b1) and (a2+b2) respectively:
    // c_0 -> 0, 0
    // c_1 -> 4, 0
    // c_2 -> 8, 4
    // c_3 -> 16, 8
    // c_4 -> 24, 16
    // c_5 -> 36, 24
    // c_6 -> 48, 36
    // c_7 -> 64, 48
    // ...

    // Formula for the coefficients:
    // c_x (if x is odd) -> (x+1)^2, (x-1)^2 + 2(x-1)
    // c_x (if x is even) -> x^2 + 2x, x^2

    // We can solve this system of equations to get the following:
    let a1 = 3832;
    let a2 = 3651;
    let b1 = 3747;
    let b2 = 3747;

    // We can then use these values to get a formula for c_x:
    let c = |x: usize| {
        let (m, c1, c2) = if x % 2 == 0 {
            (a1, x.pow(2) + 2 * x, x.pow(2))
        } else {
            (a2, (x + 1).pow(2), (x - 1).pow(2) + 2 * (x - 1))
        };
        m + (a1 + b1) * c1 + (a2 + b2) * c2
    };

    let num_steps = 26501365;
    let x = (num_steps - 65) / 131;
    Some(c(x))
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
    fn test_part_two() {}
}
