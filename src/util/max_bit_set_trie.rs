use crate::util::{BitSet, ContainmentType};
use petgraph::Graph;

#[derive(Debug)]
struct Node<K, V> {
    /// INVARIANT: `set` is a superset of all children's sets.
    set: K,

    /// INVARIANT: `min_value` is less than or equal to the minimum value of all valid children.
    /// It may be less than the minimum if there is a value associated with this branch node.
    min_value: V,

    /// INVARIANT: `max_value` is the maximum value of all children.
    max_value: V,

    // TODO: INVARIANT: `children` is sorted by `min_value`?
    children: Vec<Node<K, V>>,
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
    fn leaf(set: K, value: V) -> Self {
        Self {
            set,
            min_value: value,
            max_value: value,
            children: Vec::new(),
        }
    }

    fn pair_branch(left: Self, right: Self) -> Self {
        Self {
            set: left.set.union(&right.set),
            min_value: left.min_value.min(right.min_value),
            max_value: left.max_value.max(right.max_value),
            children: vec![left, right],
        }
    }

    fn insert_if_max(&mut self, set: K, value: V) -> Result<InsertResult, ()> {
        match set.containment_type(&self.set) {
            // TODO: Should superset behavior be the same as equal behavior? Except also update the set.
            ContainmentType::None | ContainmentType::Superset => Err(()),
            ContainmentType::Equal => {
                if self.min_value >= value {
                    // An equal set with a higher value already exists.
                    return Ok(InsertResult::NotInserted);
                }

                if value >= self.max_value {
                    // New set is a superset of all children and new value is greater than all children, so this node
                    // can be replaced with a leaf.
                    self.min_value = value;
                    self.max_value = value;
                    self.children.clear();
                } else {
                    self.min_value = value;

                    let mut i = 0;
                    while i < self.children.len() {
                        let child = &self.children[i];
                        if child.max_value <= value {
                            self.children.swap_remove(i);
                        } else if child.min_value <= value {
                            let child = self.children.swap_remove(i);
                            self.children.extend(child.children);
                        } else {
                            i += 1;
                        }
                    }
                }

                self.assert_invariants();
                Ok(InsertResult::Inserted)
            }
            ContainmentType::Subset => {
                if self.min_value >= value {
                    // A superset with a higher value already exists.
                    return Ok(InsertResult::NotInserted);
                }

                // Value will definitely be inserted in this subtree, so update max_value.
                self.max_value = self.max_value.max(value);

                // Try to insert into a child node.
                for child in self.children.iter_mut() {
                    if let Ok(result) = child.insert_if_max(set, value) {
                        self.assert_invariants();
                        return Ok(result);
                    }
                }

                // Could not insert into a child node, so add a new child.
                self.children.push(Node::leaf(set, value));

                self.assert_invariants();
                Ok(InsertResult::Inserted)
            }
        }
    }

    #[inline]
    fn assert_invariants(&self) {
        assert!(self
            .children
            .iter()
            .all(|child| self.set.is_superset(&child.set)));
        assert!(self
            .children
            .iter()
            .all(|child| self.min_value <= child.min_value));
        assert!(self
            .children
            .iter()
            .all(|child| self.max_value >= child.max_value));
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
                self.root = Some(Node::leaf(set, value));
                return true;
            }
            Some(node) => node,
        };

        match node.insert_if_max(set, value) {
            Ok(InsertResult::Inserted) => true,
            Ok(InsertResult::NotInserted) => false,
            Err(()) => {
                // Could not insert into the root node, so replace it with a branch node holding both the old root
                // and the new pair.

                let old_root = self.root.take().unwrap();
                self.root = Some(Node::pair_branch(old_root, Node::leaf(set, value)));

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
