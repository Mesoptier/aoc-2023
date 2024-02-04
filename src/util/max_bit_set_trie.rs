use crate::util::BitSet;
use derive_where::derive_where;
use petgraph::Graph;
use std::fmt::{Debug, Display};

pub trait SetKey: Sized {
    type Key: Ord;
    fn split_first(self) -> Option<(Self::Key, Self)>;
    fn is_empty(&self) -> bool;
}

impl<B> SetKey for B
where
    B: BitSet + Copy,
{
    type Key = B::Index;
    fn split_first(mut self) -> Option<(Self::Key, Self)> {
        self.pop().map(|first| (first, self))
    }
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[derive_where(Default; V)]
#[derive_where(Debug; V, S::Key)]
struct Node<S: SetKey, V> {
    terminal_value: Option<V>,
    children: Vec<(S::Key, Node<S, V>)>,
}

impl<S: SetKey, V> Node<S, V> {
    fn empty() -> Self {
        Self {
            terminal_value: None,
            children: Vec::new(),
        }
    }
}

impl<S, V> Node<S, V>
where
    S: SetKey + Copy,
    S::Key: Copy,
    V: Ord + Copy,
{
    /// Returns `true` if this node contains a subset of `set` with a value greater than or equal to `value`.
    fn query(&self, set: S, value: V) -> bool {
        if let Some(terminal_value) = self.terminal_value {
            if terminal_value >= value {
                return true;
            }
        }

        let mut set = set;
        let mut children = self.children.as_slice();

        while let Some((key, rest)) = set.split_first() {
            let result = children.binary_search_by_key(&key, |(key, _)| *key);

            let index = match result {
                Ok(index) => {
                    let (_, child) = &children[index];
                    if child.query(rest, value) {
                        return true;
                    }
                    index + 1
                }
                Err(index) => index,
            };
            children = &children[index..];
            set = rest;
        }

        false
    }

    /// Inserts a new (set, value) pair into the node.
    fn insert(&mut self, set: S, value: V) {
        match set.split_first() {
            None => {
                if matches!(self.terminal_value, Some(terminal_value) if terminal_value >= value) {
                    unreachable!("query should have returned true");
                }
                self.terminal_value = Some(value);
            }
            Some((key, rest)) => {
                let result = self.children.binary_search_by_key(&key, |(key, _)| *key);
                let child = match result {
                    Ok(index) => {
                        let (_, child) = &mut self.children[index];
                        child
                    }
                    Err(index) => {
                        self.children.insert(index, (key, Node::empty()));
                        let (_, child) = &mut self.children[index];
                        child
                    }
                };
                child.insert(rest, value);
            }
        }
    }
}

#[derive_where(Default; V)]
#[derive_where(Debug; V, S::Key)]
pub struct MaxSubSetTrie<S: SetKey, V> {
    root: Node<S, V>,
}

impl<S, V> MaxSubSetTrie<S, V>
where
    S: SetKey + Copy,
    S::Key: Copy,
    V: Ord + Copy,
{
    pub fn new() -> Self {
        Self {
            root: Node::empty(),
        }
    }

    /// Inserts a new set-value pair into the trie if a subset with a higher value does not already
    /// exist. Returns `true` if the value was inserted, `false` otherwise.
    pub fn insert_if_max(&mut self, set: S, value: V) -> bool {
        if self.root.query(set, value) {
            return false;
        }
        self.root.insert(set, value);
        true
    }
}

impl<S, V> Display for MaxSubSetTrie<S, V>
where
    S: SetKey,
    S::Key: Debug,
    V: Ord + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stack = vec![(&self.root, None, 0)];
        while let Some((node, key, depth)) = stack.pop() {
            let key_str = key
                .map(|key| format!("{:?}", key))
                .unwrap_or_else(|| "root".to_string());
            writeln!(
                f,
                "{:indent$}{}: {:?}",
                "",
                key_str,
                node.terminal_value,
                indent = depth * 2
            )?;

            for (key, child) in node.children.iter().rev() {
                stack.push((child, Some(key), depth + 1));
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct NodeInfo<V> {
    pub terminal_value: Option<V>,
    pub degree: usize,
}

impl<S, V> From<&Node<S, V>> for NodeInfo<V>
where
    S: SetKey,
    V: Copy,
{
    fn from(node: &Node<S, V>) -> Self {
        Self {
            terminal_value: node.terminal_value,
            degree: node.children.len(),
        }
    }
}

impl<S, V> From<&MaxSubSetTrie<S, V>> for Graph<NodeInfo<V>, S::Key>
where
    S: SetKey + Copy + Debug,
    S::Key: Copy + Debug,
    V: Ord + Copy + Debug,
{
    fn from(trie: &MaxSubSetTrie<S, V>) -> Self {
        let mut graph = Graph::new();

        let root_index = graph.add_node(NodeInfo::from(&trie.root));
        let mut stack = vec![(root_index, &trie.root)];

        while let Some((parent_index, parent)) = stack.pop() {
            for (child_key, child) in &parent.children {
                let child_index = graph.add_node(NodeInfo::from(child));
                graph.add_edge(parent_index, child_index, *child_key);
                stack.push((child_index, child));
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
    struct NaiveMaxSubSetTrie<K, V> {
        pairs: Vec<(K, V)>,
    }

    impl<K, V> NaiveMaxSubSetTrie<K, V>
    where
        K: BitSet + Copy,
        V: Ord + Copy,
    {
        fn new() -> Self {
            Self { pairs: Vec::new() }
        }

        fn insert_if_max(&mut self, set: K, value: V) -> bool {
            for (existing_set, existing_value) in &mut self.pairs {
                if existing_set.is_subset(&set) && *existing_value >= value {
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
            let mut trie = MaxSubSetTrie::new();
            let mut naive_trie = NaiveMaxSubSetTrie::new();

            for (set, value) in entries {
                prop_assert_eq!(
                    trie.insert_if_max(set, value), naive_trie.insert_if_max(set, value),
                    "set = {:?}, value = {:?}\nTRIE:\n{}\nNAIVE:\n{:?}",
                    set, value, trie, naive_trie.pairs,
                );
            }
        }
    }
}
