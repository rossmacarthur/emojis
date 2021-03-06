//! Lookup, iterate, and search emojis.
//!
//! # Examples
//!
//! Lookup any emoji.
//! ```
//! let face = emojis::lookup("🤨").unwrap();
//! assert_eq!(face.as_str(), "\u{1F928}");
//! assert_eq!(face.name(), "face with raised eyebrow");
//! assert_eq!(face.group(), emojis::Group::SmileysAndEmotion);
//! ```
//!
//! Iterate over all the emojis.
//! ```
//! let emoji = emojis::iter().next().unwrap();
//! assert_eq!(emoji, "😀");
//! ```
//!
//! Iterate over all the emojis in a group.
//! ```
//! let emoji = emojis::Group::FoodAndDrink.emojis().next().unwrap();
//! assert_eq!(emoji, "🍇");
//! ```
//!
//! Fuzzy search for emojis.
//! ```
//! let emoji = emojis::search("rket").next().unwrap();
//! assert_eq!(emoji, "🚀");
//! ```

#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod search;

use core::cmp;
use core::convert;
use core::ops;

pub use crate::generated::Group;

#[cfg(feature = "search")]
pub use crate::search::search;

/// Represents an emoji.
///
/// See [Unicode.org](https://unicode.org/emoji/charts/full-emoji-list.html) for
/// more information.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Emoji {
    id: usize,
    emoji: &'static str,
    name: &'static str,
    group: Group,
}

impl Emoji {
    /// Returns this emoji as a string.
    ///
    /// Note: `Emoji` also implements [`Deref`](#impl-Deref) to [`str`], and
    /// [`AsRef<str>`](#impl-AsRef<str>) so this method shouldn't have to be
    /// used very frequently.
    ///
    /// # Examples
    ///
    /// ```
    /// let rocket = emojis::lookup("🚀").unwrap();
    /// assert_eq!(rocket.as_str(), "🚀")
    /// ```
    pub const fn as_str(&self) -> &str {
        self.emoji
    }

    /// Returns the CLDR short name for this emoji.
    ///
    /// # Examples
    ///
    /// ```
    /// let cool = emojis::lookup("😎").unwrap();
    /// assert_eq!(cool.name(), "smiling face with sunglasses");
    /// ```
    pub const fn name(&self) -> &str {
        self.name
    }

    /// Returns this emoji's group.
    ///
    /// # Examples
    ///
    /// ```
    /// # use emojis::Group;
    /// #
    /// let flag = emojis::lookup("🇿🇦").unwrap();
    /// assert_eq!(flag.group(), Group::Flags);
    /// ```
    pub const fn group(&self) -> Group {
        self.group
    }
}

impl cmp::PartialEq<str> for Emoji {
    fn eq(&self, s: &str) -> bool {
        cmp::PartialEq::eq(self.as_str(), s)
    }
}

impl cmp::PartialOrd for Emoji {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(cmp::Ord::cmp(self, other))
    }
}

impl cmp::Ord for Emoji {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        cmp::Ord::cmp(&self.id, &other.id)
    }
}

impl convert::AsRef<str> for Emoji {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl ops::Deref for Emoji {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

/// Returns an iterator over all emojis.
///
/// Ordered by Unicode CLDR data.
///
/// # Examples
///
/// ```
/// let mut iter = emojis::iter();
/// assert_eq!(iter.next().unwrap(), "😀");
/// ```
pub fn iter() -> impl Iterator<Item = &'static Emoji> {
    crate::generated::EMOJIS.iter()
}

/// Lookup an emoji by Unicode value.
///
/// # Examples
///
/// ```
/// let rocket = emojis::lookup("🚀").unwrap();
/// assert!(emojis::lookup("ʕっ•ᴥ•ʔっ").is_none());
/// ```
pub fn lookup(emoji: &str) -> Option<Emoji> {
    crate::generated::EMOJIS
        .iter()
        .find(|&e| e == emoji)
        .copied()
}

impl Group {
    /// Returns an iterator over all emojis in this group.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut iter = emojis::Group::Flags.emojis();
    /// assert_eq!(iter.next().unwrap(), "🏁");
    /// ```
    pub fn emojis(&self) -> impl Iterator<Item = &'static Emoji> {
        let group = *self;
        iter()
            .skip_while(move |emoji| emoji.group != group)
            .take_while(move |emoji| emoji.group == group)
    }
}

mod generated;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_ordering() {
        let grinning_face = lookup("😀");
        let winking_face = lookup("😉");
        assert!(grinning_face < winking_face);
        assert!(winking_face > grinning_face);
        assert!(grinning_face == grinning_face);
    }
}
