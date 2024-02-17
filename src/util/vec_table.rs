use crate::util::indexer::Indexer;
use crate::util::KeyFor;
use std::borrow::{Borrow, BorrowMut};
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VecTable<K, V, I, D = Box<[V]>> {
    data: D,
    indexer: I,
    _phantom: PhantomData<(K, V)>,
}

impl<K, V, I> VecTable<K, V, I>
where
    V: Default,
    I: Indexer<K>,
{
    pub fn new(indexer: I) -> Self {
        let mut data = Vec::with_capacity(indexer.len());
        data.resize_with(indexer.len(), Default::default);
        Self {
            data: data.into_boxed_slice(),
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
            data: data.into_boxed_slice(),
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
            data: data.into_boxed_slice(),
            indexer,
            _phantom: PhantomData,
        }
    }

    pub fn to_vec(self) -> Vec<V> {
        self.data.into_vec()
    }
}

impl<K, V, I, D> VecTable<K, V, I, D>
where
    I: Indexer<K>,
    D: Borrow<[V]>,
{
    /// Returns a reference to the value associated with the given key.
    pub fn get(&self, key: &K) -> &V {
        let index = self.indexer.index_for(key);
        unsafe {
            // SAFETY: `index` is guaranteed to be in bounds.
            self.data.borrow().get_unchecked(index)
        }
    }

    /// Returns an iterator over the values in the table.
    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.borrow().iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = K> + '_
    where
        I: KeyFor<K>,
    {
        self.indexer.iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)>
    where
        I: KeyFor<K>,
    {
        self.keys().zip(self.values())
    }

    pub fn view<J: Indexer<K>>(&self, indexer: J) -> VecTable<K, V, J, &[V]> {
        assert_eq!(self.indexer.len(), indexer.len());
        VecTable {
            data: self.data.borrow(),
            indexer,
            _phantom: PhantomData,
        }
    }
}

impl<K, V, I, D> VecTable<K, V, I, D>
where
    I: Indexer<K>,
    D: BorrowMut<[V]>,
{
    /// Returns a mutable reference to the value associated with the given key.
    pub fn get_mut(&mut self, key: &K) -> &mut V {
        let index = self.indexer.index_for(key);
        unsafe {
            // SAFETY: `index` is guaranteed to be in bounds.
            self.data.borrow_mut().get_unchecked_mut(index)
        }
    }

    /// Returns an iterator over the values in the table.
    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.borrow_mut().iter_mut()
    }

    /// Inserts the given value at the given key and returns the previous value.
    pub fn insert(&mut self, key: &K, value: V) -> V {
        std::mem::replace(self.get_mut(key), value)
    }

    pub fn view_mut<J: Indexer<K>>(&mut self, indexer: J) -> VecTable<K, V, J, &mut [V]> {
        assert_eq!(self.indexer.len(), indexer.len());
        VecTable {
            data: self.data.borrow_mut(),
            indexer,
            _phantom: PhantomData,
        }
    }
}

impl<K, V, I, D> VecTable<K, V, I, D> {
    /// Returns a reference to the underlying indexer.
    pub fn indexer(&self) -> &I {
        &self.indexer
    }
}

impl<K, V, I, D> Index<K> for VecTable<K, V, I, D>
where
    I: Indexer<K>,
    D: Borrow<[V]>,
{
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(&key)
    }
}

impl<K, V, I, D> IndexMut<K> for VecTable<K, V, I, D>
where
    I: Indexer<K>,
    D: BorrowMut<[V]>,
{
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(&key)
    }
}
