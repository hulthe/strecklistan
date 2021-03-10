pub mod export;

use crate::fuzzy_search::{FuzzyCharMatch, FuzzyScore};
use seed::browser::dom::event_handler::ev;
use seed::dom_entity_names::Ev;
use seed::virtual_dom::event_handler_manager::event_handler::EventHandler;
use semver::Version;

pub const DATE_INPUT_FMT: &'static str = "%Y-%m-%d";
//pub const TIME_INPUT_FMT: &'static str = "%H:%M";

/// Check if client version supports api version
pub fn compare_semver(client_version: Version, api_version: Version) -> bool {
    match (&client_version, &api_version) {
        (
            Version {
                major: 0,
                minor: 0,
                patch: v1,
                ..
            },
            Version {
                major: 0,
                minor: 0,
                patch: v2,
                ..
            },
        ) => v1 == v2,
        (
            Version {
                major: 0,
                minor: mi1,
                patch: p1,
                ..
            },
            Version {
                major: 0,
                minor: mi2,
                patch: p2,
                ..
            },
        ) => (mi1 == mi2) && (p2 >= p1),
        (
            Version {
                major: ma1,
                minor: mi1,
                ..
            },
            Version {
                major: ma2,
                minor: mi2,
                ..
            },
        ) => (ma1 == ma2) && (mi2 >= mi1),
    }
}

pub fn simple_ev<Ms: Clone + 'static>(trigger: impl Into<Ev>, message: Ms) -> EventHandler<Ms> {
    ev(trigger, move |_| message)
}

/// Compare a base string to a user-input search
///
/// Returns a tuple of the match score, as well as the indices of every char in `search` which maps
/// to an index in `base`
pub fn compare_fuzzy<B, S>(base: B, search: S) -> FuzzyScore
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
                matches.push(FuzzyCharMatch {
                    search_str_index: i,
                    base_str_index: j,
                });

                score += add;
                base = base_tmp;
                break;
            } else {
                add = 2;
            }
        }
    }

    FuzzyScore { score, matches }
}

use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Write;
struct WriteComparer<'a> {
    cmp_to: &'a str,
    ord: Ordering,
    s1_ended: bool,
}

impl<'a> WriteComparer<'a> {
    pub fn new(cmp_to: &'a str) -> WriteComparer<'a> {
        WriteComparer {
            cmp_to,
            ord: Ordering::Equal,
            s1_ended: false,
        }
    }
}

impl<'a> Write for WriteComparer<'a> {
    fn write_str(&mut self, s1: &str) -> Result<(), std::fmt::Error> {
        if self.s1_ended && s1 != "" {
            self.s1_ended = false;
            self.ord = Ordering::Equal;
        }
        let mut s1 = s1;
        loop {
            //println!("s1: \"{}\"  s2: \"{}\"", s1, self.cmp_to);
            if self.ord != Ordering::Equal {
                break;
            } else if self.cmp_to == "" {
                if s1 != "" {
                    self.ord = Ordering::Greater; // TODO
                }
                break;
            } else if s1 == "" {
                self.ord = Ordering::Less; // TODO
                self.s1_ended = true;
                break;
            }

            let c1 = s1.chars().next().unwrap();
            let c2 = self.cmp_to.chars().next().unwrap();

            self.ord = c1.cmp(&c2);

            s1 = &s1[c1.len_utf8()..];
            self.cmp_to = &self.cmp_to[c2.len_utf8()..];
        }
        Ok(())
    }
}

pub trait CompareToStr {
    fn cmp_to_str(&self, s: &str) -> Ordering;
}

impl<T> CompareToStr for T
where
    T: Display,
{
    fn cmp_to_str(&self, s: &str) -> Ordering {
        let mut w = WriteComparer::new(s);
        write!(&mut w, "{}", self).unwrap();
        w.ord
    }
}

#[cfg(test)]
mod test {
    use super::CompareToStr;
    use std::cmp::Ordering;
    #[test]
    fn test_str_cmp() {
        assert_eq!(1.cmp_to_str("1"), Ordering::Equal);
        assert_eq!(2.cmp_to_str("2"), Ordering::Equal);
        assert_eq!(3.cmp_to_str("3"), Ordering::Equal);
        assert_eq!(4.cmp_to_str("4"), Ordering::Equal);
        assert_eq!(10.cmp_to_str("10"), Ordering::Equal);
        assert_eq!(1.cmp_to_str("01"), Ordering::Greater);
        assert_eq!(111.cmp_to_str("111"), Ordering::Equal);
        assert_eq!(999.cmp_to_str("999"), Ordering::Equal);
        assert_eq!(9999.cmp_to_str("999"), Ordering::Greater);
        assert_eq!((-10).cmp_to_str("-10"), Ordering::Equal);
        assert_eq!((-10).cmp_to_str("99"), Ordering::Less);
        assert_eq!(89.cmp_to_str("99"), Ordering::Less);

        for i in -99..=99 {
            let s = format!("{}", i);
            assert_eq!(i.cmp_to_str(&s), Ordering::Equal);
        }
    }
}
