#![no_std]

use core::cmp;
use core::convert;
use core::ops;
use core::slice;

/// Represents an emoji, as defined by the Unicode standard.
#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Emoji(str);

/// Macro to construct a `const` [`Emoji`].
///
/// This is required until we can make [`Emoji::new()`] `const`.
macro_rules! emoji {
    ($inner:expr) => {{
        let inner: &str = $inner;
        let emoji: &$crate::Emoji = unsafe { core::mem::transmute(inner) };
        emoji
    }};
}

impl Emoji {
    /// Construct a new `Emoji`.
    ///
    /// For a `const` version of this use [`new!()`].
    ///
    /// # Examples
    ///
    /// ```
    /// # use emojis::Emoji;
    /// #
    /// let rocket: &Emoji = Emoji::new("🚀");
    /// ```
    #[cfg(test)]
    fn new(inner: &str) -> &Self {
        let ptr = inner as *const str as *const Self;
        // Safety: `Self` is #[repr(transparent)]
        unsafe { &*ptr }
    }

    /// Return a reference to the underlying string.
    ///
    /// `Emoji` also implements [`Deref`](#impl-Deref) to [`str`] so this
    /// shouldn't be needed too often.
    ///
    /// # Examples
    ///
    /// ```
    /// # use emojis::Emoji;
    /// #
    /// let rocket = emojis::lookup("🚀").unwrap();
    /// assert_eq!(rocket.as_str(), "🚀")
    /// ```
    #[inline]
    pub const fn as_str(&self) -> &str {
        &self.0
    }
}

impl cmp::PartialEq<str> for &Emoji {
    fn eq(&self, s: &str) -> bool {
        PartialEq::eq(self.as_str(), s)
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
pub fn iter() -> slice::Iter<'static, &'static Emoji> {
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
pub fn lookup(emoji: &str) -> Option<&Emoji> {
    generated::EMOJIS.iter().copied().find(|e| e == emoji)
}

mod generated;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_new() {
        const GRINNING_FACE: &Emoji = emoji!("\u{1f600}");
        assert_eq!(GRINNING_FACE, Emoji::new("😀"));
    }
}
