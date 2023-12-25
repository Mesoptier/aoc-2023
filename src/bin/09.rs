use itertools::Itertools;

advent_of_code::solution!(9);

fn parse_input(input: &str) -> Vec<Vec<i32>> {
    input
        .lines()
        .map(|line| line.split(' ').map(|s| s.parse::<i32>().unwrap()).collect())
        .collect()
}

fn extrapolate_history(history: Vec<i32>) -> (i32, i32) {
    if history.iter().all(|x| *x == 0) {
        return (0, 0);
    }

    let first = *history.first().unwrap();
    let last = *history.last().unwrap();

    let history = history
        .into_iter()
        .tuple_windows()
        .map(|(a, b)| b - a)
        .collect_vec();

    let (dfirst, dlast) = extrapolate_history(history);
    (first - dfirst, last + dlast)
}

pub fn part_one(input: &str) -> Option<i32> {
    let histories = parse_input(input);
    histories
        .into_iter()
        .map(extrapolate_history)
        .map(|(_, last)| last)
        .sum1()
}

pub fn part_two(input: &str) -> Option<i32> {
    let histories = parse_input(input);
    histories
        .into_iter()
        .map(extrapolate_history)
        .map(|(first, _)| first)
        .sum1()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(114));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(2));
    }
}
