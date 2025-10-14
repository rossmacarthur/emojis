//! Fetch and parse raw emoji data from Unicode.org.

mod data;
mod variations;

use std::collections::HashMap;
use std::str;

use anyhow::bail;
use anyhow::Context as _;
use anyhow::Result;
use serde::Serialize;

pub use crate::unicode::data::Group;
use crate::unicode::data::Status;

pub const VERSION_MAJOR: &str = "17";
pub const VERSION_MINOR: &str = "0";
pub const VERSION_PATCH: &str = "0";

pub struct ParsedData {
    pub emojis: Vec<Emoji>,
    pub variations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Emoji {
    pub index: usize,
    pub entry: data::Entry,
    pub skin_tones: usize,
    pub skin_tone: Option<SkinTone>,
    pub variations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum SkinTone {
    Default,
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
    LightAndMediumLight,
    LightAndMedium,
    LightAndMediumDark,
    LightAndDark,
    MediumLightAndLight,
    MediumLightAndMedium,
    MediumLightAndMediumDark,
    MediumLightAndDark,
    MediumAndLight,
    MediumAndMediumLight,
    MediumAndMediumDark,
    MediumAndDark,
    MediumDarkAndLight,
    MediumDarkAndMediumLight,
    MediumDarkAndMedium,
    MediumDarkAndDark,
    DarkAndLight,
    DarkAndMediumLight,
    DarkAndMedium,
    DarkAndMediumDark,
}

impl Emoji {
    pub fn as_str(&self) -> &str {
        &self.entry.emoji
    }
}

pub fn build() -> Result<ParsedData> {
    let mut emojis_map: HashMap<String, Vec<Emoji>> = HashMap::new();

    let variations = variations::parse()?;

    for (index, entry) in data::parse()?.into_iter().enumerate() {
        if let Group::Component = entry.group {
            continue;
        }

        let base_name = parse_base_name(&entry.name);

        match entry.status {
            Status::Component => unreachable!(),
            Status::MinimallyQualified | Status::Unqualified => {
                // find fully qualified variation
                let base = emojis_map
                    .get_mut(&base_name)
                    .and_then(|e| e.last_mut())
                    .with_context(|| {
                        format!(
                            "failed to find fully qualified variation for '{}'",
                            entry.name
                        )
                    })?;
                assert_eq!(base.entry.status, Status::FullyQualified);
                assert_eq!(base.entry.group, entry.group);
                base.variations.push(entry.emoji);
            }
            Status::FullyQualified => {
                let skin_tone = parse_skin_tone(&entry)?;

                match skin_tone {
                    None | Some(SkinTone::Default) => {
                        // normal emoji, simply add to the list
                        let emojis = emojis_map.entry(base_name.clone()).or_default();
                        assert!(emojis.is_empty(), "base emoji not the first entry!");
                        emojis.push(Emoji {
                            index,
                            entry,
                            skin_tones: 1,
                            skin_tone,
                            variations: Vec::new(),
                        });
                    }

                    Some(skin_tone) => {
                        // find the default skin tone to set it
                        let emojis = emojis_map.get_mut(&base_name).with_context(|| {
                            format!(
                                "failed to find the base emoji for '{}' (base: {})",
                                entry.name, &base_name
                            )
                        })?;

                        emojis[0].skin_tone = Some(SkinTone::Default);
                        emojis[0].skin_tones += 1;
                        assert!(emojis[0].skin_tones <= 26);

                        // making sure to be consistent with the ordering of skin tones
                        let i = emojis.partition_point(|e| e.skin_tone < Some(skin_tone));
                        emojis.insert(
                            i,
                            Emoji {
                                index,
                                entry,
                                skin_tones: 1,
                                skin_tone: Some(skin_tone),
                                variations: Vec::new(),
                            },
                        );
                    }
                }
            }
        }
    }

    // now flatten the emoji map, ensuring the order is consistent with the
    // original data (.index)
    let emojis = {
        let mut grouped: Vec<_> = emojis_map.into_values().collect();
        grouped.sort_by_key(|e| e[0].index);
        for emojis in &mut grouped {
            emojis.sort_by_key(|e| e.skin_tone);
        }
        grouped.into_iter().flatten().collect()
    };

    Ok(ParsedData { emojis, variations })
}

fn parse_skin_tone(entry: &data::Entry) -> Result<Option<SkinTone>> {
    use SkinTone::*;

    let skin_tones: Vec<_> = entry
        .emoji
        .chars()
        .filter_map(|c| match c {
            '\u{1f3fb}' => Some(Light),
            '\u{1f3fc}' => Some(MediumLight),
            '\u{1f3fd}' => Some(Medium),
            '\u{1f3fe}' => Some(MediumDark),
            '\u{1f3ff}' => Some(Dark),
            _ => None,
        })
        .collect();

    let skin_tone = match *skin_tones.as_slice() {
        [] => return Ok(None),
        [a] => a,
        [Light, MediumLight] => LightAndMediumLight,
        [Light, Medium] => LightAndMedium,
        [Light, MediumDark] => LightAndMediumDark,
        [Light, Dark] => LightAndDark,
        [MediumLight, Light] => MediumLightAndLight,
        [MediumLight, Medium] => MediumLightAndMedium,
        [MediumLight, MediumDark] => MediumLightAndMediumDark,
        [MediumLight, Dark] => MediumLightAndDark,
        [Medium, Light] => MediumAndLight,
        [Medium, MediumLight] => MediumAndMediumLight,
        [Medium, MediumDark] => MediumAndMediumDark,
        [Medium, Dark] => MediumAndDark,
        [MediumDark, Light] => MediumDarkAndLight,
        [MediumDark, MediumLight] => MediumDarkAndMediumLight,
        [MediumDark, Medium] => MediumDarkAndMedium,
        [MediumDark, Dark] => MediumDarkAndDark,
        [Dark, Light] => DarkAndLight,
        [Dark, MediumLight] => DarkAndMediumLight,
        [Dark, Medium] => DarkAndMedium,
        [Dark, MediumDark] => DarkAndMediumDark,
        [a, b] if a == b => a,
        _ => bail!("unrecognized skin tone combination, {:?}", skin_tones),
    };

    Ok(Some(skin_tone))
}

/// Given an emoji name parse the expected base name.
///
/// See the test below for examples
fn parse_base_name(name: &str) -> String {
    let mut it = name.rsplitn(2, ':');
    let right = it.next().unwrap().trim();
    match it.next() {
        Some(left) => {
            let right = right
                .split(',')
                .map(str::trim)
                .filter(|part| !part.ends_with("skin tone"))
                .collect::<Vec<_>>()
                .join(", ");
            if right.is_empty() {
                return left.to_owned();
            }
            if right == "person, person" {
                return left.to_owned();
            }
            format!("{left}: {right}")
        }
        None => right.to_owned(),
    }
}

#[test]
fn test_parse_base_name() {
    struct Case {
        name: &'static str,
        exp: &'static str,
    }
    for case in [
        Case {
            name: "grinning face",
            exp: "grinning face",
        },
        Case {
            name: "man: blond hair",
            exp: "man: blond hair",
        },
        Case {
            name: "handshake: light skin tone, medium-light skin tone",
            exp: "handshake",
        },
        Case {
            name: "kiss: woman, man, light skin tone",
            exp: "kiss: woman, man",
        },
        Case {
            name: "kiss: person, person, light skin tone, medium-light skin tone",
            exp: "kiss",
        },
    ] {
        assert_eq!(parse_base_name(case.name), case.exp);
    }
}
