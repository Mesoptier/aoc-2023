use std::collections::HashMap;
use std::hash::Hash;

use itertools::Itertools;

advent_of_code::solution!(14);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
struct Segment {
    start: usize, // inclusive
    end: usize,   // exclusive
    num_rounded_rocks: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Field {
    vertical_segments: Vec<Vec<Segment>>,
    horizontal_segments: Vec<Vec<Segment>>,
}

impl Field {
    fn from_input(input: &str) -> Self {
        let grid = input
            .lines()
            .map(|line| line.chars().collect_vec())
            .collect_vec();

        let dim = grid.len();
        assert_eq!(grid[0].len(), grid.len());

        let build_segments = |vertical: bool| -> Vec<Vec<Segment>> {
            let mut segments = vec![vec![]; dim];

            for i in 0..dim {
                let mut segment_start = None;
                let mut num_rounded_rocks = 0;

                for j in 0..dim {
                    let (x, y) = if vertical { (i, j) } else { (j, i) };
                    let c = grid[y][x];

                    if c != '#' && segment_start.is_none() {
                        segment_start = Some(j);
                    }
                    if c == 'O' && vertical {
                        // Rounded rocks are only added for vertical segments, since the first slide direction is always up
                        num_rounded_rocks += 1;
                    }
                    if c == '#' && segment_start.is_some() {
                        segments[i].push(Segment {
                            start: segment_start.unwrap(),
                            end: j,
                            num_rounded_rocks,
                        });
                        segment_start = None;
                        num_rounded_rocks = 0;
                    }
                }

                if let Some(segment_start) = segment_start {
                    segments[i].push(Segment {
                        start: segment_start,
                        end: dim,
                        num_rounded_rocks,
                    });
                }
            }

            segments
        };

        Self {
            vertical_segments: build_segments(true),
            horizontal_segments: build_segments(false),
        }
    }

    fn slide_rounded_rocks(&mut self, vertical: bool, reverse: bool) {
        let (segments, other_segments) = if vertical {
            (&mut self.vertical_segments, &mut self.horizontal_segments)
        } else {
            (&mut self.horizontal_segments, &mut self.vertical_segments)
        };

        for i in 0..segments.len() {
            for segment in &mut segments[i] {
                // Range of rounded rocks in this segment
                let (start_j, end_j) = if reverse {
                    (segment.end - segment.num_rounded_rocks, segment.end)
                } else {
                    (segment.start, segment.start + segment.num_rounded_rocks)
                };

                // Transfer rounded rocks to intersecting segments
                for j in start_j..end_j {
                    // TODO: Binary search (but really, this could be a lookup table)
                    for other_segment in &mut other_segments[j] {
                        if other_segment.start <= i && i < other_segment.end {
                            other_segment.num_rounded_rocks += 1;
                        }
                    }
                }

                segment.num_rounded_rocks = 0;
            }
        }
    }

    fn total_load(&self, vertical: bool) -> usize {
        if vertical {
            let dim = self.vertical_segments.len();
            self.vertical_segments
                .iter()
                .map(|segments| {
                    segments
                        .iter()
                        .map(|segment| {
                            (segment.start..segment.end)
                                .take(segment.num_rounded_rocks)
                                .map(|y| dim - y)
                                .sum::<usize>()
                        })
                        .sum::<usize>()
                })
                .sum()
        } else {
            let dim = self.horizontal_segments.len();
            self.horizontal_segments
                .iter()
                .enumerate()
                .map(|(y, segments)| {
                    segments
                        .iter()
                        .map(|segment| segment.num_rounded_rocks * (dim - y))
                        .sum::<usize>()
                })
                .sum()
        }
    }

    fn print(&self) {
        let dim = self.vertical_segments.len();
        for y in 0..dim {
            for x in 0..dim {
                if let Some(segment) = self.vertical_segments[x]
                    .iter()
                    .find(|segment| segment.start <= y && y < segment.end)
                {
                    if y < segment.start + segment.num_rounded_rocks {
                        print!("O");
                    } else {
                        print!(".");
                    }
                } else {
                    print!("#");
                }
            }
            println!();
        }
        println!();
    }
}

pub fn part_one(input: &str) -> Option<usize> {
    let field = Field::from_input(input);
    Some(field.total_load(true))
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut field = Field::from_input(input);

    let mut cycles = 0;
    let mut cache = HashMap::<Field, usize>::new();
    let mut total_loads = vec![];

    // First cycle
    // Sliding north is implicit in loading the field
    field.slide_rounded_rocks(true, false);
    field.slide_rounded_rocks(false, false);
    field.slide_rounded_rocks(true, true);
    cycles += 1;

    loop {
        field.slide_rounded_rocks(false, true);
        field.slide_rounded_rocks(true, false);
        field.slide_rounded_rocks(false, false);
        field.slide_rounded_rocks(true, true);

        let total_load = field.total_load(false);
        cycles += 1;

        if let Some(prev_cycles) = cache.insert(field.clone(), cycles) {
            let cycles_repeat = cycles - prev_cycles;
            let cycles_remaining = (1_000_000_000 - cycles) % cycles_repeat;
            return Some(total_loads[total_loads.len() - cycles_repeat + cycles_remaining]);
        }

        total_loads.push(total_load);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(136));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(64));
    }
}
