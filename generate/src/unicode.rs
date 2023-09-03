//! Fetch and parse raw emoji data from Unicode.org.

use std::iter;
use std::str;
use std::str::FromStr;

use anyhow::{bail, ensure, Context, Result};
use heck::CamelCase;
use indexmap::IndexMap;
use then::Some;

const URL: &str = "https://unicode.org/Public/emoji/15.1/emoji-test.txt";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    FullyQualified,
    MinimallyQualified,
    Unqualified,
    Component,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnicodeVersion {
    major: u32,
    minor: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Emoji {
    emoji: String,
    name: String,
    unicode_version: UnicodeVersion,
    status: Status,
    skin_tones: usize,
    skin_tone: Option<SkinTone>,
    variations: Vec<String>,
}

pub type ParsedData = IndexMap<String, IndexMap<String, Vec<Emoji>>>;

type Lines<'a> = iter::Peekable<str::Lines<'a>>;

fn fetch_emoji_data() -> Result<String> {
    let mut buf = Vec::new();
    let mut easy = curl::easy::Easy::new();
    easy.fail_on_error(true)?;
    easy.follow_location(true)?;
    easy.url(URL)?;
    {
        let mut transfer = easy.transfer();
        transfer.write_function(|data| {
            buf.extend_from_slice(data);
            Ok(data.len())
        })?;
        transfer.perform()?;
    }
    Ok(String::from_utf8(buf)?)
}

trait LinesExt {
    fn consume_until_starts_with(&mut self, prefix: &str, stop: Option<&str>) -> Option<String>;

    fn next_group(&mut self) -> Option<String> {
        self.consume_until_starts_with("# group: ", None)
    }

    fn next_subgroup(&mut self) -> Option<String> {
        self.consume_until_starts_with("# subgroup: ", Some("# group: "))
    }
}

impl LinesExt for Lines<'_> {
    fn consume_until_starts_with(&mut self, prefix: &str, stop: Option<&str>) -> Option<String> {
        loop {
            match self.peek() {
                Some(line) => {
                    if let Some(prefix) = stop {
                        if line.starts_with(prefix) {
                            return None;
                        }
                    }
                    if line.starts_with(prefix) {
                        let name = line.strip_prefix(prefix).unwrap().to_owned();
                        self.next();
                        return Some(name);
                    }
                    self.next();
                }
                None => return None,
            }
        }
    }
}

fn parse_code_point(code_point: &str) -> Result<char> {
    let scalar = u32::from_str_radix(code_point, 16).context("not hex")?;
    std::char::from_u32(scalar).context("not Unicode scalar value")
}

impl FromStr for UnicodeVersion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        ensure!(s.starts_with('E'));
        let (major, minor) = s[1..].split_once('.').context("decimal")?;
        let major = major.parse()?;
        let minor = minor.parse()?;
        Ok(Self { major, minor })
    }
}

fn skin_tones() -> impl Iterator<Item = SkinTone> {
    [
        SkinTone::Light,
        SkinTone::MediumLight,
        SkinTone::Medium,
        SkinTone::MediumDark,
        SkinTone::Dark,
    ]
    .into_iter()
}

impl SkinTone {
    fn code_point(self) -> char {
        match self {
            Self::Light => '\u{1f3fb}',
            Self::MediumLight => '\u{1f3fc}',
            Self::Medium => '\u{1f3fd}',
            Self::MediumDark => '\u{1f3fe}',
            Self::Dark => '\u{1f3ff}',
            _ => unreachable!(),
        }
    }
}

