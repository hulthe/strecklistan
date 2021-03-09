use std::cmp::{Ord, Ordering, PartialOrd};

pub trait FuzzySearch {
    fn compare_fuzzy(&self, search: &str) -> FuzzyScore;
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FuzzyScore {
    pub score: i32,
    pub matches: Vec<FuzzyCharMatch>,
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct FuzzyCharMatch {
    pub base_str_index: usize,
    pub search_str_index: usize,
}

impl PartialOrd for FuzzyScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FuzzyScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}
