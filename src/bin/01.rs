advent_of_code::solution!(1);

pub fn part_one(input: &str) -> Option<u32> {
    input
        .lines()
        .map(|line| {
            let first = line.chars().find_map(|c| c.to_digit(10)).unwrap();
            let last = line.chars().rev().find_map(|c| c.to_digit(10)).unwrap();
            first * 10 + last
        })
        .sum::<u32>()
        .into()
}

pub fn part_two(input: &str) -> Option<u32> {
    input
        .lines()
        .map(|line| {
            let first = (0..line.len())
                .find_map(|index| match &line[index..] {
                    s if s.starts_with('1') || s.starts_with("one") => Some(1),
                    s if s.starts_with('2') || s.starts_with("two") => Some(2),
                    s if s.starts_with('3') || s.starts_with("three") => Some(3),
                    s if s.starts_with('4') || s.starts_with("four") => Some(4),
                    s if s.starts_with('5') || s.starts_with("five") => Some(5),
                    s if s.starts_with('6') || s.starts_with("six") => Some(6),
                    s if s.starts_with('7') || s.starts_with("seven") => Some(7),
                    s if s.starts_with('8') || s.starts_with("eight") => Some(8),
                    s if s.starts_with('9') || s.starts_with("nine") => Some(9),
                    _ => None,
                })
                .unwrap();
            let last = (0..line.len())
                .rev()
                .find_map(|index| match &line[..=index] {
                    s if s.ends_with('1') || s.ends_with("one") => Some(1),
                    s if s.ends_with('2') || s.ends_with("two") => Some(2),
                    s if s.ends_with('3') || s.ends_with("three") => Some(3),
                    s if s.ends_with('4') || s.ends_with("four") => Some(4),
                    s if s.ends_with('5') || s.ends_with("five") => Some(5),
                    s if s.ends_with('6') || s.ends_with("six") => Some(6),
                    s if s.ends_with('7') || s.ends_with("seven") => Some(7),
                    s if s.ends_with('8') || s.ends_with("eight") => Some(8),
                    s if s.ends_with('9') || s.ends_with("nine") => Some(9),
                    _ => None,
                })
                .unwrap();

            first * 10 + last
        })
        .sum::<u32>()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file_part(
            "examples", DAY, 1,
        ));
        assert_eq!(result, Some(142));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file_part(
            "examples", DAY, 2,
        ));
        assert_eq!(result, Some(281));
    }
}
