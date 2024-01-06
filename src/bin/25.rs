use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use nom::bytes::complete::tag;
use nom::character::complete::alpha1;
use nom::multi::separated_list1;
use nom::sequence::separated_pair;
use nom::IResult;

advent_of_code::solution!(25);

fn parse_line(line: &str) -> IResult<&str, (&str, Vec<&str>)> {
    separated_pair(alpha1, tag(": "), separated_list1(tag(" "), alpha1))(line)
}

fn remove_edge(adjacency_list: &mut [Vec<usize>], edge: (usize, usize)) {
    adjacency_list[edge.0].retain(|&x| x != edge.1);
    adjacency_list[edge.1].retain(|&x| x != edge.0);
}
fn insert_edge(adjacency_list: &mut [Vec<usize>], edge: (usize, usize)) {
    adjacency_list[edge.0].push(edge.1);
    adjacency_list[edge.1].push(edge.0);
}

/// Breadth-first search to find the distance from `node` to all other nodes.
fn distances_to_all_nodes(adjacency_list: &[Vec<usize>], node: usize) -> Vec<usize> {
    let mut distances = vec![usize::MAX; adjacency_list.len()];
    let mut queue = VecDeque::new();
    queue.push_back((node, 0));
    distances[node] = 0;

    while let Some((node, distance)) = queue.pop_front() {
        for &neighbor in adjacency_list[node].iter() {
            if distance + 1 < distances[neighbor] {
                queue.push_back((neighbor, distance + 1));
                distances[neighbor] = distance + 1;
            }
        }
    }

    distances
}

/// A* search to find a path from `start_node` to `end_node`, using `heuristic` to guide the search. The heuristic
/// should be an estimate of the distance from a node to the end node, and should be both admissible and consistent.
fn find_path(
    adjacency_list: &[Vec<usize>],
    start_node: usize,
    end_node: usize,
    heuristic: &[usize],
) -> Option<Vec<(usize, usize)>> {
    #[derive(Eq, PartialEq)]
    struct Entry {
        node: usize,
        score_estimate: usize,
    }
    impl PartialOrd<Self> for Entry {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }
    impl Ord for Entry {
        fn cmp(&self, other: &Self) -> Ordering {
            other.score_estimate.cmp(&self.score_estimate)
        }
    }
    let mut priority_queue = BinaryHeap::<Entry>::new();
    let mut visited = vec![false; adjacency_list.len()];
    let mut parents = vec![None; adjacency_list.len()];
    let mut scores = vec![usize::MAX; adjacency_list.len()];

    priority_queue.push(Entry {
        node: start_node,
        score_estimate: heuristic[start_node],
    });
    visited[start_node] = true;
    scores[start_node] = 0;

    while let Some(entry) = priority_queue.pop() {
        if entry.node == end_node {
            // Backtrack to find the path
            let mut path = vec![];
            let mut node = end_node;
            while let Some(parent) = parents[node] {
                path.push((parent, node));
                node = parent;
            }
            return Some(path);
        }

        for &adj_node in adjacency_list[entry.node].iter() {
            if !visited[adj_node] {
                let score = scores[entry.node] + 1;
                if score < scores[adj_node] {
                    priority_queue.push(Entry {
                        node: adj_node,
                        score_estimate: score + heuristic[adj_node],
                    });
                    visited[adj_node] = true;
                    parents[adj_node] = Some(entry.node);
                    scores[adj_node] = score;
                }
            }
        }
    }

    None
}

/// Find the size of the component containing `node`
fn find_connected_component_size(adjacency_list: &[Vec<usize>], node: usize) -> usize {
    let mut visited = vec![false; adjacency_list.len()];
    let mut stack = vec![node];
    let mut count = 0;
    while let Some(node) = stack.pop() {
        if visited[node] {
            continue;
        }
        count += 1;
        visited[node] = true;
        for &connection in &adjacency_list[node] {
            stack.push(connection);
        }
    }
    count
}

pub fn part_one(input: &str) -> Option<usize> {
    let (mut adjacency_list, mut forward_edges) = {
        let mut name_to_index = HashMap::<&str, usize>::new();
        let mut adjacency_list = vec![];
        let mut forward_edges = vec![];
        input
            .lines()
            .map(|line| parse_line(line).unwrap().1)
            .for_each(|(name, connections)| {
                let index = *name_to_index.entry(name).or_insert_with(|| {
                    adjacency_list.push(vec![]);
                    adjacency_list.len() - 1
                });
                for connection in connections {
                    let connection_index = *name_to_index.entry(connection).or_insert_with(|| {
                        adjacency_list.push(vec![]);
                        adjacency_list.len() - 1
                    });
                    adjacency_list[index].push(connection_index);
                    adjacency_list[connection_index].push(index);
                    forward_edges.push((index, connection_index));
                }
            });

        (adjacency_list, forward_edges)
    };

    forward_edges.sort_by_key(|&edge| {
        usize::MAX - (adjacency_list[edge.0].len() + adjacency_list[edge.1].len())
    });

    'outer: for edge_i in forward_edges {
        remove_edge(&mut adjacency_list, edge_i);

        // Build heuristic for path search in the inner loops
        let heuristic = distances_to_all_nodes(&adjacency_list, edge_i.1);

        // Find path from edge_i.0 to edge_i.1
        let path_j = find_path(&adjacency_list, edge_i.0, edge_i.1, &heuristic).unwrap();

        for edge_j in path_j {
            remove_edge(&mut adjacency_list, edge_j);

            // Find path from edge_i.0 to edge_i.1
            let path_k = find_path(&adjacency_list, edge_i.0, edge_i.1, &heuristic).unwrap();

            // If there is a bridge, it must be along this path
            for edge_k in path_k {
                remove_edge(&mut adjacency_list, edge_k);

                if find_path(&adjacency_list, edge_i.0, edge_i.1, &heuristic).is_none() {
                    // No alternative path from edge_i.0 to edge_i.1, so edge_k must've been a bridge
                    break 'outer;
                }

                insert_edge(&mut adjacency_list, edge_k);
            }

            insert_edge(&mut adjacency_list, edge_j);
        }

        insert_edge(&mut adjacency_list, edge_i);
    }

    let group_size1 = find_connected_component_size(&adjacency_list, 0);
    let group_size2 = adjacency_list.len() - group_size1;

    Some(group_size1 * group_size2)
}

pub fn part_two(_input: &str) -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_one() {
        let result = part_one(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, Some(54));
    }

    #[test]
    fn test_part_two() {
        let result = part_two(&advent_of_code::template::read_file("examples", DAY));
        assert_eq!(result, None);
    }
}
