use itertools::Itertools;

pub struct CharGrid<'a> {
    data: &'a [u8],
    width: usize,
    width_with_nl: usize,
    height: usize,
}

impl<'a> CharGrid<'a> {
    pub fn new(data: &'a str) -> Self {
        let data = data.as_bytes();

        let (width, line_sep_char) = data
            .iter()
            .find_position(|&c| matches!(c, b'\n' | b'\r'))
            .unwrap();

        let width_with_nl = width
            + match line_sep_char {
                b'\n' => 1,
                b'\r' => 2,
                _ => unreachable!(),
            };

        // Note: we allow the last line to not have a newline, hence the ceiling division
        let height = (data.len() + width_with_nl - 1) / width_with_nl;

        debug_assert!(
            data.len() == height * width_with_nl
                || data.len() == height * width_with_nl - width_with_nl + width,
            "data must be rectangular (with or without trailing newline)"
        );

        Self {
            data,
            width,
            width_with_nl,
            height,
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> Option<char> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(unsafe {
                // SAFETY: coord is within bounds
                self.get_unchecked(x, y)
            })
        }
    }

    #[inline]
    pub unsafe fn get_unchecked(&self, x: usize, y: usize) -> char {
        *self.data.get_unchecked(y * self.width_with_nl + x) as char
    }
}
