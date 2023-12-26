use itertools::Itertools;

advent_of_code::solution!(11);

fn solve(input: &str, expansion_factor: usize) -> Option<usize> {
    let mut galaxies = vec![];
    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            if c == '#' {
                galaxies.push((x, y));
            }
        }
    }

    // Vertical expansion
    let mut y = 0;
    let mut y_expansion = 0;
    for galaxy in galaxies.iter_mut() {
        if galaxy.1 > y + 1 {
            y_expansion += (galaxy.1 - y - 1) * (expansion_factor - 1);
        }
        y = galaxy.1;
        galaxy.1 += y_expansion;
    }

    // Horizontal expansion
    let mut x = 0;
    let mut x_expansion = 0;
    galaxies.sort_by_key(|(x, _)| *x);
    for galaxy in galaxies.iter_mut() {
        if galaxy.0 > x + 1 {
            x_expansion += (galaxy.0 - x - 1) * (expansion_factor - 1);
        }
        x = galaxy.0;
        galaxy.0 += x_expansion;
    }

    galaxies
        .iter()
        .tuple_combinations()
        .map(|((x1, y1), (x2, y2))| x1.abs_diff(*x2) + y1.abs_diff(*y2))
        .sum1()
}

pub fn part_one(input: &str) -> Option<usize> {
    solve(input, 2)
}

pub fn part_two(input: &str) -> Option<usize> {
    solve(input, 1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(374));
    }

    #[test]
    fn test_part_two() {
        let result = solve(&advent_of_code::template::read_file("examples", DAY), 10);
        assert_eq!(result, Some(1030));

        let result = solve(&advent_of_code::template::read_file("examples", DAY), 100);
        assert_eq!(result, Some(8410));
    }
}
