use crate::util::Indexer;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}

impl Coord {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone)]
pub struct CoordIndexer {
    pub width: usize,
    pub height: usize,
}

impl CoordIndexer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl Indexer<Coord> for CoordIndexer {
    fn len(&self) -> usize {
        self.width * self.height
    }

    fn index_for(&self, coord: &Coord) -> usize {
        coord.y * self.width + coord.x
    }
}

impl CoordIndexer {
    /// Returns the coordinate one step in the given direction from the given coordinate, if it is in bounds.
    pub fn step(&self, coord: Coord, direction: Direction) -> Option<Coord> {
        let Coord { x, y } = coord;
        match direction {
            Direction::Up if y > 0 => Some(Coord { x, y: y - 1 }),
            Direction::Right if x + 1 < self.width => Some(Coord { x: x + 1, y }),
            Direction::Down if y + 1 < self.height => Some(Coord { x, y: y + 1 }),
            Direction::Left if x > 0 => Some(Coord { x: x - 1, y }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectedCoord {
    pub coord: Coord,
    pub direction: Direction,
}

impl DirectedCoord {
    pub fn new(x: usize, y: usize, direction: Direction) -> Self {
        Self {
            coord: Coord::new(x, y),
            direction,
        }
    }
}

pub struct DirectedCoordIndexer {
    pub width: usize,
    pub height: usize,
}

impl DirectedCoordIndexer {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height }
    }
}

impl From<CoordIndexer> for DirectedCoordIndexer {
    fn from(coord_indexer: CoordIndexer) -> Self {
        Self {
            width: coord_indexer.width,
            height: coord_indexer.height,
        }
    }
}

impl Indexer<DirectedCoord> for DirectedCoordIndexer {
    fn len(&self) -> usize {
        self.width * self.height * 4
    }

    fn index_for(&self, directed_coord: &DirectedCoord) -> usize {
        let DirectedCoord {
            coord: Coord { x, y },
            direction,
        } = *directed_coord;
        let direction_index = match direction {
            Direction::Up => 0,
            Direction::Right => 1,
            Direction::Down => 2,
            Direction::Left => 3,
        };
        (y * self.width + x) * 4 + direction_index
    }
}
