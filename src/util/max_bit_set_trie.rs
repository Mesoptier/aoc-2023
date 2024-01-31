use std::cmp::Ordering;

#[derive(Debug, Default)]
struct Node<K, V> {
    children: Vec<(K, Node<K, V>)>,
    terminal_value: Option<V>,
}

enum InsertResult {
    Inserted,
    Superseded,
}

impl<K, V> Node<K, V>
where
    K: Ord + Copy,
    V: Ord + Copy,
{
    fn new_branch(set: &[K], value: V) -> Self {
        match set.split_first() {
            None => Self {
                children: Vec::new(),
                terminal_value: Some(value),
            },
            Some((key, remaining_set)) => Self {
                children: vec![(*key, Self::new_branch(remaining_set, value))],
                terminal_value: None,
            },
        }
    }

    fn insert_if_max(
        &mut self,
        set: &[K],
        value: V,
        allow_insert: bool,
    ) -> Result<InsertResult, ()> {
        if set.is_empty() {
            for (_, child) in &mut self.children {
                if let Ok(result) = child.insert_if_max(set, value, false) {
                    return Ok(result);
                }
            }

            return match self.terminal_value {
                None if allow_insert => {
                    self.terminal_value = Some(value);
                    Ok(InsertResult::Inserted)
                }
                Some(terminal_value) if terminal_value >= value => Ok(InsertResult::Superseded),
                Some(terminal_value) if allow_insert => {
                    debug_assert!(terminal_value < value);
                    self.terminal_value = Some(value);
                    Ok(InsertResult::Inserted)
                }
                _ => Err(()),
            };
        }

        let (key, remaining_set) = set.split_first().unwrap();
        for (child_key, child) in &mut self.children {
            match (*child_key).cmp(key) {
                Ordering::Less => {
                    if let Ok(result) = child.insert_if_max(set, value, false) {
                        return Ok(result);
                    }
                }
                Ordering::Equal => {
                    if let Ok(result) = child.insert_if_max(remaining_set, value, allow_insert) {
                        return Ok(result);
                    }
                }
                Ordering::Greater => break,
            }
        }

        if allow_insert {
            let index = self
                .children
                .partition_point(|(child_key, _)| *child_key < *key);
            self.children
                .insert(index, (*key, Self::new_branch(remaining_set, value)));
            Ok(InsertResult::Inserted)
        } else {
            Err(())
        }
    }
}

/// Specialized data structure for storing bitsets with associated values. The single supported operation is InsertIfMax,
/// which inserts a new set-value pair into the trie if a superset with a higher value does not already exist.
#[derive(Debug, Default)]
pub struct MaxBitSetTrie<K, V> {
    root: Node<K, V>,
}

impl<K, V> MaxBitSetTrie<K, V>
where
    K: Ord + Copy,
    V: Ord + Copy,
{
    pub fn new() -> Self {
        Self {
            root: Node {
                children: Vec::new(),
                terminal_value: None,
            },
        }
    }

    pub fn insert_if_max(&mut self, set: &[K], value: V) -> bool {
        match self.root.insert_if_max(set, value, true) {
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
                let set_indices = (0..8).filter(|i| set.get(*i)).collect::<Vec<_>>();
                prop_assert_eq!(
                    trie.insert_if_max(&set_indices, value), naive_trie.insert_if_max(set, value),
                    "set = {:?}, value = {:?}\ntrie = {:#?}\nnaive_trie = {:?}",
                    set, value, trie, naive_trie,
                );
            }
        }
    }
}
