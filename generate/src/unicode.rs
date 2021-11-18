//! Fetch and parse raw emoji data from Unicode.org.

use std::iter;
use std::str;

use anyhow::{bail, Context, Result};
use heck::CamelCase;
use indexmap::IndexMap;

const URL: &str = "https://unicode.org/Public/emoji/13.1/emoji-test.txt";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    FullyQualified,
    MinimallyQualified,
    Unqualified,
    Component,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkinTone {
    Default,
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Emoji {
    emoji: String,
    name: String,
    status: Status,
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

impl SkinTone {
    fn tones() -> impl Iterator<Item = Self> {
        IntoIterator::into_iter([
            Self::Light,
            Self::MediumLight,
            Self::Medium,
            Self::MediumDark,
            Self::Dark,
        ])
    }

    fn code_point(&self) -> char {
        match self {
            Self::Default => unreachable!(),
            Self::Light => '\u{1f3fb}',
            Self::MediumLight => '\u{1f3fc}',
            Self::Medium => '\u{1f3fd}',
            Self::MediumDark => '\u{1f3fe}',
            Self::Dark => '\u{1f3ff}',
        }
    }
}

impl Emoji {
    fn from_line(line: &str) -> Result<Self> {
        let (code_points, rest) = line.split_once(';').context("expected code points")?;
        let (status, rest) = rest.split_once('#').context("expected status")?;
        let name = rest.trim().splitn(3, ' ').nth(2).context("expected name")?;

        let emoji: String = code_points
            .trim()
            .split(' ')
            .map(parse_code_point)
            .collect::<Result<_>>()?;
        let name = name.trim().to_owned();
        let status = match status.trim() {
            "fully-qualified" => Status::FullyQualified,
            "minimally-qualified" => Status::MinimallyQualified,
            "unqualified" => Status::Unqualified,
            "component" => Status::Component,
            s => bail!("unrecognized status `{}`", s),
        };
        let skin_tone = emoji.chars().find_map(|c| {
            SkinTone::tones().find_map(|tone| (tone.code_point() == c).then(|| tone))
        });

        Ok(Self {
            emoji,
            name,
            status,
            skin_tone,
            variations: Vec::new(),
        })
    }

    pub fn as_string(&self) -> &String {
        &self.emoji
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn skin_tone(&self) -> Option<SkinTone> {
        self.skin_tone
    }

    pub fn variations(&self) -> &[String] {
        &self.variations
    }
}

fn parse_emoji_data(data: &str) -> Result<ParsedData> {
    let mut parsed_data = ParsedData::default();
    let mut lines = data.lines().peekable();
    while let Some(group) = lines.next_group() {
        let group = group.replace("&", "And").to_camel_case();
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
                            Some(SkinTone::Default) | None => {
                                // normal emoji, simply add
                            }
                            Some(_) => {
                                // this emoji has a skin tone we need to find
                                // the default skin tone variation and set it
                                parsed_data[&group][&subgroup]
                                    .iter_mut()
                                    .rev()
                                    .find(|emoji| {
                                        matches!(emoji.skin_tone, Some(SkinTone::Default) | None)
                                    })
                                    .with_context(ctx)?
                                    .skin_tone = Some(SkinTone::Default);
                            }
                        }

                        parsed_data
                            .entry(group.clone())
                            .or_default()
                            .entry(subgroup.clone())
                            .or_insert_with(Vec::new)
                            .push(emoji);
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
