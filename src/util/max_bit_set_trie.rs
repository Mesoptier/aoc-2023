use crate::util::{BitSet, ContainmentType};
use petgraph::Graph;
use std::fmt::Debug;

#[derive(Debug)]
struct Node<B, V> {
    /// The set represented by this node. For internal nodes, this is the union of the sets of the children.
    set: B,
    /// The value (if any) associated with the set represented by this node.
    terminal_value: Option<V>,
    /// The minimum value of any set represented by this node or its children.
    min_value: V,
    /// The maximum value of any set represented by this node or its children.
    max_value: V,
    children: Vec<Node<B, V>>,
}

impl<B, V> Node<B, V>
where
    B: BitSet + Copy + Debug,
    V: Ord + Copy + Debug,
{
    /// Returns `true` if this node contains a superset of `set` with a value greater than or equal to `value`.
    fn contains_sup(&self, set: B, value: V) -> bool {
        if self.set.is_superset(&set) {
            if let Some(terminal_value) = self.terminal_value {
                if terminal_value >= value {
                    return true;
                }
            }

            // TODO: Could probably use min/max values to prune early.
            self.children
                .iter()
                .any(|child| child.contains_sup(set, value))
        } else {
            false
        }
    }

    /// Inserts a new (set, value) pair into the node, if possible without violating invariants. Returns `true` if the
    /// pair was inserted.
    fn insert(&mut self, set: B, value: V) -> bool {
        match self.set.containment_type(&set) {
            ContainmentType::None | ContainmentType::Subset => false,
            ContainmentType::Equal => {
                match self.terminal_value {
                    Some(terminal_value) if terminal_value < value => {
                        self.terminal_value = Some(value);
                        // Updating min_value is not necessary, since min_value <= terminal_value < value.
                        // self.min_value = self.min_value.min(value);
                        self.max_value = self.max_value.max(value);
                        true
                    }
                    None => {
                        self.terminal_value = Some(value);
                        self.min_value = self.min_value.min(value);
                        self.max_value = self.max_value.max(value);
                        true
                    }
                    _ => unreachable!(
                        "Node::insert() called with but set already present with higher value\n{:?}", self,
                    ),
                }
            }
            ContainmentType::Superset => {
                for child in &mut self.children {
                    if child.insert(set, value) {
                        self.min_value = self.min_value.min(value);
                        self.max_value = self.max_value.max(value);
                        return true;
                    }
                }

                self.children.push(Node {
                    set,
                    terminal_value: Some(value),
                    min_value: value,
                    max_value: value,
                    children: Vec::new(),
                });
                true
            }
        }
    }

    // TODO: Combine `contains_sup` and `insert` into a single function, to avoid traversing the tree twice.
}

/// Data structure that maps BitSets to values and supports querying the maximum value of supersets for a given BitSet.
#[derive(Debug, Default)]
pub struct MaxBitSetTrie<B, V> {
    root: Option<Node<B, V>>,
}

impl<B, V> MaxBitSetTrie<B, V>
where
    B: BitSet + Copy + Debug,
    V: Ord + Copy + Debug,
{
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Inserts a new set-value pair into the trie if a superset with a higher value does not already
    /// exist. Returns `true` if the value was inserted, `false` otherwise.
    pub fn insert_if_max(&mut self, set: B, value: V) -> bool {
        if let Some(root) = &mut self.root {
            if root.contains_sup(set, value) {
                false
            } else if root.insert(set, value) {
                true
            } else {
                let root = self.root.take().unwrap();

                if set.is_superset(&root.set) {
                    self.root = Some(Node {
                        set,
                        terminal_value: Some(value),
                        min_value: root.min_value.min(value),
                        max_value: root.max_value.max(value),
                        children: vec![root],
                    });
                } else {
                    self.root = Some(Node {
                        set: root.set.union(&set),
                        terminal_value: None,
                        min_value: root.min_value.min(value),
                        max_value: root.max_value.max(value),
                        children: vec![
                            root,
                            Node {
                                set,
                                terminal_value: Some(value),
                                min_value: value,
                                max_value: value,
                                children: Vec::new(),
                            },
                        ],
                    });
                }

                true
            }
        } else {
            self.root = Some(Node {
                set,
                terminal_value: Some(value),
                min_value: value,
                max_value: value,
                children: Vec::new(),
            });
            true
        }
    }
}

impl<K, V> From<&MaxBitSetTrie<K, V>> for Graph<(K, V), ()>
where
    K: BitSet + Copy,
    V: Ord + Copy,
{
    fn from(trie: &MaxBitSetTrie<K, V>) -> Self {
        let mut graph = Graph::new();

        if let Some(root) = &trie.root {
            let root_index = graph.add_node((root.set, root.min_value));
            let mut stack = vec![(root_index, root)];

            while let Some((parent_index, parent)) = stack.pop() {
                for child in &parent.children {
                    let child_index = graph.add_node((child.set, child.min_value));
                    graph.add_edge(parent_index, child_index, ());
                    stack.push((child_index, child));
                }
            }
        }

        graph
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use proptest::{prop_assert_eq, proptest};

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
                    "set = {:?}, value = {:?}\ntrie = {:?}\nnaive_trie = {:?}",
                    set, value, trie, naive_trie,
                );
            }
        }
    }
}
