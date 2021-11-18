//! ✨ Lookup and iterate over emoji names, shortcodes, and groups.
//!
//! # Examples
//!
//! Lookup any emoji by Unicode value or GitHub shortcode.
//! ```
//! let hand = emojis::lookup("🤌").unwrap();
//! // Or
//! let hand = emojis::lookup("pinched_fingers").unwrap();
//!
//! assert_eq!(hand.as_str(), "\u{1f90c}");
//! assert_eq!(hand.name(), "pinched fingers");
//! assert_eq!(hand.group(), emojis::Group::PeopleAndBody);
//! assert_eq!(hand.shortcode().unwrap(), "pinched_fingers");
//! assert_eq!(hand.skin_tone().unwrap(), emojis::SkinTone::Default);
//! ```
//!
//! Iterate over all the emojis.
//! ```
//! let smiley = emojis::iter().next().unwrap();
//! assert_eq!(smiley, "😀");
//! ```
//!
//! Iterate over all the emojis in a group.
//! ```
//! let grapes = emojis::Group::FoodAndDrink.emojis().next().unwrap();
//! assert_eq!(grapes, "🍇");
//! ```
//!
//! Iterate over the skin tones for an emoji.
//!
//! ```
//! let raised_hands = emojis::lookup("🙌🏼").unwrap();
//! let iter = raised_hands.skin_tones().unwrap();
//! let skin_tones: Vec<_> = iter.map(emojis::Emoji::as_str).collect();
//! assert_eq!(skin_tones, ["🙌", "🙌🏻", "🙌🏼", "🙌🏽", "🙌🏾", "🙌🏿"]);
//! ```

#![no_std]

#[cfg(test)]
extern crate alloc;

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
    id: u16,
    emoji: &'static str,
    name: &'static str,
    group: Group,
    skin_tone: Option<(u16, SkinTone)>,
    aliases: Option<&'static [&'static str]>,
    variations: &'static [&'static str],
}

/// Represents the skin tone of an emoji.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkinTone {
    Default,
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
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

    /// Returns the skin tone of this emoji.
    ///
    /// # Examples
    ///
    /// ```
    /// use emojis::SkinTone;
    ///
    /// let peace = emojis::lookup("✌️").unwrap();
    /// assert_eq!(peace.skin_tone(), Some(SkinTone::Default));
    ///
    /// let peace = emojis::lookup("✌🏽").unwrap();
    /// assert_eq!(peace.skin_tone(), Some(SkinTone::Medium));
    /// ```
    ///
    /// For emojis where skin tones are not applicable this will be `None`.
    ///
    /// ```
    /// let cool = emojis::lookup("😎").unwrap();
    /// assert!(cool.skin_tone().is_none());
    /// ```
    pub fn skin_tone(&self) -> Option<SkinTone> {
        self.skin_tone.map(|(_, v)| v)
    }

    /// Returns an iterator over the emoji and all the related skin tone emojis.
    ///
    /// # Examples
    ///
    /// ```
    /// use emojis::Emoji;
    ///
    /// let luck = emojis::lookup("🤞🏼").unwrap();
    /// let tones: Vec<_> = luck.skin_tones().unwrap().map(Emoji::as_str).collect();
    /// assert_eq!(tones, ["🤞", "🤞🏻", "🤞🏼", "🤞🏽", "🤞🏾", "🤞🏿"]);
    /// ```
    ///
    /// For emojis where skin tones are not applicable this will return `None`.
    ///
    /// ```
    /// let cool = emojis::lookup("😎").unwrap();
    /// assert!(cool.skin_tones().is_none());
    /// ```
    pub fn skin_tones(&self) -> Option<impl Iterator<Item = &'static Self>> {
        let (id, _) = self.skin_tone?;
        Some(crate::generated::EMOJIS[id as usize..].iter().take(6))
    }

    /// Returns a version of this emoji that has the given skin tone.
    ///
    /// # Examples
    ///
    /// ```
    /// use emojis::SkinTone;
    ///
    /// let peace = emojis::lookup("🙌🏼")
    ///     .unwrap()
    ///     .with_skin_tone(SkinTone::MediumDark)
    ///     .unwrap();
    /// assert_eq!(peace, emojis::lookup("🙌🏾").unwrap());
    /// ```
    ///
    /// For emojis where skin tones are not applicable this will be `None`.
    ///
    /// ```
    /// use emojis::SkinTone;
    ///
    /// let cool = emojis::lookup("😎").unwrap();
    /// assert!(cool.with_skin_tone(SkinTone::Medium).is_none());
    /// ```
    pub fn with_skin_tone(&self, skin_tone: SkinTone) -> Option<&'static Self> {
        self.skin_tones()?
            .find(|emoji| emoji.skin_tone().unwrap() == skin_tone)
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
/// - Ordered by Unicode CLDR data.
/// - Excludes skin tones.
///
/// # Examples
///
/// ```
/// let mut iter = emojis::iter();
/// assert_eq!(iter.next().unwrap(), "😀");
/// ```
pub fn iter() -> impl Iterator<Item = &'static Emoji> {
    crate::generated::EMOJIS
        .iter()
        .filter(|emoji| emoji.skin_tone().is_none())
}

/// Lookup an emoji by Unicode value or shortcode.
///
/// # Examples
///
/// ```
/// let rocket = emojis::lookup("🚀").unwrap();
/// assert_eq!(rocket.shortcode().unwrap(), "rocket");
///
/// let rocket = emojis::lookup("rocket").unwrap();
/// assert_eq!(rocket, "🚀");
/// ```
pub fn lookup(query: &str) -> Option<&'static Emoji> {
    crate::generated::EMOJIS.iter().find(|&e| {
        e == query
            || e.variations.contains(&query)
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

    use alloc::format;
    use alloc::vec::Vec;

    #[test]
    fn emoji_partial_eq_str() {
        assert_eq!(lookup("😀").unwrap(), "😀");
    }

    #[test]
    fn emoji_display() {
        let buf = format!("{}", lookup("😀").unwrap());
        assert_eq!(buf.as_str(), "😀");
    }

    #[test]
    fn lookup_variation() {
        assert_eq!(lookup("☹"), lookup("☹️"));
    }

    #[test]
    fn skin_tones() {
        let skin_tones = [
            SkinTone::Default,
            SkinTone::Light,
            SkinTone::MediumLight,
            SkinTone::Medium,
            SkinTone::MediumDark,
            SkinTone::Dark,
        ];
        for emoji in iter() {
            match emoji.skin_tone() {
                Some(_) => {
                    let emojis: Vec<_> = emoji.skin_tones().unwrap().collect();
                    assert_eq!(emojis.len(), 6);
                    let default = emojis[0];
                    for (emoji, skin_tone) in emojis.into_iter().zip(skin_tones) {
                        assert_eq!(emoji.skin_tone().unwrap(), skin_tone);
                        assert_eq!(emoji.with_skin_tone(SkinTone::Default).unwrap(), default);
                    }
                }
                None => {
                    assert!(emoji.skin_tones().is_none());
                }
            }
        }
    }
}
