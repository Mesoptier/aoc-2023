#![feature(portable_simd)]

use std::collections::HashMap;
use std::hash::Hash;
use std::simd::prelude::*;

advent_of_code::solution!(14);

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
#[repr(align(16))] // Align to 128 bits
struct BitMatrix {
    /// 128 rows of 128 bits (= 16 bytes) each
    data: [u8; 16 * 128],
}

impl BitMatrix {
    fn new() -> Self {
        Self {
            data: [0; 16 * 128],
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

    fn rotate_right(&self) -> Self {
        let mut result = Self::new();

        let input = &self.data;
        let output = &mut result.data;

        for i in 0..16 {
            for j in 0..16 {
                let block = Self::get_block_reflected(input, i, j);
                if block == 0 {
                    continue;
                }

                let transposed = Self::transpose_block(block);
                Self::set_block(output, j, 15 - i, transposed);
            }
        }

        result
    }

    fn get_block_reflected(data: &[u8], i: usize, j: usize) -> u64 {
        let mut x = 0;
        for k in 0..8 {
            x |= (data[16 * (8 * i + k) + j] as u64) << (8 * k);
        }
        x
    }

    fn set_block(data: &mut [u8], i: usize, j: usize, mut x: u64) {
        for k in (0..8).rev() {
            data[16 * (8 * i + k) + j] = x as u8;
            x >>= 8;
        }
    }

    fn transpose_block(block: u64) -> u64 {
        // Based on transpose8rS64 from Hacker's Delight

        let mut x = block;
        let mut t;

        t = (x ^ (x >> 7)) & 0x00AA00AA00AA00AA;
        x = x ^ t ^ (t << 7);
        t = (x ^ (x >> 14)) & 0x0000CCCC0000CCCC;
        x = x ^ t ^ (t << 14);
        t = (x ^ (x >> 28)) & 0x00000000F0F0F0F0;
        x = x ^ t ^ (t << 28);

        x
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

type FieldCacheKey = [u8; 16 * 100];

struct Field {
    dim: usize,
    rotation: usize,
    rocks: BitMatrix,
    blocks_per_rotation: [BitMatrix; 4],
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

        let blocks_0 = blocks;
        let blocks_1 = blocks_0.rotate_right();
        let blocks_2 = blocks_1.rotate_right();
        let blocks_3 = blocks_2.rotate_right();

        Self {
            dim,
            rotation: 0,
            rocks,
            blocks_per_rotation: [blocks_0, blocks_1, blocks_2, blocks_3],
        }
    }

    fn rotate_right(&mut self) {
        self.rotation = (self.rotation + 1) % 4;
        self.rocks = self.rocks.rotate_right();
    }

    fn start_i(&self) -> usize {
        match self.rotation {
            0 => 0,
            1 => 0,
            2 => 128 - self.dim,
            3 => 128 - self.dim,
            _ => unreachable!(),
        }
    }

    fn roll_up(&mut self) {
        let start_i = self.start_i();

        const LANES: usize = 16; // 128 / 8, so each SIMD register holds an entire row of the BitMatrix.
        let (rocks_rows_prefix, rocks_rows, rocks_rows_suffix) =
            self.rocks.data.as_simd_mut::<LANES>();
        let (blocks_rows_prefix, blocks_rows, blocks_rows_suffix) = self.blocks_per_rotation
            [self.rotation]
            .data
            .as_simd::<LANES>();

        // The logic below only works if each entry in the rocks and blocks data spans an entire row of 128 bits.
        // So we need to ensure that the prefix and suffix are empty.
        // TODO: This should always be the case since the BitMatrix is aligned to 128 bits. Can the compiler check this?
        assert_eq!(rocks_rows_prefix.len(), 0);
        assert_eq!(rocks_rows_suffix.len(), 0);
        assert_eq!(blocks_rows_prefix.len(), 0);
        assert_eq!(blocks_rows_suffix.len(), 0);

        for i in (start_i + 1)..start_i + self.dim {
            let mut rolling_rocks = rocks_rows[i];
            rocks_rows[i] = Simd::splat(0);

            let mut j = i;
            while rolling_rocks != Simd::splat(0) && j != start_i {
                let is_blocked = rocks_rows[j - 1] | blocks_rows[j - 1];
                rocks_rows[j] |= rolling_rocks & is_blocked;
                rolling_rocks &= !is_blocked;
                j -= 1;
            }
            rocks_rows[start_i] |= rolling_rocks;
        }
    }

    fn total_load_with<F: Fn(u32, u32) -> u32>(&self, load_factor: F) -> u32 {
        let start_i = self.start_i();
        let rocks_rows = self.rocks.data.chunks_exact(16);

        rocks_rows
            .skip(start_i)
            .take(self.dim)
            .enumerate()
            .map(|(i, row)| {
                row.iter().map(|&x| x.count_ones()).sum::<u32>()
                    * load_factor(i as u32, self.dim as u32)
            })
            .sum()
    }

    /// Compute the total load on the North support beams, assuming the field is facing North.
    fn total_load(&self) -> u32 {
        debug_assert_eq!(self.rotation, 0);
        self.total_load_with(|i, dim| dim - i)
    }

    /// Compute the total load on the North support beams, assuming the field is facing South.
    fn total_load_reverse(&self) -> u32 {
        debug_assert_eq!(self.rotation, 2);
        self.total_load_with(|i, _dim| i + 1)
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

    fn cache_key(&self) -> FieldCacheKey {
        let mut key = [0; 16 * 100];
        let start = self.start_i() * 16;
        let end = start + self.dim * 16;
        key.copy_from_slice(&self.rocks.data[start..end]);
        key
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

    let mut cache = HashMap::<FieldCacheKey, usize>::new();
    let mut total_loads = vec![];

    loop {
        let total_load = field.cycle();
        cycles += 1;

        if let Some(prev_cycles) = cache.insert(field.cache_key(), cycles) {
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
