use itertools::Itertools;
advent_of_code::solution!(3);

struct CharGrid<'a> {
    data: &'a [u8],
    width: usize,
    width_with_nl: usize,
    height: usize,
}

impl<'a> CharGrid<'a> {
    fn new(data: &'a str) -> Self {
        let data = data.as_bytes();

        let (width, line_sep_char) = data
            .iter()
            .find_position(|&c| matches!(c, b'\n' | b'\r'))
            .unwrap();

        let width_with_nl = width
            + match line_sep_char {
                b'\n' => 1,
                b'\r' => 2,
                _ => unreachable!(),
            };

        // Note: we allow the last line to not have a newline, hence the ceiling division
        let height = (data.len() + width_with_nl - 1) / width_with_nl;

        debug_assert!(
            data.len() == height * width_with_nl
                || data.len() == height * width_with_nl - width_with_nl + width,
            "data must be rectangular (with or without trailing newline)"
        );

        Self {
            data,
            width,
            width_with_nl,
            height,
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<char> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(unsafe {
                // SAFETY: coord is within bounds
                self.get_unchecked(x, y)
            })
        }
    }

    unsafe fn get_unchecked(&self, x: usize, y: usize) -> char {
        *self.data.get_unchecked(y * self.width_with_nl + x) as char
    }
}

fn is_special_char(c: char) -> bool {
    !c.is_ascii_digit() && c != '.'
}

pub fn part_one(input: &str) -> Option<u32> {
    let grid = CharGrid::new(input);

    let mut result = 0;

    for y in 0..grid.height {
        let mut num = 0;
        let mut is_part_num = false;

        let y_prev = if y > 0 { y - 1 } else { y };
        let y_next = if y + 1 < grid.height { y + 1 } else { y };

        for x in 0..grid.width {
            if let Some(d) = grid.get(x, y).unwrap().to_digit(10) {
                // Check left neighbors for special char
                if !is_part_num && num == 0 && x > 0 {
                    for ny in y_prev..=y_next {
                        if is_special_char(grid.get(x - 1, ny).unwrap()) {
                            is_part_num = true;
                            break;
                        }
                    }
                }
                // Check top/bottom neighbors for special char
                if !is_part_num && y_prev != y && is_special_char(grid.get(x, y_prev).unwrap()) {
                    is_part_num = true;
                }
                if !is_part_num && y_next != y && is_special_char(grid.get(x, y_next).unwrap()) {
                    is_part_num = true;
                }

                num = num * 10 + d;
            } else {
                // Check right neighbors for special char (note that x has already been incremented)
                if num != 0 && !is_part_num {
                    for ny in y_prev..=y_next {
                        if is_special_char(grid.get(x, ny).unwrap()) {
                            is_part_num = true;
                            break;
                        }
                    }
                }

                if is_part_num {
                    result += num;
                    is_part_num = false;
                }

                num = 0;
            }
        }

        if is_part_num {
            result += num;
        }
    }

    Some(result)
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
        assert_eq!(result, Some(4361));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(467835));
    }
}
