use crate::util::{Indexer, VecMap, VecSet};
use bucket_queue::{BucketQueue, LastInFirstOutQueue};
use num::{Num, Zero};

pub trait Problem {
    type State;
    type Cost;

    fn sources(&self) -> impl Iterator<Item = Self::State>;
    fn is_target(&self, state: &Self::State) -> bool;
    fn neighbors(&self, state: &Self::State) -> impl Iterator<Item = (Self::State, Self::Cost)>;
    fn heuristic(&self, state: &Self::State) -> Self::Cost;

    // TODO: Remove this method, in favor of a generic Queue type
    fn cost_to_index(cost: Self::Cost) -> usize;
}

pub trait BiDirProblem: Problem {
    fn targets(&self) -> impl Iterator<Item = Self::State>;
    fn is_source(&self, state: &Self::State) -> bool;
    fn rev_neighbors(&self, state: &Self::State)
        -> impl Iterator<Item = (Self::State, Self::Cost)>;
    fn rev_heuristic(&self, state: &Self::State) -> Self::Cost;
}

pub fn a_star<P, SI>(problem: P, state_indexer: SI) -> Option<P::Cost>
where
    P: Problem,
    SI: Indexer<P::State> + Clone,
    P::State: Copy,
    P::Cost: Num + Ord + Copy,
{
    let mut queue = BucketQueue::<Vec<P::State>>::new();
    let mut best_costs: VecMap<P::State, P::Cost, SI> = VecMap::new(state_indexer.clone());
    let mut visited = VecSet::new(state_indexer.clone());

    for state in problem.sources() {
        let cost = P::Cost::zero();
        best_costs.insert(&state, cost);
        let est_cost = cost + problem.heuristic(&state);
        queue.push(state, P::cost_to_index(est_cost));
    }

    while let Some(state) = queue.pop_min() {
        if !visited.insert(state) {
            // Already visited this state
            continue;
        }

        let cost = *best_costs.get(&state).unwrap();

        if problem.is_target(&state) {
            // Found the target state
            return Some(cost);
        }

        for (next_state, next_cost) in problem.neighbors(&state) {
            let next_cost = (cost + next_cost) as P::Cost;
            match best_costs.entry(&next_state) {
                Some(best_cost) if *best_cost <= next_cost => {
                    // If we've already found a better path to this state, skip it
                    continue;
                }
                entry => {
                    // Otherwise, update the best cost and add the state to the queue
                    *entry = Some(next_cost);
                }
            }

            let est_next_cost = next_cost + problem.heuristic(&next_state);
            queue.push(next_state, P::cost_to_index(est_next_cost));
        }
    }

    None
}
