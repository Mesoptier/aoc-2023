use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;

use itertools::{izip, Itertools};

advent_of_code::solution!(14);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Segment {
    i: usize,
    j_range: Range<usize>,
    // Index of the segment intersecting this one in the other direction for each j in j_range
    lookup: Vec<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct Field {
    dim: usize,

    vertical_segments: Vec<Segment>,   // (i, j) = (x, y)
    horizontal_segments: Vec<Segment>, // (i, j) = (y, x)

    // Number of rounded rocks per vertical/horizontal segment
    vertical_counts: Vec<usize>,
    horizontal_counts: Vec<usize>,
}

impl Field {
    fn from_input(input: &str) -> Self {
        let grid = input
            .lines()
            .map(|line| line.chars().collect_vec())
            .collect_vec();

        let dim = grid.len();
        assert_eq!(grid[0].len(), grid.len());

        let build_segments = |vertical: bool| -> (Vec<Segment>, Vec<usize>) {
            let mut segments = vec![];
            let mut counts = vec![];

            for i in 0..dim {
                let mut j_range_start = None;
                let mut count = 0;

                for j in 0..dim {
                    let (x, y) = if vertical { (i, j) } else { (j, i) };
                    let c = grid[y][x];

                    if c != '#' && j_range_start.is_none() {
                        j_range_start = Some(j);
                    }
                    if c == 'O' && vertical {
                        // Rounded rocks are only added for vertical segments,
                        // since the first slide direction is always up
                        count += 1;
                    }
                    if c == '#' && j_range_start.is_some() {
                        segments.push(Segment {
                            i,
                            j_range: j_range_start.unwrap()..j,
                            lookup: vec![],
                        });
                        counts.push(count);

                        j_range_start = None;
                        count = 0;
                    }
                }

                if let Some(segment_start) = j_range_start {
                    segments.push(Segment {
                        i,
                        j_range: segment_start..dim,
                        lookup: vec![],
                    });
                    counts.push(count);
                }
            }

            (segments, counts)
        };

        let build_lookup = |segments: &mut Vec<Segment>, other_segments: &Vec<Segment>| {
            for segment in segments {
                segment.lookup = vec![usize::MAX; segment.j_range.len()];

                other_segments
                    .iter()
                    .enumerate()
                    .filter(|(_, other_segment)| {
                        segment.j_range.contains(&other_segment.i)
                            && other_segment.j_range.contains(&segment.i)
                    })
                    .for_each(|(idx, other_segment)| {
                        let offset = other_segment.i - segment.j_range.start;
                        segment.lookup[offset] = idx;
                    });
            }
        };

        // Build segments
        let (mut vertical_segments, vertical_counts) = build_segments(true);
        let (mut horizontal_segments, horizontal_counts) = build_segments(false);

        // Build lookup tables
        build_lookup(&mut vertical_segments, &horizontal_segments);
        build_lookup(&mut horizontal_segments, &vertical_segments);

        Self {
            dim,
            vertical_segments,
            horizontal_segments,
            vertical_counts,
            horizontal_counts,
        }
    }

    fn slide_rounded_rocks(&mut self, vertical: bool, reverse: bool) {
        let (segments, counts, other_counts) = if vertical {
            (
                &self.vertical_segments,
                &mut self.vertical_counts,
                &mut self.horizontal_counts,
            )
        } else {
            (
                &self.horizontal_segments,
                &mut self.horizontal_counts,
                &mut self.vertical_counts,
            )
        };

        for (segment, count) in izip!(segments, counts) {
            let (offset_start, offset_end) = if reverse {
                ((segment.j_range.len() - *count), segment.j_range.len())
            } else {
                (0, *count)
            };

            // Transfer rounded rocks to segments in the other direction
            for other_segment_idx in segment.lookup[offset_start..offset_end].iter() {
                other_counts[*other_segment_idx] += 1;
            }
            *count = 0;
        }
    }

    /// Calculates the total load on the north support beams.
    ///
    /// Assumes the last slide direction was vertical, either north (`reverse = false`) or south (`reverse = true`).
    fn total_load(&self, reverse: bool) -> usize {
        let mut total_load = 0;
        for (segment, count) in izip!(&self.vertical_segments, &self.vertical_counts) {
            let (y_start, y_end) = if !reverse {
                (segment.j_range.start, segment.j_range.start + *count)
            } else {
                (segment.j_range.end - *count, segment.j_range.end)
            };

            for y in y_start..y_end {
                let load = self.dim - y;
                total_load += load;
            }
        }
        total_load
    }
}

pub fn part_one(input: &str) -> Option<usize> {
    let field = Field::from_input(input);
    Some(field.total_load(false))
}

pub fn part_two(input: &str) -> Option<usize> {
    let mut field = Field::from_input(input);

    let mut cycles = 0;
    let mut cache = HashMap::<Vec<usize>, usize>::new();
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
        // .total_load() assumes last slide direction was vertical, so we calculate it before sliding east
        let total_load = field.total_load(true);
        field.slide_rounded_rocks(true, true);

        cycles += 1;

        if let Some(prev_cycles) = cache.insert(field.horizontal_counts.clone(), cycles) {
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
