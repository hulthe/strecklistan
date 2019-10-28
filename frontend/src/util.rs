use crate::fuzzy_search::FuzzySearch;
use laggit_api::book_account::BookAccount;
use laggit_api::member::Member;
use std::rc::Rc;

/// Compare a base string to a user-input search
///
/// Returns a tuple of the match score, as well as the indices of every char in `search` which maps
/// to an index in `base`
pub fn compare_fuzzy<B, S>(base: B, search: S) -> (i32, Vec<(usize, usize)>)
where
    B: Iterator<Item = char> + Clone,
    S: IntoIterator<Item = char>,
{
    let mut base = base.into_iter().enumerate();

    // How alike the search string is to self.name
    //let mut score = -(search.len() as i32);
    let mut score = 0;

    // Vector of which char index in s maps to which char index in self.name
    let mut matches = vec![];

    for (i, sc) in search.into_iter().enumerate() {
        let sc = sc.to_ascii_lowercase();
        let mut add = 3;
        let mut base_tmp = base.clone();
        while let Some((j, bc)) = base_tmp.next() {
            let bc = bc.to_ascii_lowercase();
            if bc == sc {
                matches.push((i, j));
                score += add;
                base = base_tmp;
                break;
            } else {
                add = 2;
            }
        }
    }

    (score, matches)
}

pub fn sort_tillgodolista_search(
    search: &str,
    list: &mut Vec<(i32, Vec<(usize, usize)>, Rc<BookAccount>, Rc<Member>)>,
) {
    for (score, matches, _acc, member) in list.iter_mut() {
        let (s, m) = member.compare_fuzzy(search);
        *score = s;
        *matches = m;
    }
    list.sort_by(|(scr_a, _, acc_a, _), (scr_b, _, acc_b, _)| {
        scr_b.cmp(scr_a).then(acc_a.id.cmp(&acc_b.id))
    });
}
