use crate::util::indexer::Indexer;
use crate::util::vec_table::VecTable;

pub struct VecMap<K, V, I> {
    table: VecTable<K, Option<V>, I>,
}

impl<K, V, I> VecMap<K, V, I>
where
    I: Indexer<K>,
{
    pub fn new(indexer: I) -> Self {
        Self {
            table: VecTable::new(indexer),
        }
    }

    /// Returns a reference to the value associated with the given key.
    pub fn get(&self, key: &K) -> Option<&V> {
        self.table.get(key).as_ref()
    }

    /// Returns a mutable reference to the value associated with the given key.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.table.get_mut(key).as_mut()
    }

    /// Inserts the given value into the map and returns the previous value associated with the key.
    pub fn insert(&mut self, key: &K, value: V) -> Option<V> {
        self.table.insert(key, Some(value))
    }

    /// Removes the value associated with the given key from the map and returns it.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.table.get_mut(key).take()
    }

    pub fn entry(&mut self, key: &K) -> Entry<V> {
        Entry(self.table.get_mut(key))
    }

    /// Returns the number of elements in the map.
    pub fn len(&self) -> usize {
        self.table.values().filter(|v| v.is_some()).count()
    }

    /// Returns `true` if the map contains no elements.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct Entry<'a, V>(&'a mut Option<V>);

impl<'a, V> Entry<'a, V> {
    pub fn or_insert(self, value: V) -> &'a mut V {
        self.0.get_or_insert(value)
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, f: F) -> &'a mut V {
        self.0.get_or_insert_with(f)
    }

    pub fn and_modify<F: FnOnce(&mut V)>(self, f: F) -> &'a mut V {
        if let Some(value) = self.0.as_mut() {
            f(value);
        }
        self.0.as_mut().unwrap()
    }
}