impl Emoji {
    fn from_line(line: &str) -> Result<Self> {
        let (code_points, rest) = line.split_once(';').context("expected code points")?;
        let (status, rest) = rest.split_once('#').context("expected status")?;
        let mut rest = rest.trim().splitn(3, ' ');
        let actual = rest.next().context("expected emoji")?;
        let unicode_version = rest.next().context("expected unicode version")?;
        let name = rest.next().context("expected name")?;

        let emoji: String = code_points
            .trim()
            .split(' ')
            .map(parse_code_point)
            .collect::<Result<_>>()?;
        if emoji != actual {
            bail!("emoji mismatch");
        }
        let unicode_version =
            UnicodeVersion::from_str(unicode_version).context("failed to parse unicode version")?;
        let name = name.trim().to_owned();
        let status = match status.trim() {
            "fully-qualified" => Status::FullyQualified,
            "minimally-qualified" => Status::MinimallyQualified,
            "unqualified" => Status::Unqualified,
            "component" => Status::Component,
            s => bail!("unrecognized status `{}`", s),
        };
        let skin_tones: Vec<_> = emoji
            .chars()
            .filter_map(|c| skin_tones().find_map(|st| (st.code_point() == c).some(st)))
            .collect();

        use SkinTone::*;
        let skin_tone = match *skin_tones.as_slice() {
            [] => None,
            [a] => Some(a),
            [Light, MediumLight] => Some(LightAndMediumLight),
            [Light, Medium] => Some(LightAndMedium),
            [Light, MediumDark] => Some(LightAndMediumDark),
            [Light, Dark] => Some(LightAndDark),
            [MediumLight, Light] => Some(MediumLightAndLight),
            [MediumLight, Medium] => Some(MediumLightAndMedium),
            [MediumLight, MediumDark] => Some(MediumLightAndMediumDark),
            [MediumLight, Dark] => Some(MediumLightAndDark),
            [Medium, Light] => Some(MediumAndLight),
            [Medium, MediumLight] => Some(MediumAndMediumLight),
            [Medium, MediumDark] => Some(MediumAndMediumDark),
            [Medium, Dark] => Some(MediumAndDark),
            [MediumDark, Light] => Some(MediumDarkAndLight),
            [MediumDark, MediumLight] => Some(MediumDarkAndMediumLight),
            [MediumDark, Medium] => Some(MediumDarkAndMedium),
            [MediumDark, Dark] => Some(MediumDarkAndDark),
            [Dark, Light] => Some(DarkAndLight),
            [Dark, MediumLight] => Some(DarkAndMediumLight),
            [Dark, Medium] => Some(DarkAndMedium),
            [Dark, MediumDark] => Some(DarkAndMediumDark),
            [a, b] if a == b => Some(a),
            _ => bail!("unrecognized skin tone combination, {:?}", skin_tones),
        };

        Ok(Self {
            emoji,
            name,
            unicode_version,
            status,
            skin_tone,
            skin_tones: 1,
            variations: Vec::new(),
        })
    }

    pub fn as_str(&self) -> &str {
        &self.emoji
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn unicode_version(&self) -> &UnicodeVersion {
        &self.unicode_version
    }

    pub fn skin_tone(&self) -> Option<SkinTone> {
        self.skin_tone
    }

    pub fn skin_tones(&self) -> usize {
        self.skin_tones
    }

    pub fn variations(&self) -> &[String] {
        &self.variations
    }
}

fn parse_emoji_data(data: &str) -> Result<ParsedData> {
    let mut parsed_data = ParsedData::default();
    let mut lines = data.lines().peekable();
    while let Some(group) = lines.next_group() {
        let group = group.replace('&', "And").to_camel_case();
        while let Some(subgroup) = lines.next_subgroup() {
            for line in &mut lines {
                if line.is_empty() {
                    break;
                }
                let emoji = Emoji::from_line(line)?;

                let ctx = || {
                    format!(
                        "failed to find fully qualified variation for `{}`",
                        emoji.name()
                    )
                };

                match emoji.status {
                    Status::Component => continue,

                    Status::MinimallyQualified | Status::Unqualified => {
                        // find fully qualified variation
                        parsed_data[&group][&subgroup]
                            .iter_mut()
                            .last()
                            .with_context(ctx)?
                            .variations
                            .push(emoji.emoji);
                    }

                    Status::FullyQualified => {
                        match emoji.skin_tone {
                            None | Some(SkinTone::Default) => {
                                // normal emoji, simply add
                                parsed_data
                                    .entry(group.clone())
                                    .or_default()
                                    .entry(subgroup.clone())
                                    .or_insert_with(Vec::new)
                                    .push(emoji);
                            }

                            Some(skin_tone) => {
                                // find the default skin tone to set it
                                let i = {
                                    let (i, def) = parsed_data[&group][&subgroup]
                                        .iter_mut()
                                        .enumerate()
                                        .rev()
                                        .find(|(_, e)| {
                                            matches!(e.skin_tone, None | Some(SkinTone::Default))
                                                && emoji.name.contains(&emoji.name)
                                        })
                                        .with_context(ctx)?;
                                    def.skin_tone = Some(SkinTone::Default);
                                    def.skin_tones += 1;
                                    i
                                };

                                // now add this emoji to the list making sure to
                                // be consistent with the ordering of skin tones
                                let v = parsed_data
                                    .entry(group.clone())
                                    .or_default()
                                    .entry(subgroup.clone())
                                    .or_insert_with(Vec::new);
                                let j = v[i..].partition_point(|e| e.skin_tone < Some(skin_tone));
                                v.insert(i + j, emoji);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(parsed_data)
}

pub fn fetch_and_parse_emoji_data() -> Result<ParsedData> {
    let data = fetch_emoji_data()?;
    let parsed_data = parse_emoji_data(&data)?;
    Ok(parsed_data)
}
