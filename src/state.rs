use ordered_float::OrderedFloat;
use std::cmp::Ordering;

#[derive(Clone, Hash)]
pub struct State {
    pub id: String,
    pub score: OrderedFloat<f32>,
    pub scorea: OrderedFloat<f32>,
    pub scoreb: OrderedFloat<f32>,
    pub scorec: OrderedFloat<f32>,
    pub scored: OrderedFloat<f32>,
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.score == other.score
    }
}

impl Eq for State {}

// The priority queue depends on `Ord`.
// Explicitly implement the trait so the queue becomes a min-heap
// instead of a max-heap.
impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        // Notice that the we flip the ordering on costs.
        // In case of a tie we compare positions - this step is necessary
        // to make implementations of `PartialEq` and `Ord` consistent.
        other.score.partial_cmp(&self.score).unwrap()
    }
}

// `PartialOrd` needs to be implemented as well.
impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}