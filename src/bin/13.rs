use itertools::Itertools;

use advent_of_code::util::BitMatrix;

advent_of_code::solution!(13);

fn parse_input_iter(input: &str) -> impl Iterator<Item = Pattern> + '_ {
    input.lines().batching(|lines| {
        let mut data = BitMatrix::<4>::new();
        let mut width = 0;
        let mut height = 0;

        for (i, line) in lines.take_while(|line| !line.is_empty()).enumerate() {
            for (j, c) in line.chars().enumerate() {
                if c == '#' {
                    data.set(i, j);
                }
            }

            width = width.max(line.len());
            height += 1;
        }

        if height > 0 {
            Some(Pattern {
                data,
                width,
                height,
            })
        } else {
            None
        }
    })
}

struct Pattern {
    data: BitMatrix<4>,
    width: usize,
    height: usize,
}

impl Pattern {
    fn transpose(&mut self) {
        self.data = self.data.transpose();
        std::mem::swap(&mut self.width, &mut self.height);
    }

    fn find_horizontal_reflection_line(&self, target_smudges: usize) -> Option<usize> {
        let rows = unsafe {
            let (prefix, rows, suffix) = self.data.bytes().align_to::<u32>();
            assert!(prefix.is_empty());
            assert!(suffix.is_empty());
            rows
        };

        (1..self.height).find(|&num_rows_above| {
            let num_rows_below = self.height - num_rows_above;
            let max_offset = usize::min(num_rows_above - 1, num_rows_below - 1);

            let mut smudges = 0;

            for offset in 0..=max_offset {
                let row_above = rows[num_rows_above - offset - 1];
                let row_below = rows[num_rows_above + offset];

                let diff = row_above ^ row_below;
                smudges += diff.count_ones() as usize;

                if smudges > target_smudges {
                    return false;
                }
            }

            smudges == target_smudges
        })
    }
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..self.height {
            for j in 0..self.width {
                write!(f, "{}", if self.data.get(i, j) { '#' } else { '.' })?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn solve(input: &str, smudges: usize) -> Option<usize> {
    parse_input_iter(input)
        .map(|mut pattern| {
            pattern
                .find_horizontal_reflection_line(smudges)
                .map_or_else(
                    || {
                        pattern.transpose();
                        pattern.find_horizontal_reflection_line(smudges).unwrap()
                    },
                    |line| line * 100,
                )
        })
        .sum::<usize>()
        .into()
}

pub fn part_one(input: &str) -> Option<usize> {
    solve(input, 0)
}

pub fn part_two(input: &str) -> Option<usize> {
    solve(input, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(405));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(400));
    }
}
