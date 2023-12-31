use enum_dispatch::enum_dispatch;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Display;

use itertools::Itertools;
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
struct ModuleSpec<T> {
    label: T,
    module_type: ModuleType,
    destinations: Vec<T>,
}

fn parse_input(input: &str) -> IResult<&str, Vec<ModuleSpec<String>>> {
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
    source: usize,
    destination: usize,
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

#[enum_dispatch(PulseReceiver)]
enum Module {
    Broadcast(BroadcastModule),
    FlipFlop(FlipFlopModule),
    Conjunction(ConjunctionModule),
}

#[enum_dispatch]
trait PulseReceiver {
    fn receive_pulse(&mut self, pulse: Pulse, queue: &mut VecDeque<Pulse>);
}

struct BroadcastModule {
    label: usize,
    destinations: Vec<usize>,
}

impl BroadcastModule {
    fn new(label: usize, destinations: Vec<usize>) -> Self {
        Self {
            label,
            destinations,
        }
    }
}

impl PulseReceiver for BroadcastModule {
    fn receive_pulse(&mut self, pulse: Pulse, queue: &mut VecDeque<Pulse>) {
        queue.extend(self.destinations.iter().map(|destination| Pulse {
            source: self.label,
            destination: *destination,
            is_high: pulse.is_high,
        }));
    }
}

struct FlipFlopModule {
    label: usize,
    destinations: Vec<usize>,
    is_high: bool,
}

impl FlipFlopModule {
    fn new(label: usize, destinations: Vec<usize>) -> Self {
        Self {
            label,
            destinations,
            is_high: false,
        }
    }
}

impl PulseReceiver for FlipFlopModule {
    fn receive_pulse(&mut self, pulse: Pulse, queue: &mut VecDeque<Pulse>) {
        if pulse.is_high {
            return;
        }

        self.is_high = !self.is_high;
        queue.extend(self.destinations.iter().map(|destination| Pulse {
            source: self.label,
            destination: *destination,
            is_high: self.is_high,
        }));
    }
}

struct ConjunctionModule {
    label: usize,
    destinations: Vec<usize>,
    /// A bit vector where each bit represents whether the last pulse from that module was high or low. This assumes
    /// that the total number of modules is less than the number of bits in usize.
    inputs: usize,
}

impl ConjunctionModule {
    fn new(label: usize, destinations: Vec<usize>, inputs: Vec<usize>) -> Self {
        Self {
            label,
            destinations,
            inputs: {
                let mut values = usize::MAX;
                for idx in inputs {
                    // Unset the bit at idx
                    values &= !(1 << idx);
                }
                values
            },
        }
    }
}

impl PulseReceiver for ConjunctionModule {
    fn receive_pulse(&mut self, pulse: Pulse, queue: &mut VecDeque<Pulse>) {
        if pulse.is_high {
            self.inputs |= 1 << pulse.source;
        } else {
            self.inputs &= !(1 << pulse.source);
        }

        let is_high = self.inputs == usize::MAX;
        queue.extend(self.destinations.iter().map(|destination| Pulse {
            source: self.label,
            destination: *destination,
            is_high: !is_high,
        }))
    }
}

fn initialize_modules(input: &str) -> (HashMap<String, usize>, Vec<Module>) {
    let (_, module_specs) = parse_input(input).unwrap();

    let source_labels = module_specs
        .iter()
        .map(|spec| spec.label.clone())
        .collect::<HashSet<_>>();
    let destination_labels = module_specs
        .iter()
        .flat_map(|spec| spec.destinations.clone())
        .collect::<HashSet<_>>();
    let destination_labels = destination_labels
        .difference(&source_labels)
        .cloned()
        .collect_vec();
    let source_labels = source_labels.iter().cloned().collect_vec();

    let label_to_id = HashMap::<String, usize>::from_iter(
        source_labels
            .iter()
            .chain(destination_labels.iter())
            .enumerate()
            .map(|(id, label)| (label.clone(), id)),
    );

    let mut module_specs = module_specs
        .into_iter()
        .map(|spec| {
            let id = label_to_id[&spec.label];
            let destinations = spec
                .destinations
                .into_iter()
                .map(|destination| label_to_id[&destination])
                .collect();
            ModuleSpec {
                label: id,
                module_type: spec.module_type,
                destinations,
            }
        })
        .collect_vec();

    let reverse_adjacency_list: Vec<Vec<usize>> = module_specs
        .iter()
        .flat_map(|module_spec| {
            module_spec
                .destinations
                .iter()
                .map(move |destination| (*destination, module_spec.label))
        })
        .fold(
            vec![vec![]; label_to_id.len()],
            |mut map, (destination, source)| {
                map[destination].push(source);
                map
            },
        );

    module_specs.sort_by_key(|spec| spec.label);
    let modules = module_specs
        .into_iter()
        .map(|module_spec| -> Module {
            match module_spec.module_type {
                ModuleType::Broadcast => {
                    BroadcastModule::new(module_spec.label, module_spec.destinations.to_vec())
                        .into()
                }
                ModuleType::FlipFlop => {
                    FlipFlopModule::new(module_spec.label, module_spec.destinations.to_vec()).into()
                }
                ModuleType::Conjunction => ConjunctionModule::new(
                    module_spec.label,
                    module_spec.destinations.to_vec(),
                    reverse_adjacency_list[module_spec.label].clone(),
                )
                .into(),
            }
        })
        .collect_vec();

    (label_to_id, modules)
}

pub fn part_one(input: &str) -> Option<u32> {
    let (label_to_id, mut modules) = initialize_modules(input);

    let mut queue = VecDeque::new();

    let mut low_pulses_sent = 0;
    let mut high_pulses_sent = 0;

    let broadcaster_id = label_to_id["broadcaster"];

    for _ in 0..1000 {
        queue.push_back(Pulse {
            source: usize::MAX,
            destination: broadcaster_id,
            is_high: false,
        });

        while let Some(pulse) = queue.pop_front() {
            match pulse.is_high {
                true => high_pulses_sent += 1,
                false => low_pulses_sent += 1,
            };

            if let Some(module) = modules.get_mut(pulse.destination) {
                module.receive_pulse(pulse, &mut queue);
            }
        }
    }

    Some(low_pulses_sent * high_pulses_sent)
}

pub fn part_two(input: &str) -> Option<u32> {
    let (label_to_id, mut modules) = initialize_modules(input);

    let mut queue = VecDeque::new();

    let mut button_presses = 0;

    let broadcaster_id = label_to_id["broadcaster"];
    let rx_id = label_to_id["rx"];

    loop {
        queue.push_back(Pulse {
            source: usize::MAX,
            destination: broadcaster_id,
            is_high: false,
        });
        button_presses += 1;

        if button_presses % 1_000_000 == 0 {
            println!("{} button presses", button_presses);
        }

        while let Some(pulse) = queue.pop_front() {
            if pulse.destination == rx_id && !pulse.is_high {
                return Some(button_presses);
            }

            if let Some(module) = modules.get_mut(pulse.destination) {
                module.receive_pulse(pulse, &mut queue);
            }
        }
    }
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
