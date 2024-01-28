use crate::util::BitSet;
use petgraph::Graph;

#[derive(Debug)]
enum Node<K, V> {
    Leaf {
        set: K,
        value: V,
    },
    Branch {
        /// INVARIANT: `set` is a superset of all children's sets.
        set: K,

        /// INVARIANT: `min_value` is less than or equal to the minimum value of all children.
        /// It may be less than the minimum if there is a value associated with this branch node.
        min_value: V,

        /// INVARIANT: `max_value` is the maximum value of all children.
        max_value: V,

        children: Vec<Node<K, V>>,
    },
}

enum InsertResult {
    /// The value was inserted.
    Inserted,
    /// The value was not inserted because a superset with a higher value already exists.
    NotInserted,
}

impl<K, V> Node<K, V>
where
    K: BitSet + Copy,
    V: Ord + Copy,
{
    fn set(&self) -> K {
        match self {
            Node::Leaf { set, .. } => *set,
            Node::Branch { set, .. } => *set,
        }
    }

    fn min_value(&self) -> V {
        match self {
            Node::Leaf { value, .. } => *value,
            Node::Branch { min_value, .. } => *min_value,
        }
    }

    fn max_value(&self) -> V {
        match self {
            Node::Leaf { value, .. } => *value,
            Node::Branch { max_value, .. } => *max_value,
        }
    }

    fn insert_if_max(&mut self, set: K, value: V) -> Result<InsertResult, ()> {
        if !set.is_subset(&self.set()) {
            return Err(());
        }

        match self {
            Node::Leaf {
                set: node_set,
                value: node_value,
            } => {
                if *node_value >= value {
                    // A superset with a higher value already exists.
                    return Ok(InsertResult::NotInserted);
                }

                if *node_set == set {
                    // Exact match, update self.
                    *node_value = value;
                } else {
                    *self = Node::Branch {
                        set: *node_set,
                        min_value: *node_value,
                        max_value: value,
                        children: vec![Node::Leaf { set, value }],
                    };
                }

                Ok(InsertResult::Inserted)
            }
            Node::Branch {
                set: node_set,
                min_value,
                max_value,
                children,
                ..
            } => {
                if *min_value >= value {
                    // A superset with a higher value already exists.
                    return Ok(InsertResult::NotInserted);
                }

                if *node_set == set {
                    // Exact match, update self.
                    if *max_value <= value {
                        *self = Node::Leaf { set, value };
                    } else {
                        *min_value = value;
                        children.retain(|child| child.max_value() >= value);
                    }

                    return Ok(InsertResult::Inserted);
                }

                for child in children.iter_mut() {
                    if let Ok(result) = child.insert_if_max(set, value) {
                        return Ok(result);
                    }
                }

                // No child was a superset of `set`, so add a new child.
                children.push(Node::Leaf { set, value });
                Ok(InsertResult::Inserted)
            }
        }
    }
}

/// Data structure that maps BitSets to values and supports querying the maximum value of supersets for a given BitSet.
#[derive(Debug, Default)]
pub struct MaxBitSetTrie<K, V> {
    root: Option<Node<K, V>>,
}

impl<K, V> MaxBitSetTrie<K, V>
where
    K: BitSet + Copy,
    V: Ord + Copy,
{
    pub fn new() -> Self {
        Self { root: None }
    }

    /// Inserts a new set-value pair into the trie if a superset with a higher value does not already
    /// exist. Returns `true` if the value was inserted, `false` otherwise.
    pub fn insert_if_max(&mut self, set: K, value: V) -> bool {
        let node = match &mut self.root {
            None => {
                self.root = Some(Node::Leaf { set, value });
                return true;
            }
            Some(node) => node,
        };

        match node.insert_if_max(set, value) {
            Ok(InsertResult::Inserted) => true,
            Ok(InsertResult::NotInserted) => false,
            Err(()) => {
                let old_node = std::mem::replace(
                    node,
                    Node::Branch {
                        set: set.union(&node.set()),
                        min_value: value.min(node.min_value()),
                        max_value: value.max(node.max_value()),
                        children: Vec::new(),
                    },
                );

                match node {
                    Node::Branch { children, .. } => {
                        children.push(old_node);
                        children.push(Node::Leaf { set, value });
                    }
                    _ => unreachable!(),
                }

                true
            }
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
            let root_index = graph.add_node((root.set(), root.min_value()));
            let mut stack = vec![(root_index, root)];

            while let Some((parent_index, parent)) = stack.pop() {
                match parent {
                    Node::Leaf { .. } => {}
                    Node::Branch { children, .. } => {
                        for child in children {
                            let child_index = graph.add_node((child.set(), child.min_value()));
                            graph.add_edge(parent_index, child_index, ());
                            stack.push((child_index, child));
                        }
                    }
                }
            }
        }

        graph
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut trie = MaxBitSetTrie::<u8, u8>::new();
        println!("{:?}", trie);
        assert!(trie.insert_if_max(0b0000, 0));
        println!("{:?}", trie);
        assert!(trie.insert_if_max(0b0011, 2));
        println!("{:?}", trie);
        assert!(!trie.insert_if_max(0b0001, 1)); // 0b0001 is a subset of 0b0011 with a lower value
        println!("{:?}", trie);
        assert!(trie.insert_if_max(0b0001, 4)); // 0b0001 is a subset of 0b0011, but has a higher value
        println!("{:?}", trie);
        assert!(trie.insert_if_max(0b0011, 5));
        println!("{:?}", trie);
    }
}
