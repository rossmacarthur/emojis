#![cfg(feature = "search")]

use std::cmp::Reverse;
use std::prelude::v1::*;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use itertools::Itertools;

use crate::Emoji;

/// Search all emojis.
///
/// This function returns an iterator over emojis matching the given search
/// query. The query is matched against the emoji CLDR short names and exact
/// matches and higher scores are returned first.
///
/// # Examples
///
/// ```
/// let mut iter = emojis::search("star");
/// assert_eq!(iter.next().unwrap(), "⭐");
/// assert_eq!(iter.next().unwrap(), "🤩");
/// assert_eq!(iter.next().unwrap(), "✡️");
/// ```
pub fn search(query: &str) -> impl Iterator<Item = &'static Emoji> {
    let matcher = SkimMatcherV2::default();
    crate::generated::EMOJIS
        .iter()
        .filter_map(|emoji| {
            matcher
                .fuzzy_indices(emoji.name(), query)
                .map(|(score, _)| (emoji, score))
        })
        .sorted_by_key(|(emoji, score)| {
            // fuzzy-matcher doesn't seem to give exact matches a higher score
            // so we do this to put iexact matches first.
            let exact = emoji.name().to_lowercase() == query.to_lowercase();
            (Reverse(exact), Reverse(*score), emoji.id)
        })
        .map(|(emoji, _)| emoji)
        .collect::<Vec<_>>()
        .into_iter()
}
