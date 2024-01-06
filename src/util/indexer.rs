#[allow(clippy::len_without_is_empty)]
pub trait Indexer<K> {
    /// Returns the required length of the data vector.
    fn len(&self) -> usize;

    /// Returns the index of the key in the data vector.
    /// This index is guaranteed to be in bounds (i.e. less than `len()`).
    fn index_for(&self, key: &K) -> usize;
}
