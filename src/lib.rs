//! ✨ Lookup and iterate over emoji names, shortcodes, and groups.
//!
//! # Examples
//!
//! Lookup any emoji by Unicode value or GitHub shortcode.
//! ```
//! let hand = emojis::get("🤌").unwrap();
//! // or
//! let hand = emojis::get_by_shortcode("pinched_fingers").unwrap();
//!
//! assert_eq!(hand.as_str(), "\u{1f90c}");
//! assert_eq!(hand.name(), "pinched fingers");
//! assert_eq!(hand.unicode_version(), emojis::UnicodeVersion::new(13, 0));
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
//! Iterate and filter out newer emoji versions.
//! ```
//! let iter = emojis::iter().filter(|e| {
//!     e.unicode_version() < emojis::UnicodeVersion::new(13, 0)
//! });
//! assert_eq!(iter.count(), 1738);
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
//! let raised_hands = emojis::get("🙌🏼").unwrap();
//! let iter = raised_hands.skin_tones().unwrap();
//! let skin_tones: Vec<_> = iter.map(emojis::Emoji::as_str).collect();
//! assert_eq!(skin_tones, ["🙌", "🙌🏻", "🙌🏼", "🙌🏽", "🙌🏾", "🙌🏿"]);
//! ```

#![no_std]

#[cfg(test)]
extern crate alloc;

mod gen;

use core::cmp;
use core::convert;
use core::fmt;
use core::hash;

pub use crate::gen::Group;

/// Represents an emoji.
///
/// See [Unicode.org](https://unicode.org/emoji/charts/full-emoji-list.html) for
/// more information.
#[derive(Debug)]
pub struct Emoji {
    emoji: &'static str,
    name: &'static str,
    unicode_version: UnicodeVersion,
    group: Group,
    skin_tone: Option<(u16, SkinTone)>,
    aliases: Option<&'static [&'static str]>,
}

/// Represents a Unicode version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UnicodeVersion {
    major: u32,
    minor: u32,
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

impl UnicodeVersion {
    /// Construct a new version.
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    pub const fn major(self) -> u32 {
        self.major
    }

    pub const fn minor(self) -> u32 {
        self.minor
    }
}

impl Emoji {
    /// Returns this emoji as a string.
    ///
    /// # Examples
    ///
    /// ```
    /// let rocket = emojis::get("🚀").unwrap();
    /// assert_eq!(rocket.as_str(), "🚀")
    /// ```
    pub const fn as_str(&self) -> &str {
        self.emoji
    }

    /// Returns this emoji as slice of UTF-8 encoded bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// let rocket = emojis::get("🚀").unwrap();
    /// assert_eq!(rocket.as_bytes(), &[0xf0, 0x9f, 0x9a, 0x80]);
    /// ```
    pub const fn as_bytes(&self) -> &[u8] {
        self.emoji.as_bytes()
    }

    /// Returns the CLDR short name for this emoji.
    ///
    /// # Examples
    ///
    /// ```
    /// let cool = emojis::get("😎").unwrap();
    /// assert_eq!(cool.name(), "smiling face with sunglasses");
    /// ```
    pub const fn name(&self) -> &str {
        self.name
    }

    /// Returns the Unicode version this emoji first appeared in.
    ///
    /// # Examples
    ///
    /// ```
    /// use emojis::UnicodeVersion;
    ///
    /// let villain = emojis::get("🦹").unwrap();
    /// assert_eq!(villain.unicode_version(), UnicodeVersion::new(11, 0));
    /// ```
    pub const fn unicode_version(&self) -> UnicodeVersion {
        self.unicode_version
    }

    /// Returns this emoji's group.
    ///
    /// # Examples
    ///
    /// ```
    /// use emojis::Group;
    ///
    /// let flag = emojis::get("🇿🇦").unwrap();
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
    /// let peace = emojis::get("✌️").unwrap();
    /// assert_eq!(peace.skin_tone(), Some(SkinTone::Default));
    ///
    /// let peace = emojis::get("✌🏽").unwrap();
    /// assert_eq!(peace.skin_tone(), Some(SkinTone::Medium));
    /// ```
    ///
    /// For emojis where skin tones are not applicable this will be `None`.
    ///
    /// ```
    /// let cool = emojis::get("😎").unwrap();
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
    /// let luck = emojis::get("🤞🏼").unwrap();
    /// let tones: Vec<_> = luck.skin_tones().unwrap().map(Emoji::as_str).collect();
    /// assert_eq!(tones, ["🤞", "🤞🏻", "🤞🏼", "🤞🏽", "🤞🏾", "🤞🏿"]);
    /// ```
    ///
    /// For emojis where skin tones are not applicable this will return `None`.
    ///
    /// ```
    /// let cool = emojis::get("😎").unwrap();
    /// assert!(cool.skin_tones().is_none());
    /// ```
    pub fn skin_tones(&self) -> Option<impl Iterator<Item = &'static Self>> {
        let (i, _) = self.skin_tone?;
        Some(crate::gen::EMOJIS[i as usize..].iter().take(6))
    }

    /// Returns a version of this emoji that has the given skin tone.
    ///
    /// # Examples
    ///
    /// ```
    /// use emojis::SkinTone;
    ///
    /// let peace = emojis::get("🙌🏼")
    ///     .unwrap()
    ///     .with_skin_tone(SkinTone::MediumDark)
    ///     .unwrap();
    /// assert_eq!(peace, emojis::get("🙌🏾").unwrap());
    /// ```
    ///
    /// For emojis where skin tones are not applicable this will be `None`.
    ///
    /// ```
    /// use emojis::SkinTone;
    ///
    /// let cool = emojis::get("😎").unwrap();
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
    /// let thinking = emojis::get("🤔").unwrap();
    /// assert_eq!(thinking.shortcode().unwrap(), "thinking");
    /// ```
    ///
    /// [gemoji]: https://github.com/github/gemoji
    pub fn shortcode(&self) -> Option<&str> {
        self.aliases.and_then(|aliases| aliases.first().copied())
    }

    /// Returns an iterator over this emoji's GitHub shortcode and aliases.
    ///
    /// See [gemoji] for more information.
    ///
    /// # Examples
    ///
    /// ```
    /// let thinking = emojis::get("🤔").unwrap();
    /// assert_eq!(thinking.aliases().next().unwrap(), "thinking");
    /// ```
    ///
    /// [gemoji]: https://github.com/github/gemoji
    pub fn aliases(&self) -> impl Iterator<Item = &str> + '_ {
        self.aliases.into_iter().flatten().copied()
    }
}

