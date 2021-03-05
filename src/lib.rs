#![no_std]

use core::cmp;
use core::convert;
use core::ops;
use core::slice;

/// Represents an emoji, as defined by the Unicode standard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Emoji {
    id: usize,
    emoji: &'static str,
    name: &'static str,
}

impl Emoji {
    /// Returns this emoji as a string.
    ///
    /// `Emoji` also implements [`Deref`](#impl-Deref) to [`str`] so this
    /// shouldn't be needed too often.
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

    /// Returns the CLDR Short Name for this emoji.
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
}

impl cmp::PartialEq<str> for &Emoji {
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
/// let mut it = emojis::iter();
/// assert_eq!(it.next().unwrap(), "😀");
/// ```
pub fn iter() -> slice::Iter<'static, Emoji> {
    generated::EMOJIS.iter()
}

/// Lookup an emoji by Unicode value.
///
/// # Examples
///
/// ```
/// # use emojis::Emoji;
/// #
/// let rocket: &Emoji = emojis::lookup("🚀").unwrap();
/// assert!(emojis::lookup("ʕっ•ᴥ•ʔっ").is_none());
/// ```
pub fn lookup(emoji: &str) -> Option<Emoji> {
    generated::EMOJIS.iter().find(|e| e == emoji).copied()
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
