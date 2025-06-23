//! Fetch and parse raw emoji data from Unicode.org.

mod data;
mod variations;

use std::str;

use anyhow::bail;
use anyhow::Context as _;
use anyhow::Result;
use serde::Serialize;

pub use crate::unicode::data::Group;
use crate::unicode::data::Status;

pub struct ParsedData {
    pub emojis: Vec<Emoji>,
    pub variations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Emoji {
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
    let mut emojis: Vec<Emoji> = Vec::new();
    let variations = variations::parse()?;

    for entry in data::parse()? {
        if let Group::Component = entry.group {
            continue;
        }

        match entry.status {
            Status::Component => continue,
            Status::MinimallyQualified | Status::Unqualified => {
                // find fully qualified variation
                emojis
                    .last_mut()
                    .with_context(|| {
                        format!(
                            "failed to find fully qualified variation for '{}'",
                            entry.name
                        )
                    })?
                    .variations
                    .push(entry.emoji);
            }
            Status::FullyQualified => {
                let skin_tone = parse_skin_tone(&entry)?;

                match skin_tone {
                    None | Some(SkinTone::Default) => {
                        // normal emoji, simply add
                        emojis.push(Emoji {
                            entry,
                            skin_tones: 1,
                            skin_tone,
                            variations: Vec::new(),
                        });
                    }

                    Some(skin_tone) => {
                        // find the default skin tone to set it
                        let i = {
                            let (i, def) = emojis
                                .iter_mut()
                                .enumerate()
                                .rev()
                                .find(|(_, e)| {
                                    matches!(e.skin_tone, None | Some(SkinTone::Default))
                                        && e.entry.group == entry.group
                                        && e.entry.subgroup == entry.subgroup
                                })
                                .with_context(|| {
                                    format!(
                                        "failed to find the default skin tone for '{}'",
                                        entry.name
                                    )
                                })?;
                            def.skin_tone = Some(SkinTone::Default);
                            def.skin_tones += 1;
                            i
                        };

                        // now add this emoji to the list making sure to
                        // be consistent with the ordering of skin tones
                        let j = emojis[i..].partition_point(|e| e.skin_tone < Some(skin_tone));
                        emojis.insert(
                            i + j,
                            Emoji {
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
