use std::cmp::Ordering;

use itertools::Itertools;

advent_of_code::solution!(7);

#[derive(Copy, Clone, Eq, PartialEq)]
enum JCardType {
    Jack,
    Joker,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Ord, PartialOrd)]
struct Card(u32);

impl Card {
    fn new(c: char, j_card_type: JCardType) -> Self {
        Self(match c {
            'A' => 14,
            'K' => 13,
            'Q' => 12,
            'J' => match j_card_type {
                JCardType::Jack => 11,
                JCardType::Joker => 1,
            },
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
        })
    }

    fn is_joker(&self) -> bool {
        self.0 == 1
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
    hand_type: HandType,
}

impl Hand {
    fn new(cards: [Card; 5], j_card_type: JCardType) -> Self {
        let sorted_cards = {
            let mut cards = cards;
            cards.sort_unstable();
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

        let hand_type = match j_card_type {
            JCardType::Jack => hand_type,
            JCardType::Joker => {
                let num_jokers = sorted_cards
                    .into_iter()
                    .filter(|card| card.is_joker())
                    .count();

                match (hand_type, num_jokers) {
                    (HandType::FiveOfAKind, _) => HandType::FiveOfAKind,
                    (HandType::FourOfAKind, 1) | (HandType::FourOfAKind, 4) => {
                        HandType::FiveOfAKind
                    }
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
        };

        Self { cards, hand_type }
    }
}

impl PartialOrd<Self> for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> Ordering {
        self.hand_type
            .cmp(&other.hand_type)
            .reverse()
            .then(self.cards.cmp(&other.cards))
    }
}

fn parse_input_iter(input: &str) -> impl Iterator<Item = ([char; 5], u32)> + '_ {
    input.lines().map(|line| {
        let cards: [u8; 5] = line.as_bytes()[..5].try_into().unwrap();
        let cards = cards.map(|c| c as char);
        let bid = line[6..].parse().unwrap();
        (cards, bid)
    })
}

fn solve(input: &str, j_card_type: JCardType) -> Option<u32> {
    let mut hands = parse_input_iter(input)
        .map(|(cards, bid)| {
            let cards = cards.map(|c| Card::new(c, j_card_type));
            (Hand::new(cards, j_card_type), bid)
        })
        .collect::<Vec<_>>();

    hands.sort_unstable();

    hands
        .into_iter()
        .enumerate()
        .map(|(rank, (_, bid))| bid * (rank as u32 + 1))
        .sum1()
}

pub fn part_one(input: &str) -> Option<u32> {
    solve(input, JCardType::Jack)
}

pub fn part_two(input: &str) -> Option<u32> {
    solve(input, JCardType::Joker)
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
