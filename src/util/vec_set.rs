use crate::util::indexer::Indexer;
use crate::util::vec_map::VecMap;

pub struct VecSet<V, I> {
    map: VecMap<V, (), I>,
}

impl<V, I> VecSet<V, I>
where
    I: Indexer<V>,
{
    pub fn new(indexer: I) -> Self {
        Self {
            map: VecMap::new(indexer),
        }
    }

    /// Adds a value to the set.
    ///
    /// Returns whether the value was newly inserted.
    pub fn insert(&mut self, value: V) -> bool {
        self.map.insert(&value, ()).is_none()
    }

    /// Returns `true` if the set contains a value.
    pub fn contains(&self, value: &V) -> bool {
        self.map.get(value).is_some()
    }

    /// Returns the number of values in the set.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the set contains no values.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}
