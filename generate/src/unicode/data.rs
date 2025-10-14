//! Parses the Unicode emoji data into a more usable format.

use anyhow::bail;
use anyhow::ensure;
use anyhow::Context as _;
use anyhow::Result;
use constcat::concat;
use serde::Serialize;

use crate::unicode::{VERSION_MAJOR, VERSION_MINOR, VERSION_PATCH};
use crate::util;

const URL: &str = concat!(
    "https://unicode.org/Public/",
    VERSION_MAJOR,
    ".",
    VERSION_MINOR,
    ".",
    VERSION_PATCH,
    "/emoji/emoji-test.txt"
);

/// A single entry in the file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Entry {
    pub group: Group,
    pub subgroup: String,
    pub status: Status,
    pub unicode_version: UnicodeVersion,
    pub emoji: String,
    pub name: String,
}

/// The Unicode emoji group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub enum Group {
    SmileysAndEmotion,
    PeopleAndBody,
    AnimalsAndNature,
    FoodAndDrink,
    TravelAndPlaces,
    Activities,
    Objects,
    Symbols,
    Flags,
    Component,
}

/// The Unicode emoji data status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    /// A qualified emoji character, or an emoji sequence in which each emoji character is qualified.
    ///
    /// https://www.unicode.org/reports/tr51/#def_fully_qualified_emoji
    FullyQualified,

    /// An emoji sequence in which the first character is qualified but the sequence is not fully qualified.
    ///
    /// https://www.unicode.org/reports/tr51/#def_minimally_qualified_emoji
    MinimallyQualified,

    /// An emoji that is neither fully-qualified nor minimally qualified.
    ///
    /// https://www.unicode.org/reports/tr51/#def_unqualified_emoji
    Unqualified,

    /// Not an emoji, but defines building block code point(s) for emojis.
    Component,
}

/// The Unicode version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UnicodeVersion {
    major: u32,
    minor: u32,
}

pub fn parse() -> Result<Vec<Entry>> {
    let data = util::cached_download(URL)?;
    let entries = parse_emoji_data(&data)?;
    Ok(entries)
}

fn parse_emoji_data(data: &str) -> Result<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut group = String::new();
    let mut subgroup = String::new();
    for line in data.lines() {
        if line.is_empty() {
            continue;
        } else if let Some(g) = line.strip_prefix("# group: ") {
            group = g.trim().to_owned();
        } else if let Some(s) = line.strip_prefix("# subgroup: ") {
            subgroup = s.trim().to_owned();
        } else if line.starts_with('#') {
            continue;
        } else {
            ensure!(!group.is_empty(), "missing group");
            ensure!(!subgroup.is_empty(), "missing subgroup");
            let entry = parse_entry(parse_group(&group)?, subgroup.clone(), line)?;
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn parse_group(s: &str) -> Result<Group> {
    Ok(match s {
        "Smileys & Emotion" => Group::SmileysAndEmotion,
        "People & Body" => Group::PeopleAndBody,
        "Animals & Nature" => Group::AnimalsAndNature,
        "Food & Drink" => Group::FoodAndDrink,
        "Travel & Places" => Group::TravelAndPlaces,
        "Activities" => Group::Activities,
        "Objects" => Group::Objects,
        "Symbols" => Group::Symbols,
        "Flags" => Group::Flags,
        "Component" => Group::Component,
        _ => bail!("invalid group: {}", s),
    })
}

fn parse_entry(group: Group, subgroup: String, line: &str) -> Result<Entry> {
    let (code_points, rest) = line.split_once(';').context("expected code points")?;
    let (status, rest) = rest.split_once('#').context("expected status")?;
    let (emoji, unicode_version, name) = {
        let mut rest = rest.trim().splitn(3, |c: char| c.is_ascii_whitespace());
        let emoji = rest.next().context("expected emoji")?;
        let unicode_version = rest.next().context("expected unicode version")?;
        let name = rest.next().context("expected name")?;
        ensure!(rest.next().is_none());
        (emoji, unicode_version, name)
    };

    // Verify that the emoji matches the code points defined.
    if emoji != String::from_iter(parse_code_points(code_points)?) {
        bail!("emoji mismatch");
    }

    let status = parse_status(status.trim())?;
    let emoji = emoji.to_owned();
    let unicode_version =
        parse_unicode_version(unicode_version.trim()).context("invalid unicode version")?;
    let name = name.to_owned();

    Ok(Entry {
        group,
        subgroup,
        status,
        unicode_version,
        emoji,
        name,
    })
}

fn parse_code_points(s: &str) -> Result<Vec<char>> {
    s.split_ascii_whitespace()
        .map(parse_code_point)
        .collect::<Result<_>>()
        .context("invalid code points")
}

fn parse_code_point(s: &str) -> Result<char> {
    let scalar = u32::from_str_radix(s, 16).context("not hex")?;
    char::from_u32(scalar).context("not Unicode scalar value")
}

fn parse_status(s: &str) -> Result<Status> {
    Ok(match s {
        "fully-qualified" => Status::FullyQualified,
        "minimally-qualified" => Status::MinimallyQualified,
        "unqualified" => Status::Unqualified,
        "component" => Status::Component,
        _ => bail!("invalid status: {:?}", s),
    })
}

fn parse_unicode_version(s: &str) -> Result<UnicodeVersion> {
    let (major, minor) = s
        .strip_prefix('E')
        .context("missing 'E'")?
        .split_once('.')
        .context("missing decimal")?;
    let major = major.parse().context("invalid major version")?;
    let minor = minor.parse().context("invalid minor version")?;
    Ok(UnicodeVersion { major, minor })
}
