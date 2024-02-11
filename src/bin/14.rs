use std::collections::HashMap;
use std::hash::Hash;

advent_of_code::solution!(14);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct BitMatrix {
    data: [u128; 128],
}

impl BitMatrix {
    fn new() -> Self {
        Self { data: [0; 128] }
    }

    fn get(&self, i: usize, j: usize) -> bool {
        (self.data[i] >> j) & 1 == 1
    }

    fn set(&mut self, i: usize, j: usize) {
        self.data[i] |= 1 << j;
    }

    fn clear(&mut self, i: usize, j: usize) {
        self.data[i] &= !(1 << j);
    }

    fn rotate_right(&self, dim: usize) -> Self {
        // TODO: Optimize algorithm (SIMD, in-place, etc.)
        let mut result = Self::new();
        for i in 0..dim {
            for j in 0..dim {
                if self.get(i, j) {
                    result.set(j, dim - i - 1);
                }
            }
        }
        result
    }
}

struct Field {
    dim: usize,
    rocks: BitMatrix,
    blocks: BitMatrix,
}

impl Field {
    fn from_input(input: &str) -> Self {
        let (lines, dim) = {
            let mut lines = input.lines().peekable();
            let dim = lines.peek().unwrap().len();
            (lines, dim)
        };

        let mut rocks = BitMatrix::new();
        let mut blocks = BitMatrix::new();

        for (i, line) in lines.enumerate() {
            for (j, c) in line.chars().enumerate() {
                match c {
                    '.' => {}
                    'O' => rocks.set(i, j),
                    '#' => blocks.set(i, j),
                    _ => unreachable!(),
                }
            }
        }

        Self { dim, rocks, blocks }
    }

    fn rotate_right(&mut self) {
        self.rocks = self.rocks.rotate_right(self.dim);
        self.blocks = self.blocks.rotate_right(self.dim); // TODO: Can be cached
    }

    fn roll_up(&mut self) {
        for i in 1..self.dim {
            let mut rolling_rocks = self.rocks.data[i];
            self.rocks.data[i] = 0;

            let mut j = i;
            while rolling_rocks != 0 && j != 0 {
                let is_blocked = self.rocks.data[j - 1] | self.blocks.data[j - 1];
                self.rocks.data[j] |= rolling_rocks & is_blocked;
                rolling_rocks &= !is_blocked;
                j -= 1;
            }
            self.rocks.data[0] |= rolling_rocks;
        }
    }

    fn total_load(&self) -> u32 {
        let mut total_load = 0;
        for i in 0..self.dim {
            total_load += (self.dim - i) as u32 * self.rocks.data[i].count_ones();
        }
        total_load
    }

    fn total_load_reverse(&self) -> u32 {
        let mut total_load = 0;
        for i in 0..self.dim {
            total_load += (i + 1) as u32 * self.rocks.data[i].count_ones();
        }
        total_load
    }

    fn cycle(&mut self) -> u32 {
        self.roll_up(); // rolled north
        self.rotate_right();
        self.roll_up(); // rolled west
        self.rotate_right();
        self.roll_up(); // rolled south
        let result = self.total_load_reverse();
        self.rotate_right();
        self.roll_up(); // rolled east
        self.rotate_right();
        result
    }
}

pub fn part_one(input: &str) -> Option<u32> {
    let mut field = Field::from_input(input);
    field.roll_up();
    Some(field.total_load())
}

pub fn part_two(input: &str) -> Option<u32> {
    let mut field = Field::from_input(input);
    let mut cycles = 0;

    let mut cache = HashMap::<BitMatrix, usize>::new();
    let mut total_loads = vec![];

    loop {
        let total_load = field.cycle();
        cycles += 1;

        if let Some(prev_cycles) = cache.insert(field.rocks, cycles) {
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
