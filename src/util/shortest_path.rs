use num::{Num, Zero};

use crate::util::{Indexer, VecMap};

pub trait Problem {
    type State;
    type Cost;

    fn sources(&self) -> impl IntoIterator<Item = Self::State>;
    fn is_target(&self, state: &Self::State) -> bool;
    fn successors(
        &self,
        state: &Self::State,
    ) -> impl IntoIterator<Item = (Self::State, Self::Cost)>;
    fn heuristic(&self, state: &Self::State) -> Self::Cost;
}

pub trait BiDirProblem: Problem {
    fn targets(&self) -> impl IntoIterator<Item = Self::State>;
    fn is_source(&self, state: &Self::State) -> bool;
    fn rev_successors(
        &self,
        state: &Self::State,
    ) -> impl IntoIterator<Item = (Self::State, Self::Cost)>;
    fn rev_heuristic(&self, state: &Self::State) -> Self::Cost;
}

pub trait OpenSet<State, Cost> {
    fn insert(&mut self, state: State, cost: Cost);
    fn pop_min(&mut self) -> Option<State>;
}

pub fn a_star<P, OS, SI>(problem: P, mut open_set: OS, state_indexer: SI) -> Option<P::Cost>
where
    P: Problem,
    P::State: Copy,
    P::Cost: Num + Ord + Copy,
    OS: OpenSet<P::State, P::Cost>,
    SI: Indexer<P::State>,
{
    let mut best_costs = VecMap::new(state_indexer);

    for state in problem.sources() {
        let cost = P::Cost::zero();
        let est_cost = cost + problem.heuristic(&state);
        best_costs.insert(&state, cost);
        open_set.insert(state, est_cost);
    }

    while let Some(state) = open_set.pop_min() {
        let cost = *best_costs.get(&state).unwrap();

        if problem.is_target(&state) {
            // Found the target state
            return Some(cost);
        }

        problem
            .successors(&state)
            .into_iter()
            .filter_map(|(next_state, next_cost)| {
                let next_cost = (cost + next_cost) as P::Cost;
                match best_costs.entry(&next_state) {
                    Some(best_cost) if *best_cost <= next_cost => {
                        // If we've already found a better path to this state, skip it
                        None
                    }
                    entry => {
                        // Otherwise, update the best cost and add the state to the queue
                        *entry = Some(next_cost);

                        let est_next_cost = next_cost + problem.heuristic(&next_state);
                        Some((next_state, est_next_cost))
                    }
                }
            })
            .for_each(|(next_state, est_next_cost)| {
                open_set.insert(next_state, est_next_cost);
            });
    }

    None
}
