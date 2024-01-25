#[allow(clippy::len_without_is_empty)]
pub trait Indexer<K> {
    /// Returns the required length of the data vector.
    fn len(&self) -> usize;

    /// Returns the index of the key in the data vector.
    /// This index is guaranteed to be in bounds (i.e. less than `len()`).
    fn index_for(&self, key: &K) -> usize;
}

pub trait KeyFor<K>: Indexer<K> {
    /// Returns the key for the given index.
    /// This index is guaranteed to be in bounds (i.e. less than `len()`).
    fn key_for(&self, index: usize) -> K;

    /// Returns an iterator over the keys supported by this indexer.
    fn iter(&self) -> impl Iterator<Item = K> {
        (0..self.len()).map(move |index| self.key_for(index))
    }
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

macro_rules! impl_linear_indexer {
    ($($ty:ty),*) => {
        $(
            impl Indexer<$ty> for LinearIndexer<$ty> {
                #[inline]
                fn len(&self) -> usize {
                    self.len as usize
                }

                #[inline]
                fn index_for(&self, key: &$ty) -> usize {
                    *key as usize
                }
            }

            impl KeyFor<$ty> for LinearIndexer<$ty> {
                #[inline]
                fn key_for(&self, index: usize) -> $ty {
                    index as $ty
                }
            }
        )*
    };
    () => {};
}

impl_linear_indexer!(u8, u16, u32, u64, u128, usize);
