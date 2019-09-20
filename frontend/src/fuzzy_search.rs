use crate::util::compare_fuzzy;

pub trait FuzzySearch {
    fn get_search_string(&self) -> &str;

    fn compare_fuzzy(&self, search: &str) -> (i32, Vec<(usize, usize)>) {
        compare_fuzzy(self.get_search_string(), search)
    }
}
