#[allow(clippy::len_without_is_empty)]
pub trait Indexer<K> {
    /// Returns the required length of the data vector.
    fn len(&self) -> usize;

    /// Returns the index of the key in the data vector.
    /// This index is guaranteed to be in bounds (i.e. less than `len()`).
    fn index_for(&self, key: &K) -> usize;
}

#[derive(Debug, Clone, Copy)]
pub struct LinearIndexer<T = usize> {
    len: T,
}

impl<T> LinearIndexer<T> {
    pub fn new(len: T) -> Self {
        Self { len }
    }
}

impl Indexer<usize> for LinearIndexer<usize> {
    #[inline]
    fn len(&self) -> usize {
        self.len
    }

    #[inline]
    fn index_for(&self, key: &usize) -> usize {
        *key
    }
}

impl Indexer<u32> for LinearIndexer<u32> {
    fn len(&self) -> usize {
        self.len as usize
    }

    fn index_for(&self, key: &u32) -> usize {
        *key as usize
    }
}
