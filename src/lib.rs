#![no_std]

use core::ops;

/// Represents an emoji, as defined by the Unicode standard.
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    /// let rocket = Emoji::new("🚀");
    /// assert_eq!(rocket.as_str(), "🚀")
    /// ```
    pub const fn as_str(&self) -> &str {
        &self.0
    }
}

impl ops::Deref for Emoji {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emoji_new() {
        const GRINNING_FACE: &Emoji = emoji!("\u{1f600}");
        assert_eq!(GRINNING_FACE, Emoji::new("😀"));
    }
}
