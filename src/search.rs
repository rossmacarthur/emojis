#![cfg(feature = "search")]

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd, Reverse};
use std::vec;
use std::vec::Vec;

use crate::Emoji;

/// A similarity score.
#[derive(Debug, Clone, Copy)]
struct Score(f64);

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl Eq for Score {}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.partial_cmp(&other.0).unwrap()
    }
}

fn similarity(thing: &str, query: &str) -> Score {
    let mul = if thing.starts_with(query) { 2. } else { 1. };
    Score(mul * strsim::jaro(thing, query))
}

fn emoji_score(emoji: &Emoji, query: &str) -> Option<Score> {
    let mut scores = vec![similarity(emoji.name(), query)];
    if let Some(aliases) = emoji.aliases {
        for alias in aliases {
            scores.push(similarity(alias, query))
        }
    }
    let score = scores.into_iter().max().unwrap();
    if score.0 > 0.75 {
        Some(score)
    } else {
        None
    }
}

/// Search all emojis.
///
/// This function returns an iterator over emojis matching the given search
/// query. The query is matched against the emoji CLDR short names and gemoji
/// shortcodes and the returned iterator is sorted with best scores first.
///
/// # Examples
///
/// ```
/// let mut iter = emojis::search("star");
/// assert_eq!(iter.next().unwrap(), "⭐");
/// assert_eq!(iter.next().unwrap(), "🌟");
/// assert_eq!(iter.next().unwrap(), "🌠");
/// ```
pub fn search(query: &str) -> impl Iterator<Item = &'static Emoji> {
    let mut emojis: Vec<_> = crate::generated::EMOJIS
        .iter()
        .filter_map(|emoji| emoji_score(emoji, query).map(|s| (emoji, s)))
        .collect();
    emojis.sort_by_key(|(emoji, score)| (Reverse(*score), emoji.id));
    emojis
        .into_iter()
        .map(|(emoji, _)| emoji)
        .collect::<Vec<_>>()
        .into_iter()
}
