use bytemuck::{cast_slice, cast_slice_mut};
use std::collections::HashMap;
use std::hash::Hash;

advent_of_code::solution!(14);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct BitMatrix {
    data: [u8; 8 * 16 * 16],
}

impl BitMatrix {
    fn new() -> Self {
        Self {
            data: [0; 8 * 16 * 16],
        }
    }

    fn get(&self, i: usize, j: usize) -> bool {
        let block = 16 * i + (j / 8);
        let bit = 7 - (j % 8);
        (self.data[block] >> bit) & 1 == 1
    }

    fn set(&mut self, i: usize, j: usize) {
        let block = 16 * i + (j / 8);
        let bit = 7 - (j % 8);
        self.data[block] |= 1 << bit;
    }

    fn rotate_right(mut self, dim: usize) -> Self {
        cast_slice_mut::<u8, u128>(&mut self.data)[..dim].reverse();
        self.transpose()
    }

    fn transpose(&self) -> Self {
        let mut result = Self::new();

        let input = &self.data;
        let output = &mut result.data;

        for i in 0..16 {
            for j in 0..16 {
                Self::transpose_block(input, output, i, j);
            }
        }

        result
    }

    fn get_block(data: &[u8], i: usize, j: usize) -> u64 {
        let mut x = 0;
        for k in 0..8 {
            x = x << 8 | (data[16 * (8 * i + k) + j] as u64);
        }
        x
    }

    fn set_block(data: &mut [u8], i: usize, j: usize, mut x: u64) {
        for k in (0..8).rev() {
            data[16 * (8 * i + k) + j] = x as u8;
            x >>= 8;
        }
    }

    fn transpose_block(input: &[u8], output: &mut [u8], i: usize, j: usize) {
        let mut x = Self::get_block(input, i, j);
        let mut t;

        t = (x ^ (x >> 7)) & 0x00AA00AA00AA00AA;
        x = x ^ t ^ (t << 7);
        t = (x ^ (x >> 14)) & 0x0000CCCC0000CCCC;
        x = x ^ t ^ (t << 14);
        t = (x ^ (x >> 28)) & 0x00000000F0F0F0F0;
        x = x ^ t ^ (t << 28);

        // Store x into output block
        Self::set_block(output, j, i, x);
    }

    fn print(&self, dim: usize) {
        for i in 0..dim {
            for j in 0..dim {
                print!("{}", if self.get(i, j) { '1' } else { '.' });
            }
            println!();
        }
        println!();
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
        let rocks_data = cast_slice_mut::<u8, u128>(&mut self.rocks.data);
        let blocks_data = cast_slice::<u8, u128>(&self.blocks.data);

        for i in 1..self.dim {
            let mut rolling_rocks = rocks_data[i];
            rocks_data[i] = 0;

            let mut j = i;
            while rolling_rocks != 0 && j != 0 {
                let is_blocked = rocks_data[j - 1] | blocks_data[j - 1];
                rocks_data[j] |= rolling_rocks & is_blocked;
                rolling_rocks &= !is_blocked;
                j -= 1;
            }
            rocks_data[0] |= rolling_rocks;
        }
    }

    fn total_load(&self) -> u32 {
        let rocks_data = cast_slice::<u8, u128>(&self.rocks.data);
        let mut total_load = 0;
        for i in 0..self.dim {
            total_load += (self.dim - i) as u32 * rocks_data[i].count_ones();
        }
        total_load
    }

    fn total_load_reverse(&self) -> u32 {
        let rocks_data = cast_slice::<u8, u128>(&self.rocks.data);
        let mut total_load = 0;
        for i in 0..self.dim {
            total_load += (i + 1) as u32 * rocks_data[i].count_ones();
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
    fn foo() {
        let dim = 16;
        let mut m = BitMatrix::new();
        m.set(0, 0);
        m.set(2, 12);
        m.print(dim);
        m = m.rotate_right(dim);
        m.print(dim);
    }

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
