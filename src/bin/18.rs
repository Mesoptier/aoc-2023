use itertools::Itertools;
use nom::branch::alt;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{char, digit1, space1};
use nom::combinator::{all_consuming, map, map_res};
use nom::sequence::{delimited, separated_pair, tuple};
use nom::IResult;

advent_of_code::solution!(18);

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn parse_entry(s: &str) -> IResult<&str, ((Direction, usize), (Direction, usize))> {
    all_consuming(separated_pair(
        separated_pair(
            alt((
                map(char('U'), |_| Direction::Up),
                map(char('D'), |_| Direction::Down),
                map(char('L'), |_| Direction::Left),
                map(char('R'), |_| Direction::Right),
            )),
            space1,
            map_res(digit1, str::parse),
        ),
        space1,
        delimited(
            tag("(#"),
            map(
                tuple((
                    map_res(take(5usize), |s: &str| usize::from_str_radix(s, 16)),
                    alt((
                        map(char('3'), |_| Direction::Up),
                        map(char('1'), |_| Direction::Down),
                        map(char('2'), |_| Direction::Left),
                        map(char('0'), |_| Direction::Right),
                    )),
                )),
                |(len, dir)| (dir, len),
            ),
            char(')'),
        ),
    ))(s)
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
enum EventType {
    Start,
    End,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Event {
    x: isize,
    y: isize,
    event_type: EventType,
}

fn compute_interior(dig_plan: &[(Direction, usize)]) -> usize {
    // Sweep line algorithm:
    // - Events are start-/endpoints of the vertical line segments (oriented downwards) of the dig plan.
    // - The state is the set of active vertical line segments and the current Y value.

    let mut events = vec![];
    let (mut x, mut y) = (0isize, 0isize);

    for (dir, len) in dig_plan {
        let (dx, dy) = match dir {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        };

        match dir {
            Direction::Up => {
                events.push(Event {
                    x,
                    y: y - (*len as isize),
                    event_type: EventType::Start,
                });
                events.push(Event {
                    x,
                    y,
                    event_type: EventType::End,
                });
            }
            Direction::Down => {
                events.push(Event {
                    x,
                    y,
                    event_type: EventType::Start,
                });
                events.push(Event {
                    x,
                    y: y + (*len as isize),
                    event_type: EventType::End,
                });
            }
            _ => {}
        }

        x += dx * (*len as isize);
        y += dy * (*len as isize);
    }

    events.sort_by_key(|e| {
        (
            e.y,
            e.x,
            match e.event_type {
                EventType::Start => 1,
                EventType::End => 0,
            },
        )
    });

    let mut y = isize::MIN;
    let mut endpoints = vec![];
    let mut interior = 0;

    for (event_y, grouped_events) in &events.into_iter().group_by(|e| e.y) {
        let prev_endpoints = endpoints.clone();

        // Update endpoints
        for event in grouped_events {
            match event.event_type {
                EventType::Start => endpoints.push(event.x),
                EventType::End => endpoints.retain(|&x| x != event.x),
            }
        }
        endpoints.sort();

        // How many X values are covered by the intervals bounded by endpoints
        let mut x_span = 0;
        // Overlap with next x_span
        let mut x_span_overlap = 0;
        for (prev_start, prev_end) in prev_endpoints.into_iter().tuples() {
            x_span += prev_end - prev_start + 1;

            for (&start, &end) in endpoints.iter().tuples() {
                // Subtract overlap between previous and current intervals. This only works if the intervals in the two
                // sets are internally disjoint, which is guaranteed by the sorting of endpoints.
                if prev_start <= end && prev_end >= start {
                    x_span_overlap += prev_end.min(end) - prev_start.max(start) + 1;
                }
            }
        }

        // How many Y values are covered since the previous event
        let y_span = event_y.saturating_sub(y);

        // Update interior
        // - area of the rectangles covered since previous event
        interior += (x_span * y_span) as usize;
        // - compensate for overlap with next area
        interior += (x_span - x_span_overlap) as usize;

        // Update Y
        y = event_y;
    }

    interior
}

pub fn part_one(input: &str) -> Option<usize> {
    let dig_plan = input
        .lines()
        .map(|s| parse_entry(s).unwrap().1 .0)
        .collect_vec();
    Some(compute_interior(&dig_plan))
}

pub fn part_two(input: &str) -> Option<usize> {
    let dig_plan = input
        .lines()
        .map(|s| parse_entry(s).unwrap().1 .1)
        .collect_vec();
    Some(compute_interior(&dig_plan))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(62));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
