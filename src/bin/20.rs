use std::collections::{HashMap, VecDeque};
use std::fmt::Display;

use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, line_ending};
use nom::combinator::map;
use nom::multi::separated_list1;
use nom::sequence::{preceded, separated_pair};
use nom::IResult;

advent_of_code::solution!(20);

#[derive(Copy, Clone, Debug)]
enum ModuleType {
    Broadcast,
    FlipFlop,
    Conjunction,
}

#[derive(Debug)]
struct ModuleSpec {
    label: String,
    module_type: ModuleType,
    destinations: Vec<String>,
}

fn parse_input(input: &str) -> IResult<&str, Vec<ModuleSpec>> {
    separated_list1(
        line_ending,
        map(
            separated_pair(
                alt((
                    map(tag("broadcaster"), |label| (label, ModuleType::Broadcast)),
                    map(preceded(tag("%"), alpha1), |label| {
                        (label, ModuleType::FlipFlop)
                    }),
                    map(preceded(tag("&"), alpha1), |label| {
                        (label, ModuleType::Conjunction)
                    }),
                )),
                tag(" -> "),
                separated_list1(tag(", "), map(alpha1, str::to_string)),
            ),
            |((label, module_type), destinations)| ModuleSpec {
                label: label.to_string(),
                module_type,
                destinations,
            },
        ),
    )(input)
}

#[derive(Clone)]
struct Pulse {
    source: String,
    destination: String,
    is_high: bool,
}

impl Display for Pulse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} -{}> {}",
            self.source,
            if self.is_high { "high" } else { "low" },
            self.destination,
        )
    }
}

trait Module {
    fn receive_pulse(&mut self, pulse: Pulse) -> Vec<Pulse>;
}

struct BroadcastModule {
    label: String,
    destinations: Vec<String>,
}

impl BroadcastModule {
    fn new(label: String, destinations: Vec<String>) -> Self {
        Self {
            label,
            destinations,
        }
    }
}

impl Module for BroadcastModule {
    fn receive_pulse(&mut self, pulse: Pulse) -> Vec<Pulse> {
        self.destinations
            .iter()
            .map(|destination| Pulse {
                source: self.label.clone(),
                destination: destination.clone(),
                is_high: pulse.is_high,
            })
            .collect()
    }
}

struct FlipFlopModule {
    label: String,
    destinations: Vec<String>,
    is_high: bool,
}

impl FlipFlopModule {
    fn new(label: String, destinations: Vec<String>) -> Self {
        Self {
            label,
            destinations,
            is_high: false,
        }
    }
}

impl Module for FlipFlopModule {
    fn receive_pulse(&mut self, pulse: Pulse) -> Vec<Pulse> {
        if pulse.is_high {
            return vec![];
        }

        self.is_high = !self.is_high;
        self.destinations
            .iter()
            .map(|destination| Pulse {
                source: self.label.clone(),
                destination: destination.clone(),
                is_high: self.is_high,
            })
            .collect()
    }
}

struct ConjunctionModule {
    label: String,
    destinations: Vec<String>,
    inputs: HashMap<String, bool>,
}

impl ConjunctionModule {
    fn new(label: String, destinations: Vec<String>, inputs: Vec<String>) -> Self {
        Self {
            label,
            destinations,
            inputs: inputs.into_iter().map(|input| (input, false)).collect(),
        }
    }
}

impl Module for ConjunctionModule {
    fn receive_pulse(&mut self, pulse: Pulse) -> Vec<Pulse> {
        *self.inputs.get_mut(&pulse.source).unwrap() = pulse.is_high;

        let is_high = self.inputs.values().all(|&is_high| is_high);
        self.destinations
            .iter()
            .map(|destination| Pulse {
                source: self.label.clone(),
                destination: destination.clone(),
                is_high: !is_high,
            })
            .collect()
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let (_, module_specs) = parse_input(input).unwrap();

    let reverse_adjacency_list: HashMap<String, Vec<String>> = module_specs
        .iter()
        .flat_map(|module_spec| {
            module_spec
                .destinations
                .iter()
                .map(move |destination| (destination.clone(), module_spec.label.clone()))
        })
        .fold(HashMap::new(), |mut map, (destination, source)| {
            map.entry(destination).or_default().push(source);
            map
        });

    // TODO: Use enum dispatch instead of Box<dyn Module>
    let mut modules = HashMap::<String, Box<dyn Module>>::from_iter(module_specs.iter().map(
        |module_spec| -> (String, Box<dyn Module>) {
            (
                module_spec.label.clone(),
                match module_spec.module_type {
                    ModuleType::Broadcast => Box::new(BroadcastModule::new(
                        module_spec.label.clone(),
                        module_spec.destinations.clone(),
                    )),
                    ModuleType::FlipFlop => Box::new(FlipFlopModule::new(
                        module_spec.label.clone(),
                        module_spec.destinations.clone(),
                    )),
                    ModuleType::Conjunction => Box::new(ConjunctionModule::new(
                        module_spec.label.clone(),
                        module_spec.destinations.clone(),
                        reverse_adjacency_list[&module_spec.label].clone(),
                    )),
                },
            )
        },
    ));

    let mut queue = VecDeque::new();

    let mut low_pulses_sent = 0;
    let mut high_pulses_sent = 0;

    for _ in 0..1000 {
        queue.push_back(Pulse {
            source: "button".to_string(),
            destination: "broadcaster".to_string(),
            is_high: false,
        });

        while let Some(pulse) = queue.pop_front() {
            match pulse.is_high {
                true => high_pulses_sent += 1,
                false => low_pulses_sent += 1,
            };

            if let Some(module) = modules.get_mut(&pulse.destination) {
                let pulses = module.receive_pulse(pulse);
                queue.extend(pulses);
            }
        }
    }

    Some(low_pulses_sent * high_pulses_sent)
}

pub fn part_two(input: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file_part(
            "examples", DAY, 1,
        ));
        assert_eq!(result, Some(32000000));

        let result = part_one(&advent_of_code::template::read_file_part(
            "examples", DAY, 2,
        ));
        assert_eq!(result, Some(11687500));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
