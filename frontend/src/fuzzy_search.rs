pub trait FuzzySearch {
    fn compare_fuzzy(&self, search: &str) -> (i32, Vec<(usize, usize)>);
}
