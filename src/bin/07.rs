use std::cmp::Ordering;

use itertools::Itertools;
use nom::character::complete::{alphanumeric1, digit1, line_ending, space1};
use nom::combinator::map_res;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;

advent_of_code::solution!(7);

fn parse_input(input: &str) -> IResult<&str, Vec<(&str, u32)>> {
    separated_list1(
        line_ending,
        separated_pair(alphanumeric1, space1, map_res(digit1, str::parse)),
    )(input)
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct Card(char);

impl Card {
    fn value(&self) -> u32 {
        match self.0 {
            'A' => 14,
            'K' => 13,
            'Q' => 12,
            'J' => 11,
            'T' => 10,
            '9' => 9,
            '8' => 8,
            '7' => 7,
            '6' => 6,
            '5' => 5,
            '4' => 4,
            '3' => 3,
            '2' => 2,
            _ => unreachable!(),
        }
    }
}

impl PartialOrd<Self> for Card {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Card {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
enum HandType {
    FiveOfAKind,
    FourOfAKind,
    FullHouse,
    ThreeOfAKind,
    TwoPairs,
    OnePair,
    HighCard,
}

#[derive(Eq, PartialEq, Debug)]
struct Hand {
    cards: [Card; 5],
}

impl Hand {
    fn hand_type(&self) -> HandType {
        let sorted_cards = {
            let mut cards = self.cards;
            cards.sort();
            cards
        };

        let [a, b, c, d, e] = sorted_cards;

        if a == b && b == c && c == d && d == e {
            HandType::FiveOfAKind
        } else if (a == b && b == c && c == d) || (b == c && c == d && d == e) {
            HandType::FourOfAKind
        } else if (a == b && b == c && d == e) || (a == b && c == d && d == e) {
            HandType::FullHouse
        } else if (a == b && b == c) || (b == c && c == d) || (c == d && d == e) {
            HandType::ThreeOfAKind
        } else if (a == b && c == d) || (a == b && d == e) || (b == c && d == e) {
            HandType::TwoPairs
        } else if a == b || b == c || c == d || d == e {
            HandType::OnePair
        } else {
            HandType::HighCard
        }
    }
}

impl PartialOrd<Self> for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hand_type()
            .cmp(&other.hand_type())
            .reverse()
            .then(self.cards.cmp(&other.cards))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct JokerCard(char);

impl JokerCard {
    fn value(&self) -> u32 {
        match self.0 {
            'A' => 14,
            'K' => 13,
            'Q' => 12,
            'T' => 10,
            '9' => 9,
            '8' => 8,
            '7' => 7,
            '6' => 6,
            '5' => 5,
            '4' => 4,
            '3' => 3,
            '2' => 2,
            'J' => 1,
            _ => unreachable!(),
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct JokerHand {
    cards: [JokerCard; 5],
}

impl JokerHand {
    fn hand_type(&self) -> HandType {
        let sorted_cards = {
            let mut cards = self.cards;
            cards.sort();
            cards
        };

        let [a, b, c, d, e] = sorted_cards;

        let hand_type = if a == b && b == c && c == d && d == e {
            HandType::FiveOfAKind
        } else if (a == b && b == c && c == d) || (b == c && c == d && d == e) {
            HandType::FourOfAKind
        } else if (a == b && b == c && d == e) || (a == b && c == d && d == e) {
            HandType::FullHouse
        } else if (a == b && b == c) || (b == c && c == d) || (c == d && d == e) {
            HandType::ThreeOfAKind
        } else if (a == b && c == d) || (a == b && d == e) || (b == c && d == e) {
            HandType::TwoPairs
        } else if a == b || b == c || c == d || d == e {
            HandType::OnePair
        } else {
            HandType::HighCard
        };

        let num_jokers = sorted_cards.iter().filter(|card| card.0 == 'J').count();
        match (hand_type, num_jokers) {
            (HandType::FiveOfAKind, _) => HandType::FiveOfAKind,
            (HandType::FourOfAKind, 1) | (HandType::FourOfAKind, 4) => HandType::FiveOfAKind,
            (HandType::FourOfAKind, _) => HandType::FourOfAKind,
            (HandType::FullHouse, 2) | (HandType::FullHouse, 3) => HandType::FiveOfAKind,
            (HandType::FullHouse, _) => HandType::FullHouse,
            (HandType::ThreeOfAKind, 3) => HandType::FiveOfAKind,
            (HandType::ThreeOfAKind, 1) => HandType::FourOfAKind,
            (HandType::ThreeOfAKind, _) => HandType::ThreeOfAKind,
            (HandType::TwoPairs, 2) => HandType::FourOfAKind,
            (HandType::TwoPairs, 1) => HandType::FullHouse,
            (HandType::TwoPairs, _) => HandType::TwoPairs,
            (HandType::OnePair, 2) | (HandType::OnePair, 1) => HandType::ThreeOfAKind,
            (HandType::OnePair, _) => HandType::OnePair,
            (HandType::HighCard, 1) => HandType::OnePair,
            (HandType::HighCard, _) => HandType::HighCard,
        }
    }
}

impl PartialOrd<Self> for JokerHand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JokerHand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hand_type()
            .cmp(&other.hand_type())
            .reverse()
            .then(self.cards.cmp(&other.cards))
    }
}

impl PartialOrd<Self> for JokerCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JokerCard {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value().cmp(&other.value())
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, input) = parse_input(input).unwrap();
    let mut hands = input
        .into_iter()
        .map(|(hand, bid)| {
            let cards = hand
                .chars()
                .map(Card)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            (Hand { cards }, bid)
        })
        .collect::<Vec<_>>();

    hands.sort();

    hands
        .into_iter()
        .enumerate()
        .map(|(rank, (_, bid))| bid * (rank as u32 + 1))
        .sum1()
}

pub fn part_two(input: &str) -> Option<u32> {
    let (_, input) = parse_input(input).unwrap();
    let mut hands = input
        .into_iter()
        .map(|(hand, bid)| {
            let cards = hand
                .chars()
                .map(JokerCard)
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            (JokerHand { cards }, bid)
        })
        .collect::<Vec<_>>();

    hands.sort();

    hands
        .into_iter()
        .enumerate()
        .map(|(rank, (_, bid))| bid * (rank as u32 + 1))
        .sum1()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(6440));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(5905));
    }
}
