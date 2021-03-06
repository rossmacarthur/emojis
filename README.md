# emojis

Lookup, iterate, and search emojis.

### Features

- Lookup up emoji by Unicode value.
- Lookup up emoji by GitHub / Slack shortcode. (*not implemented yet*)
- Iterate over emojis in recommended order.
- Iterate over emojis in an emoji group. E.g. "Smileys & Emotion" or "Flags".
- Fuzzy search all emojis.
- Base on the latest Unicode emoji spec (v13.1).

## Examples

```rust
// lookup any emoji
let face = emojis::lookup("🤨").unwrap();
assert_eq!(face.as_str(), "\u{1F928}");
assert_eq!(face.name(), "face with raised eyebrow");
assert_eq!(face.group(), emojis::Group::SmileysAndEmotion);

// iterate over all the emojis.
let emoji = emojis::iter().next().unwrap();
assert_eq!(emoji, "😀");

// iterate over all the emojis in a group.
let emoji = emojis::Group::FoodAndDrink.emojis().next().unwrap();
assert_eq!(emoji, "🍇");

// fuzzy search for emojis.
let emoji = emojis::search("rket").next().unwrap();
assert_eq!(emoji, "🚀");
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
