use std::collections::HashMap;

advent_of_code::solution!(3);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Coord(usize, usize);
impl Coord {
    /// Get all (orthogonal and diagonal) neighbors of this coordinate within the given bounds.
    fn neighbors(&self, bounds: Coord) -> Vec<Coord> {
        let mut neighbors = Vec::new();

        for i in self.0.saturating_sub(1)..=self.0.saturating_add(1) {
            for j in self.1.saturating_sub(1)..=self.1.saturating_add(1) {
                if i == self.0 && j == self.1 {
                    continue;
                }

                if i < bounds.0 && j < bounds.1 {
                    neighbors.push(Coord(i, j));
                }
            }
        }

        neighbors
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let grid = input
        .lines()
        .map(|line| line.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let mut result = 0;

    for i in 0..grid.len() {
        let mut j = 0;

        while j < grid[i].len() {
            let mut number = 0;
            let mut is_part_number = false;

            while let Some(digit) = grid[i][j].to_digit(10) {
                number = number * 10 + digit;

                if !is_part_number {
                    for Coord(ni, nj) in Coord(i, j).neighbors(Coord(grid.len(), grid[i].len())) {
                        match grid[ni][nj] {
                            '0'..='9' | '.' => {}
                            _ => {
                                is_part_number = true;
                                break;
                            }
                        }
                    }
                }

                if j < grid[i].len() - 1 {
                    j += 1;
                } else {
                    break;
                }
            }

            if is_part_number {
                result += number;
            }

            j += 1;
        }
    }

    Some(result)
}

pub fn part_two(input: &str) -> Option<u32> {
    let grid = input
        .lines()
        .map(|line| line.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let mut gears = HashMap::<Coord, Vec<u32>>::new();

    for i in 0..grid.len() {
        let mut j = 0;

        while j < grid[i].len() {
            let mut number = 0;
            let mut gear_coord = None;

            while let Some(digit) = grid[i][j].to_digit(10) {
                number = number * 10 + digit;

                if gear_coord.is_none() {
                    for Coord(ni, nj) in Coord(i, j).neighbors(Coord(grid.len(), grid[i].len())) {
                        if grid[ni][nj] == '*' {
                            gear_coord = Some(Coord(ni, nj));
                            break;
                        }
                    }
                }

                if j < grid[i].len() - 1 {
                    j += 1;
                } else {
                    break;
                }
            }

            if let Some(gear_coord) = gear_coord {
                gears.entry(gear_coord).or_default().push(number);
            }

            j += 1;
        }
    }

    let mut result = 0;

    for (_coord, numbers) in gears {
        if numbers.len() == 2 {
            result += numbers[0] * numbers[1];
        }
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(4361));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(467835));
    }
}