impl cmp::PartialEq<Emoji> for Emoji {
    fn eq(&self, other: &Emoji) -> bool {
        self.emoji == other.emoji
    }
}

impl cmp::PartialEq<str> for Emoji {
    fn eq(&self, s: &str) -> bool {
        self.as_str() == s
    }
}

impl cmp::PartialEq<&str> for Emoji {
    fn eq(&self, s: &&str) -> bool {
        self.as_str() == *s
    }
}

impl cmp::Eq for Emoji {}

impl hash::Hash for Emoji {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.emoji.hash(state);
    }
}

impl convert::AsRef<str> for Emoji {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl convert::AsRef<[u8]> for Emoji {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl fmt::Display for Emoji {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
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
    crate::gen::EMOJIS
        .iter()
        .filter(|emoji| matches!(emoji.skin_tone(), Some(SkinTone::Default) | None))
}

/// Find an emoji by Unicode value.
///
/// # Examples
///
/// ```
/// let rocket = emojis::get("🚀").unwrap();
/// assert_eq!(rocket.shortcode().unwrap(), "rocket");
/// ```
pub fn get(s: &str) -> Option<&'static Emoji> {
    crate::gen::unicode::MAP
        .get(s)
        .map(|&i| &crate::gen::EMOJIS[i])
}

/// Find an emoji by GitHub shortcode.
///
/// # Examples
///
/// ```
/// let rocket = emojis::get_by_shortcode("rocket").unwrap();
/// assert_eq!(rocket, "🚀");
/// ```
pub fn get_by_shortcode(s: &str) -> Option<&'static Emoji> {
    crate::gen::shortcode::MAP
        .get(s)
        .map(|&i| &crate::gen::EMOJIS[i])
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::format;
    use alloc::vec::Vec;

    #[test]
    fn emoji_partial_eq_str() {
        assert_eq!(get("😀").unwrap(), "😀");
    }

    #[test]
    fn emoji_display() {
        let buf = format!("{}", get("😀").unwrap());
        assert_eq!(buf.as_str(), "😀");
    }

    #[test]
    fn version_ordering() {
        assert!(UnicodeVersion::new(13, 0) >= UnicodeVersion::new(12, 0));
        assert!(UnicodeVersion::new(12, 1) >= UnicodeVersion::new(12, 0));
        assert!(UnicodeVersion::new(12, 0) >= UnicodeVersion::new(12, 0));
        assert!(UnicodeVersion::new(12, 0) < UnicodeVersion::new(12, 1));
        assert!(UnicodeVersion::new(11, 0) < UnicodeVersion::new(12, 1));
        assert!(UnicodeVersion::new(11, 0) < UnicodeVersion::new(12, 1));
    }

    #[test]
    fn lookup_variation() {
        assert_eq!(get("☹"), get("☹️"));
    }

    #[test]
    fn iter_only_default_skin_tones() {
        assert!(iter().all(|emoji| matches!(emoji.skin_tone(), Some(SkinTone::Default) | None)));
        assert_ne!(
            iter()
                .filter(|emoji| matches!(emoji.skin_tone(), Some(SkinTone::Default)))
                .count(),
            0
        );
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
                    for (emoji, skin_tone) in emojis.iter().zip(skin_tones) {
                        assert_eq!(emoji.skin_tone().unwrap(), skin_tone, "{:#?}", emojis);
                        assert_eq!(emoji.with_skin_tone(SkinTone::Default).unwrap(), default);
                    }
                }
                None => {
                    assert!(emoji.skin_tones().is_none());
                }
            }
        }
    }

    #[test]
    fn aliases() {
        for emoji in iter() {
            assert_eq!(emoji.shortcode(), emoji.aliases().next());
        }
    }
}
