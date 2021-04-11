# emojis

[![Crates.io Version](https://img.shields.io/crates/v/emojis.svg)](https://crates.io/crates/emojis)
[![Docs.rs Latest](https://img.shields.io/badge/docs.rs-latest-blue.svg)](https://docs.rs/emojis)
[![Build Status](https://img.shields.io/github/workflow/status/rossmacarthur/emojis/build/trunk)](https://github.com/rossmacarthur/emojis/actions?query=workflow%3Abuild)

✨ Lookup and iterate over emoji names, shortcodes, and groups.

### Features

- Lookup up emoji by Unicode value.
- Lookup up emoji by GitHub shortcode.
- Iterate over emojis in recommended order.
- Iterate over emojis in an emoji group. E.g. "Smileys & Emotion" or "Flags".
- Based on the latest Unicode emoji spec (v13.1).

## Examples

```rust
// lookup any emoji by Unicode value
let face = emojis::lookup("🤨").unwrap();
// or GitHub shortcode
let face = emojis::lookup("raised_eyebrow").unwrap();

assert_eq!(face.as_str(), "\u{1F928}");
assert_eq!(face.name(), "face with raised eyebrow");
assert_eq!(face.group(), emojis::Group::SmileysAndEmotion);
assert_eq!(face.shortcode().unwrap(), "raised_eyebrow");

// iterate over all the emojis.
let emoji = emojis::iter().next().unwrap();
assert_eq!(emoji, "😀");

// iterate over all the emojis in a group.
let emoji = emojis::Group::FoodAndDrink.emojis().next().unwrap();
assert_eq!(emoji, "🍇");
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
