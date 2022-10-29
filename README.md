# emojis

[![Crates.io Version](https://img.shields.io/crates/v/emojis.svg)](https://crates.io/crates/emojis)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/emojis)
[![Build Status](https://img.shields.io/github/workflow/status/rossmacarthur/emojis/build/trunk)](https://github.com/rossmacarthur/emojis/actions?query=workflow%3Abuild)

✨ Lookup and iterate over emoji names, shortcodes, and groups.

## Features

- Lookup up emoji by Unicode value
- Lookup up emoji by [GitHub shortcode][gemoji]
- Iterate over emojis in recommended order
- Iterate over emojis in an emoji group, e.g. "Smileys & Emotion" or "Flags"
- Iterate over the skin tones for an emoji
- Uses Unicode v15.0 emoji specification

## Getting started

First, add the `emojis` crate to your Cargo manifest.

```sh
cargo add emojis
```

Simply use the `get()` function to lookup emojis by Unicode value.
```rust
let rocket = emojis::get("🚀")?;
```

Or the `get_by_shortcode()` function to lookup emojis by [gemoji] shortcode.

```rust
let rocket = emojis::get_by_shortcode("rocket")?;
```

## MSRV

Currently the minimum supported Rust version is 1.60 due to the dependency on
[phf](https://crates.io/crates/phf). The policy of this crate is to only
increase the MSRV in a breaking release.

## Examples

The returned `Emoji` struct has various information about the emoji.
```rust
let hand = emojis::get("🤌")?;
assert_eq!(hand.as_str(), "\u{1f90c}");
assert_eq!(hand.name(), "pinched fingers");
assert_eq!(hand.unicode_version(), emojis::UnicodeVersion::new(13, 0));
assert_eq!(hand.group(), emojis::Group::PeopleAndBody);
assert_eq!(hand.shortcode()?, "pinched_fingers");
assert_eq!(hand.skin_tone()?, emojis::SkinTone::Default);
```

Another common operation is iterating over the skin tones of an emoji.
```rust
let raised_hands = emojis::get("🙌🏼")?;
let skin_tones: Vec<_> = raised_hands.skin_tones()?.map(|e| e.as_str()).collect();
assert_eq!(skin_tones, ["🙌", "🙌🏻", "🙌🏼", "🙌🏽", "🙌🏾", "🙌🏿"]);
```

You can use the `iter()` function to iterate over all emojis (only includes the
default skin tone versions).
```rust
let smiley = emojis::iter().next()?;
assert_eq!(smiley, "😀");
```

It is recommended to filter the list by the maximum Unicode version that you
wish to support.
```rust
let iter = emojis::iter().filter(|e| {
    e.unicode_version() < emojis::UnicodeVersion::new(13, 0)
});
```

Using the `Group` enum you can iterate over all emojis in a group.
```rust
let grapes = emojis::Group::FoodAndDrink.emojis().next()?;
assert_eq!(grapes, "🍇");
```

Checkout [examples/replace.rs](./examples/replace.rs) for an example that
replaces [gemoji] names in text.

```sh
$ echo "launch :rocket:" | cargo run --example replace
launch 🚀
```

[gemoji]: https://github.com/github/gemoji

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
