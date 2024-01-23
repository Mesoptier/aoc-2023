use advent_of_code::util::coord::Coord;

advent_of_code::solution!(3);

struct CharGrid<'a> {
    data: &'a [u8],
    width: usize,
    width_with_nl: usize,
    height: usize,
}

impl<'a> CharGrid<'a> {
    fn new(data: &'a str) -> Self {
        // Data must be ASCII and contain CR (not CRLF) line endings
        debug_assert!(data.is_ascii());
        debug_assert!(data.find('\r').is_none());

        let data = data.as_bytes();

        let width = data.iter().position(|&c| c == b'\n').unwrap();
        let width_with_nl = width + 1;

        let height = data.len() / width_with_nl;

        debug_assert_eq!(data.len(), height * width_with_nl);

        Self {
            data,
            width,
            width_with_nl,
            height,
        }
    }

    fn get(&self, coord: &Coord) -> Option<char> {
        if coord.x >= self.width || coord.y >= self.height {
            None
        } else {
            Some(unsafe {
                // SAFETY: coord is within bounds
                self.get_unchecked(coord)
            })
        }
    }

    unsafe fn get_unchecked(&self, coord: &Coord) -> char {
        *self
            .data
            .get_unchecked(coord.y * self.width_with_nl + coord.x) as char
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
            let coord = Coord::new(x, y);

            if let Some(d) = grid.get(&coord).unwrap().to_digit(10) {
                // Check left neighbors for special char
                if !is_part_num && num == 0 && x > 0 {
                    for ny in y_prev..=y_next {
                        if is_special_char(grid.get(&Coord::new(x - 1, ny)).unwrap()) {
                            is_part_num = true;
                            break;
                        }
                    }
                }
                // Check top/bottom neighbors for special char
                if !is_part_num
                    && y_prev != y
                    && is_special_char(grid.get(&Coord::new(x, y_prev)).unwrap())
                {
                    is_part_num = true;
                }
                if !is_part_num
                    && y_next != y
                    && is_special_char(grid.get(&Coord::new(x, y_next)).unwrap())
                {
                    is_part_num = true;
                }

                num = num * 10 + d;
            } else {
                // Check right neighbors for special char (note that x has already been incremented)
                if num != 0 && !is_part_num {
                    for ny in y_prev..=y_next {
                        let coord = Coord::new(x, ny);
                        if is_special_char(grid.get(&coord).unwrap()) {
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
