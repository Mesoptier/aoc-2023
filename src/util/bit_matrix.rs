use elain::{Align, Alignment};
use std::simd::{LaneCount, Simd, SupportedLaneCount};

/// A 2D matrix of bits, with a fixed size of N x N bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BitMatrix<const N: usize>
where
    Align<N>: Alignment,
{
    _align: Align<N>,
    data: [[[u8; N]; 8]; N],
}

impl<const N: usize> BitMatrix<N>
where
    Align<N>: Alignment,
{
    pub fn new() -> Self {
        Self {
            _align: Align::NEW,
            data: [[[0; N]; 8]; N],
        }
    }

    /// Get the value of the bit at position (i, j).
    pub fn get(&self, i: usize, j: usize) -> bool {
        self.data[i / 8][i % 8][j / 8] >> (7 - j % 8) & 1 == 1
    }

    /// Set the value of the bit at position (i, j) to 1.
    pub fn set(&mut self, i: usize, j: usize) {
        self.data[i / 8][i % 8][j / 8] |= 1 << (7 - j % 8);
    }

    /// Set the value of the bit at position (i, j) to 0.
    pub fn clear(&mut self, i: usize, j: usize) {
        self.data[i / 8][i % 8][j / 8] &= !(1 << (7 - j % 8));
    }

    /// Get the block at position (bi, bj).
    fn get_block(&self, bi: usize, bj: usize) -> u64 {
        let mut x = 0;
        for k in 0..8 {
            x |= (self.data[bi][k][bj] as u64) << (8 * (7 - k));
        }
        x
    }

    /// Get the block at position (bi, bj), with the rows reflected.
    fn get_block_reflected(&self, bi: usize, bj: usize) -> u64 {
        let mut x = 0;
        for k in 0..8 {
            x |= (self.data[bi][k][bj] as u64) << (8 * k);
        }
        x
    }

    /// Set the block at position (bi, bj).
    fn set_block(&mut self, bi: usize, bj: usize, mut block: u64) {
        for k in (0..8).rev() {
            self.data[bi][k][bj] = block as u8;
            block >>= 8;
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

    pub fn transpose(&self) -> Self {
        let mut result = Self::new();

        for i in 0..N {
            for j in 0..N {
                let block = self.get_block(i, j);
                let block = Self::transpose_block(block);
                result.set_block(j, i, block);
            }
        }

        result
    }

    pub fn rotate_right(&self) -> Self {
        let mut result = Self::new();

        for i in 0..N {
            for j in 0..N {
                let block = self.get_block_reflected(i, j);
                let transposed = Self::transpose_block(block);
                result.set_block(j, N - 1 - i, transposed);
            }
        }

        result
    }

    pub fn bytes(&self) -> &[u8] {
        unsafe {
            let (prefix, bytes, suffix) = self.data.align_to();
            assert!(prefix.is_empty());
            assert!(suffix.is_empty());
            bytes
        }
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            let (prefix, bytes, suffix) = self.data.align_to_mut();
            assert!(prefix.is_empty());
            assert!(suffix.is_empty());
            bytes
        }
    }

    pub fn rows(&self) -> &[[u8; N]] {
        unsafe {
            let (prefix, rows, suffix) = self.data.align_to();
            assert!(prefix.is_empty());
            assert!(suffix.is_empty());
            rows
        }
    }

    pub fn rows_mut(&mut self) -> &mut [[u8; N]] {
        unsafe {
            let (prefix, rows, suffix) = self.data.align_to_mut();
            assert!(prefix.is_empty());
            assert!(suffix.is_empty());
            rows
        }
    }
}

impl<const N: usize> BitMatrix<N>
where
    Align<N>: Alignment,
    LaneCount<N>: SupportedLaneCount,
{
    pub fn rows_simd(&self) -> &[Simd<u8, N>] {
        let (prefix, rows, suffix) = self.bytes().as_simd();
        assert!(prefix.is_empty());
        assert!(suffix.is_empty());
        rows
    }

    pub fn rows_simd_mut(&mut self) -> &mut [Simd<u8, N>] {
        let (prefix, rows, suffix) = self.bytes_mut().as_simd_mut();
        assert!(prefix.is_empty());
        assert!(suffix.is_empty());
        rows
    }
}

impl<const N: usize> Default for BitMatrix<N>
where
    Align<N>: Alignment,
{
    fn default() -> Self {
        Self::new()
    }
}
