use crate::util::Indexer;
use std::marker::PhantomData;

// TODO: Rename to North, East, South, West
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

pub struct Up;
pub struct Right;
pub struct Down;
pub struct Left;

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Right => Direction::Left,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
        }
    }
    
    pub fn orthogonal(self) -> [Self; 2] {
        match self {
            Direction::Up | Direction::Down => [Direction::Left, Direction::Right],
            Direction::Right | Direction::Left => [Direction::Up, Direction::Down],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Coord<T = usize> {
    pub x: T,
    pub y: T,
}

impl<T> Coord<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CoordIndexer<T = usize> {
    pub width: T,
    pub height: T,
}

impl<T> CoordIndexer<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl Indexer<Coord<usize>> for CoordIndexer<usize> {
    fn len(&self) -> usize {
        self.width * self.height
    }

    fn index_for(&self, coord: &Coord) -> usize {
        coord.y * self.width + coord.x
    }
}

impl Indexer<Coord<u32>> for CoordIndexer<u32> {
    fn len(&self) -> usize {
        (self.width * self.height) as usize
    }

    fn index_for(&self, coord: &Coord<u32>) -> usize {
        (coord.y * self.width + coord.x) as usize
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

#[derive(Copy, Clone)]
pub struct FlippedCoordIndexer<D> {
    indexer: CoordIndexer,
    _direction: PhantomData<D>,
}

impl<D> FlippedCoordIndexer<D> {
    pub fn new(indexer: CoordIndexer) -> Self {
        Self {
            indexer,
            _direction: PhantomData,
        }
    }

    pub fn width(&self) -> usize {
        self.indexer.width
    }

    pub fn height(&self) -> usize {
        self.indexer.height
    }
}

impl Indexer<Coord> for FlippedCoordIndexer<Up> {
    fn len(&self) -> usize {
        self.indexer.len()
    }

    fn index_for(&self, coord: &Coord) -> usize {
        self.indexer.index_for(coord)
    }
}

impl Indexer<Coord> for FlippedCoordIndexer<Right> {
    fn len(&self) -> usize {
        self.indexer.len()
    }

    fn index_for(&self, coord: &Coord) -> usize {
        let Coord { x, y } = *coord;
        self.indexer.index_for(&Coord {
            x: self.indexer.width - 1 - y,
            y: x,
        })
    }
}

impl Indexer<Coord> for FlippedCoordIndexer<Down> {
    fn len(&self) -> usize {
        self.indexer.len()
    }

    fn index_for(&self, coord: &Coord) -> usize {
        let Coord { x, y } = *coord;
        self.indexer.index_for(&Coord {
            x,
            y: self.indexer.height - 1 - y,
        })
    }
}

impl Indexer<Coord> for FlippedCoordIndexer<Left> {
    fn len(&self) -> usize {
        self.indexer.len()
    }

    fn index_for(&self, coord: &Coord) -> usize {
        let Coord { x, y } = *coord;
        self.indexer.index_for(&Coord { x: y, y: x })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DirectedCoord<T = usize> {
    pub coord: Coord<T>,
    pub direction: Direction,
}

impl<T> DirectedCoord<T> {
    pub fn new(x: T, y: T, direction: Direction) -> Self {
        Self {
            coord: Coord::new(x, y),
            direction,
        }
    }
}

#[derive(Copy, Clone)]
pub struct DirectedCoordIndexer<T = usize> {
    pub width: T,
    pub height: T,
}

impl<T> DirectedCoordIndexer<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl<T> From<CoordIndexer<T>> for DirectedCoordIndexer<T> {
    fn from(coord_indexer: CoordIndexer<T>) -> Self {
        Self {
            width: coord_indexer.width,
            height: coord_indexer.height,
        }
    }
}

impl Indexer<DirectedCoord<usize>> for DirectedCoordIndexer<usize> {
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

impl Indexer<DirectedCoord<u32>> for DirectedCoordIndexer<u32> {
    fn len(&self) -> usize {
        (self.width * self.height * 4) as usize
    }

    fn index_for(&self, directed_coord: &DirectedCoord<u32>) -> usize {
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
        ((y * self.width + x) * 4 + direction_index) as usize
    }
}
