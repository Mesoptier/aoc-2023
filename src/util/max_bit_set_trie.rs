use crate::util::BitSet;
use std::cmp::Ordering;

#[derive(Debug, Default)]
struct Node<B, I, V> {
    /// Union of all bitsets in this node and its descendants.
    bitset: B,

    children: Vec<(I, Node<B, I, V>)>,
    terminal_value: Option<V>,
}

enum InsertResult {
    Inserted,
    Superseded,
}

impl<B, I, V> Node<B, I, V>
where
    B: BitSet<Index = I> + Copy,
    I: Ord + Copy,
    V: Ord + Copy,
{
    fn new_branch(bitset: B, indices: &[I], value: V) -> Self {
        match indices.split_first() {
            None => Self {
                bitset,
                children: Vec::new(),
                terminal_value: Some(value),
            },
            Some((idx, remaining_indices)) => Self {
                bitset,
                children: vec![(*idx, Self::new_branch(bitset, remaining_indices, value))],
                terminal_value: None,
            },
        }
    }

    /// Whether this node or any of its descendants contain a superset of `set` with a value greater than or equal to `value`.
    fn supersedes(&self, bitset: B, indices: &[I], value: V) -> bool {
        if !self.bitset.is_superset(&bitset) {
            return false;
        }

        match indices.split_first() {
            None => {
                // This node and all of its descendants contain the empty set, so they all supersede `set`. We only need
                // to check if any of them have a value greater than or equal to `value`.
                if matches!(self.terminal_value, Some(v) if v >= value) {
                    return true;
                }
                self.children
                    .iter()
                    .any(|(_, child)| child.supersedes(bitset, indices, value))
            }
            Some((idx, remaining_indices)) => {
                for (child_idx, child) in &self.children {
                    match (*child_idx).cmp(idx) {
                        Ordering::Less => {
                            if child.supersedes(bitset, indices, value) {
                                return true;
                            }
                        }
                        Ordering::Equal => {
                            if child.supersedes(bitset, remaining_indices, value) {
                                return true;
                            }
                        }
                        Ordering::Greater => break,
                    }
                }

                false
            }
        }
    }

    fn insert_if_max(&mut self, bitset: B, indices: &[I], value: V) -> Result<InsertResult, ()> {
        if indices.is_empty() {
            if self.supersedes(bitset, indices, value) {
                return Ok(InsertResult::Superseded);
            }

            debug_assert!(
                matches!(self.terminal_value, Some(v) if v < value)
                    || self.terminal_value.is_none()
            );

            self.terminal_value = Some(value);
            return Ok(InsertResult::Inserted);
        }

        let (idx, remaining_indices) = indices.split_first().unwrap();

        if !self.bitset.is_superset(&bitset) {
            self.bitset = self.bitset.union(&bitset);

            return match self
                .children
                .binary_search_by_key(idx, |(child_idx, _)| *child_idx)
            {
                Ok(index) => {
                    let (_, child) = &mut self.children[index];
                    // TODO: Guaranteed to insert?
                    child.insert_if_max(bitset, remaining_indices, value)
                }
                Err(insert_index) => {
                    self.children.insert(
                        insert_index,
                        (*idx, Self::new_branch(bitset, remaining_indices, value)),
                    );
                    Ok(InsertResult::Inserted)
                }
            };
        }

        for (child_idx, child) in &mut self.children {
            match (*child_idx).cmp(idx) {
                Ordering::Less => {
                    if child.supersedes(bitset, indices, value) {
                        return Ok(InsertResult::Superseded);
                    }
                }
                Ordering::Equal => {
                    if let Ok(result) = child.insert_if_max(bitset, remaining_indices, value) {
                        if matches!(result, InsertResult::Inserted) {
                            self.bitset = self.bitset.union(&bitset);
                        }

                        return Ok(result);
                    }
                }
                Ordering::Greater => break,
            }
        }

        self.bitset = self.bitset.union(&bitset);
        self.children.insert(
            self.children
                .partition_point(|(child_idx, _)| *child_idx < *idx),
            (*idx, Self::new_branch(bitset, remaining_indices, value)),
        );
        Ok(InsertResult::Inserted)
    }
}

/// Specialized data structure for storing bitsets with associated values. The single supported operation is InsertIfMax,
/// which inserts a new set-value pair into the trie if a superset with a higher value does not already exist.
#[derive(Debug, Default)]
pub struct MaxBitSetTrie<B, I, V> {
    root: Node<B, I, V>,
}

impl<B, I, V> MaxBitSetTrie<B, I, V>
where
    B: BitSet<Index = I> + Copy + Default,
    I: Ord + Copy,
    V: Ord + Copy,
{
    pub fn new() -> Self {
        Self {
            root: Node {
                bitset: B::default(),
                children: Vec::new(),
                terminal_value: None,
            },
        }
    }

    pub fn insert_if_max(&mut self, bitset: B, value: V) -> bool {
        let indices = bitset.to_vec();
        match self.root.insert_if_max(bitset, &indices, value) {
            Ok(InsertResult::Inserted) => true,
            Ok(InsertResult::Superseded) => false,
            Err(()) => unreachable!(),
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::{prop_assert_eq, proptest};

    use crate::util::BitSet;

    use super::*;

    #[derive(Debug)]
    struct NaiveMaxBitSetTrie<K, V> {
        pairs: Vec<(K, V)>,
    }

    impl<K, V> NaiveMaxBitSetTrie<K, V>
    where
        K: BitSet + Copy,
        V: Ord + Copy,
    {
        fn new() -> Self {
            Self { pairs: Vec::new() }
        }

        fn insert_if_max(&mut self, set: K, value: V) -> bool {
            for (existing_set, existing_value) in &mut self.pairs {
                if existing_set.is_superset(&set) && *existing_value >= value {
                    return false;
                }
            }

            self.pairs.push((set, value));
            true
        }
    }

    proptest! {
        #[test]
        fn prop_insert_if_max(
            entries in proptest::collection::vec((0..10u8, 0..10u8), 0..10),
        ) {
            let mut trie = MaxBitSetTrie::new();
            let mut naive_trie = NaiveMaxBitSetTrie::new();

            for (set, value) in entries {
                prop_assert_eq!(
                    trie.insert_if_max(set, value), naive_trie.insert_if_max(set, value),
                    "set = {:?}, value = {:?}\ntrie = {:#?}\nnaive_trie = {:?}",
                    set, value, trie, naive_trie,
                );
            }
        }
    }
}
