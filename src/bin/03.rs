use advent_of_code::util::coord::{Coord, CoordIndexer};
use advent_of_code::util::VecTable;

advent_of_code::solution!(3);

fn parse_input(input: &str) -> VecTable<Coord, char, CoordIndexer> {
    let mut width = None;
    let data = input
        .lines()
        .flat_map(|line| {
            if width.is_none() {
                width = Some(line.len());
            } else {
                debug_assert_eq!(width.unwrap(), line.len());
            }
            line.chars()
        })
        .collect::<Vec<_>>();
    let width = width.unwrap();
    let height = data.len() / width;
    VecTable::from_vec(data, CoordIndexer::new(width, height))
}

fn is_special_char(c: char) -> bool {
    !c.is_ascii_digit() && c != '.'
}

pub fn part_one(input: &str) -> Option<u32> {
    let grid = parse_input(input);

    let mut result = 0;

    for y in 0..grid.indexer().height {
        let mut num = 0;
        let mut is_part_num = false;

        let y_prev = if y > 0 { y - 1 } else { y };
        let y_next = if y + 1 < grid.indexer().height {
            y + 1
        } else {
            y
        };

        for x in 0..grid.indexer().width {
            let coord = Coord::new(x, y);

            if let Some(d) = grid.get(&coord).to_digit(10) {
                // Check left neighbors for special char
                if !is_part_num && num == 0 && x > 0 {
                    for ny in y_prev..=y_next {
                        if is_special_char(*grid.get(&Coord::new(x - 1, ny))) {
                            is_part_num = true;
                            break;
                        }
                    }
                }
                // Check top/bottom neighbors for special char
                if !is_part_num && y_prev != y && is_special_char(*grid.get(&Coord::new(x, y_prev)))
                {
                    is_part_num = true;
                }
                if !is_part_num && y_next != y && is_special_char(*grid.get(&Coord::new(x, y_next)))
                {
                    is_part_num = true;
                }

                num = num * 10 + d;
            } else {
                // Check right neighbors for special char (note that x has already been incremented)
                if num != 0 && !is_part_num {
                    for ny in y_prev..=y_next {
                        let coord = Coord::new(x, ny);
                        if is_special_char(*grid.get(&coord)) {
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
