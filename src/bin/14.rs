use std::collections::HashMap;
use std::hash::Hash;
use std::ops::Range;

use itertools::izip;

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
        let (lines, dim) = {
            let mut lines = input.lines().peekable();
            let dim = lines.peek().unwrap().len();
            (lines, dim)
        };

        let mut vertical_segments = vec![];
        let mut horizontal_segments = vec![];

        let mut vertical_counts = vec![];
        let mut horizontal_counts = vec![];

        #[derive(Clone)]
        struct Frontier {
            j_start: usize, // y for vertical frontiers, x for horizontal frontiers
            segment_idx: usize,
            count: usize,
            lookup: Vec<usize>,
        }

        fn close_frontier(
            frontier: &mut Option<Frontier>,
            i: usize,
            j_end: usize,
            segments: &mut Vec<Segment>,
            counts: &mut Vec<usize>,
        ) {
            if let Some(frontier) = frontier.take() {
                let segment = Segment {
                    i,
                    j_range: frontier.j_start..j_end,
                    lookup: frontier.lookup,
                };

                // Resize segments and counts to fit the new segment
                if segments.len() <= frontier.segment_idx {
                    segments.resize(
                        frontier.segment_idx + 1,
                        Segment {
                            i: 0,
                            j_range: 0..0,
                            lookup: vec![],
                        },
                    );
                    counts.resize(frontier.segment_idx + 1, 0);
                }

                segments[frontier.segment_idx] = segment;
                counts[frontier.segment_idx] = frontier.count;
            }
        }

        let mut vertical_frontiers: Vec<Option<Frontier>> = vec![None; dim];
        let mut next_vertical_segment_idx = 0;
        let mut next_horizontal_segment_idx = 0;

        for (y, line) in lines.enumerate() {
            let mut horizontal_frontier: Option<Frontier> = None;

            for (x, c) in line.chars().enumerate() {
                let vertical_frontier = &mut vertical_frontiers[x];

                if c == '#' {
                    // Close current horizontal/vertical frontier
                    close_frontier(
                        &mut horizontal_frontier,
                        y,
                        x,
                        &mut horizontal_segments,
                        &mut horizontal_counts,
                    );
                    close_frontier(
                        vertical_frontier,
                        x,
                        y,
                        &mut vertical_segments,
                        &mut vertical_counts,
                    );
                } else {
                    // Open new horizontal/vertical frontier
                    let horizontal_frontier = horizontal_frontier.get_or_insert_with(|| {
                        let frontier = Frontier {
                            j_start: x,
                            segment_idx: next_horizontal_segment_idx,
                            count: 0,
                            lookup: vec![],
                        };
                        next_horizontal_segment_idx += 1;
                        frontier
                    });
                    let vertical_frontier = vertical_frontier.get_or_insert_with(|| {
                        let frontier = Frontier {
                            j_start: y,
                            segment_idx: next_vertical_segment_idx,
                            count: 0,
                            lookup: vec![],
                        };
                        next_vertical_segment_idx += 1;
                        frontier
                    });

                    // Add horizontal/vertical frontier to each other's lookup
                    horizontal_frontier
                        .lookup
                        .push(vertical_frontier.segment_idx);
                    vertical_frontier
                        .lookup
                        .push(horizontal_frontier.segment_idx);

                    if c == 'O' {
                        // Rounded rocks are only added for vertical segments
                        vertical_frontier.count += 1;
                    }
                }
            }

            // Close current horizontal frontier
            close_frontier(
                &mut horizontal_frontier,
                y,
                dim,
                &mut horizontal_segments,
                &mut horizontal_counts,
            );
        }

        // Close current vertical frontiers
        for (x, frontier) in vertical_frontiers.iter_mut().enumerate() {
            close_frontier(
                frontier,
                x,
                dim,
                &mut vertical_segments,
                &mut vertical_counts,
            );
        }

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
                (segment.j_range.len() - *count, segment.j_range.len())
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
