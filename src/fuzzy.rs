use std::iter;

use crate::types::{Playlist, Song, User};

pub(crate) enum SearchType {
    Uploader,
    Title,
    Default,
    User,
    Id,
}

pub(crate) trait FuzzyComparable<'a> {
    fn search_term(&self, search_type: &SearchType) -> &str;
}

impl<'a> FuzzyComparable<'a> for Song {
    fn search_term(&self, search_type: &SearchType) -> &str {
        type S = SearchType;
        match search_type {
            S::Uploader => &self.uploader,
            S::Title => &self.title,
            S::Default => &self.default_search,
            S::User => "",
            S::Id => "",
        }
    }
}

impl<'a> FuzzyComparable<'a> for User {
    fn search_term(&self, _: &SearchType) -> &str {
        &self.username
    }
}

impl<'a> FuzzyComparable<'a> for Playlist {
    fn search_term(&self, _: &SearchType) -> &str {
        &self.name
    }
}

// taken from https://docs.rs/crate/rust-fuzzy-search/latest/source/src/lib.rs
// I don't need all the crate and I also want to be able to tweak the code without an additional
// repo

pub(crate) fn fuzzy_search_best_n<'a, T: FuzzyComparable<'a>>(
    s: &str,
    list: &'a [T],
    n: usize,
    st: &SearchType,
) -> Vec<(&'a T, f32)> {
    fuzzy_search_sorted(s, list, st)
        .into_iter()
        .take(n)
        .collect()
}

pub(crate) fn fuzzy_search_sorted<'a, T: FuzzyComparable<'a>>(
    s: &str,
    list: &'a [T],
    st: &SearchType,
) -> Vec<(&'a T, f32)> {
    let mut res = fuzzy_search(s, list, st);
    res.sort_by(|(_, d1), (_, d2)| d2.partial_cmp(d1).unwrap());
    res
}

#[inline]
pub(crate) fn fuzzy_search<'a, T: FuzzyComparable<'a>>(
    s: &str,
    list: &'a [T],
    st: &SearchType,
) -> Vec<(&'a T, f32)> {
    list.iter()
        .map(|value| {
            let res = fuzzy_compare(s, value.search_term(st));
            (value, res)
        })
        .collect()
}

#[inline]
pub(crate) fn fuzzy_compare(a: &str, b: &str) -> f32 {
    // gets length of first input string plus 1 (because of the 3 added spaces (' '))
    let string_len = a.chars().count() + 1;

    // gets the trigrams for both strings
    let trigrams_a = trigrams(a);
    let trigrams_b = trigrams(b);

    // accumulator
    let mut acc: f32 = 0.0f32;
    // counts the number of trigrams of the
    // first string that are also present in the second one
    for t_a in &trigrams_a {
        for t_b in &trigrams_b {
            if t_a == t_b {
                acc += 1.0f32;
                break;
            }
        }
    }
    let res = acc / (string_len as f32);
    // crops between zero and one
    if (0.0f32..=1.0f32).contains(&res) {
        res
    } else {
        0.0f32
    }
}

#[inline]
fn trigrams(s: &str) -> Vec<(char, char, char)> {
    let it_1 = iter::once(' ').chain(iter::once(' ')).chain(s.chars());
    let it_2 = iter::once(' ').chain(s.chars());
    let it_3 = s.chars().chain(iter::once(' '));

    let res: Vec<(char, char, char)> = it_1
        .zip(it_2)
        .zip(it_3)
        .map(|((a, b), c): ((char, char), char)| (a, b, c))
        .collect();
    res
}
