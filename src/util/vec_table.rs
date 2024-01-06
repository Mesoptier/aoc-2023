use crate::util::indexer::Indexer;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct VecTable<K, V, I> {
    data: Vec<V>,
    indexer: I,
    _phantom: PhantomData<K>,
}

impl<K, V, I> VecTable<K, V, I>
where
    I: Indexer<K>,
    V: Default,
{
    pub fn new(indexer: I) -> Self {
        let mut data = Vec::with_capacity(indexer.len());
        data.resize_with(indexer.len(), Default::default);
        Self {
            data,
            indexer,
            _phantom: PhantomData,
        }
    }
}

impl<K, V, I> VecTable<K, V, I>
where
    V: Clone,
    I: Indexer<K>,
{
    /// Creates a new `VecTable` with the given default value and indexer.
    pub fn with_default(default: V, indexer: I) -> Self {
        let mut data = Vec::with_capacity(indexer.len());
        data.resize_with(indexer.len(), || default.clone());
        Self {
            data,
            indexer,
            _phantom: PhantomData,
        }
    }
}

impl<K, V, I> VecTable<K, V, I>
where
    I: Indexer<K>,
{
    /// Creates a new `VecTable` from the given data and indexer.
    pub fn from_vec(data: Vec<V>, indexer: I) -> Self {
        assert_eq!(data.len(), indexer.len());
        Self {
            data,
            indexer,
            _phantom: PhantomData,
        }
    }

    /// Returns a reference to the value associated with the given key.
    pub fn get(&self, key: &K) -> &V {
        let index = self.indexer.index_for(key);
        unsafe {
            // SAFETY: `index` is guaranteed to be in bounds.
            self.data.get_unchecked(index)
        }
    }

    /// Returns a mutable reference to the value associated with the given key.
    pub fn get_mut(&mut self, key: &K) -> &mut V {
        let index = self.indexer.index_for(key);
        unsafe {
            // SAFETY: `index` is guaranteed to be in bounds.
            self.data.get_unchecked_mut(index)
        }
    }

    /// Inserts the given value at the given key and returns the previous value.
    pub fn insert(&mut self, key: &K, value: V) -> V {
        std::mem::replace(self.get_mut(key), value)
    }
}

impl<K, V, I> VecTable<K, V, I> {
    /// Returns a reference to the underlying indexer.
    pub fn indexer(&self) -> &I {
        &self.indexer
    }

    /// Returns an iterator over the values in the table.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter()
    }
}

impl<K, V, I> Index<K> for VecTable<K, V, I>
where
    I: Indexer<K>,
{
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(&key)
    }
}

impl<K, V, I> IndexMut<K> for VecTable<K, V, I>
where
    I: Indexer<K>,
{
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(&key)
    }
}
