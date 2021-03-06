//! ✨ Lookup and iterate over emoji names, shortcodes, and groups.
//!
//! # Examples
//!
//! Lookup any emoji by Unicode value or GitHub shortcode.
//! ```
//! let face = emojis::lookup("🤨").unwrap();
//! // Or
//! let face = emojis::lookup("raised_eyebrow").unwrap();
//!
//! assert_eq!(face.as_str(), "\u{1F928}");
//! assert_eq!(face.name(), "face with raised eyebrow");
//! assert_eq!(face.group(), emojis::Group::SmileysAndEmotion);
//! assert_eq!(face.shortcode().unwrap(), "raised_eyebrow");
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

#![no_std]

mod generated;

use core::cmp;
use core::convert;
use core::fmt;

pub use crate::generated::Group;

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
    aliases: Option<&'static [&'static str]>,
    variations: &'static [&'static str],
    skin_tones: &'static [&'static str],
}

impl Emoji {
    /// Returns this emoji as a string.
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

    /// Returns this emoji's GitHub shortcode.
    ///
    /// See [gemoji] for more information.
    ///
    /// # Examples
    ///
    /// ```
    /// let thinking = emojis::lookup("🤔").unwrap();
    /// assert_eq!(thinking.shortcode().unwrap(), "thinking");
    /// ```
    ///
    /// [gemoji]: https://github.com/github/gemoji
    pub fn shortcode(&self) -> Option<&str> {
        self.aliases.and_then(|aliases| aliases.first().copied())
    }
}

impl cmp::PartialEq<str> for Emoji {
    fn eq(&self, s: &str) -> bool {
        cmp::PartialEq::eq(self.as_str(), s)
    }
}

impl cmp::PartialEq<&str> for Emoji {
    fn eq(&self, s: &&str) -> bool {
        cmp::PartialEq::eq(self.as_str(), *s)
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

impl fmt::Display for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
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

/// Lookup an emoji by Unicode value or shortcode.
///
/// # Examples
///
/// ```
/// let rocket = emojis::lookup("🚀").unwrap();
/// assert_eq!(rocket.shortcode(), Some("rocket"));
///
/// let rocket = emojis::lookup("rocket").unwrap();
/// assert_eq!(rocket, "🚀");
/// ```
pub fn lookup(query: &str) -> Option<&'static Emoji> {
    crate::generated::EMOJIS.iter().find(|&e| {
        e == query
            || e.variations.contains(&query)
            || e.skin_tones.contains(&query)
            || e.aliases
                .map(|aliases| aliases.contains(&query))
                .unwrap_or(false)
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    use core::fmt::Write;

    #[test]
    fn emoji_partial_eq_str() {
        assert_eq!(lookup("😀").unwrap(), "😀");
    }

    #[test]
    fn emoji_ordering() {
        let grinning_face = lookup("😀");
        let winking_face = lookup("😉");
        assert!(grinning_face < winking_face);
        assert!(winking_face > grinning_face);
        assert_eq!(grinning_face, lookup("😀"));
    }

    #[test]
    fn emoji_display() {
        let mut buf = String::<[u8; 4]>::default();
        let grinning_face = lookup("😀").unwrap();
        write!(buf, "{}", grinning_face).unwrap();
        assert_eq!(buf.as_str(), "😀");
    }

    #[test]
    fn lookup_variation() {
        assert_eq!(lookup("☹"), lookup("☹️"));
    }

    #[test]
    fn lookup_skin_tone() {
        assert_eq!(lookup("🙏🏽"), lookup("🙏"));
    }

    // Test utilities

    /// A stack allocated string that supports formatting.
    #[derive(Default)]
    struct String<T> {
        buf: T,
        pos: usize,
    }

    impl<const N: usize> String<[u8; N]> {
        fn as_str(&self) -> &str {
            core::str::from_utf8(&self.buf[..self.pos]).unwrap()
        }
    }

    impl<const N: usize> fmt::Write for String<[u8; N]> {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            let end = self.pos + bytes.len();
            if end > self.buf.len() {
                panic!("buffer overflow");
            }
            self.buf[self.pos..end].copy_from_slice(bytes);
            self.pos = end;
            Ok(())
        }
    }
}
