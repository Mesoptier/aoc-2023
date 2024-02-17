use num::{Num, Zero};

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

pub trait CostMap<State, Cost> {
    fn get(&self, state: &State) -> Option<Cost>;
    fn insert(&mut self, state: State, cost: Cost) -> bool;
}

pub fn a_star<P, OS, CM>(problem: P, mut open_set: OS, mut cost_map: CM) -> Option<P::Cost>
where
    P: Problem,
    P::State: Copy,
    P::Cost: Num + Ord + Copy,
    OS: OpenSet<P::State, P::Cost>,
    CM: CostMap<P::State, P::Cost>,
{
    for state in problem.sources() {
        let cost = P::Cost::zero();
        let est_cost = cost + problem.heuristic(&state);
        cost_map.insert(state, cost);
        open_set.insert(state, est_cost);
    }

    while let Some(state) = open_set.pop_min() {
        let cost = cost_map.get(&state).unwrap();

        if problem.is_target(&state) {
            // Found the target state
            return Some(cost);
        }

        problem
            .successors(&state)
            .into_iter()
            .for_each(|(next_state, next_cost)| {
                let next_cost = (cost + next_cost) as P::Cost;
                if cost_map.insert(next_state, next_cost) {
                    let est_next_cost = (next_cost + problem.heuristic(&next_state)) as P::Cost;
                    open_set.insert(next_state, est_next_cost);
                }
            });
    }

    None
}
