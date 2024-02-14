#![feature(portable_simd)]

use ahash::AHashMap;
use std::simd::prelude::*;

advent_of_code::solution!(14);

type BitMatrix = advent_of_code::util::BitMatrix<16>;

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

    /// The index of the first row of the field in the bit matrix.
    ///
    /// This is needed since the bit matrix is 128x128, but the field is only dim x dim.
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

        let rocks_rows = self.rocks.rows_simd_mut();
        let blocks_rows = self.blocks_per_rotation[self.rotation].rows_simd();

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
        let rocks_rows = self.rocks.rows();

        rocks_rows
            .iter()
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
}

type FieldCacheKey = [u8; 16 * 100];

impl Field {
    fn cache_key(&self) -> FieldCacheKey {
        let mut key = [0; 16 * 100];
        let start = self.start_i() * 16;
        let len = self.dim * 16;
        key[..len].copy_from_slice(&self.rocks.bytes()[start..start + len]);
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

    let mut cache = AHashMap::<FieldCacheKey, usize>::new();
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
