//! Fetch and parse raw emoji data from Unicode.org.

use std::iter;
use std::str;

use anyhow::{bail, Context, Result};
use heck::CamelCase;
use indexmap::IndexMap;
use itertools::Itertools;

const URL: &str = "https://unicode.org/Public/emoji/13.1/emoji-test.txt";

const SKIN_TONES: &[char] = &[
    '\u{1f3fb}', // light skin tone
    '\u{1f3fc}', // medium-light skin tone
    '\u{1f3fd}', // medium skin tone
    '\u{1f3fe}', // medium-dark skin tone
    '\u{1f3ff}', // dark skin tone
];

#[derive(Debug)]
pub enum Status {
    FullyQualified,
    MinimallyQualified,
    Unqualified,
    Component,
}

#[derive(Debug)]
pub struct Emoji {
    chars: Vec<char>,
    status: Status,
    name: String,
}

pub type ParsedData = IndexMap<String, IndexMap<String, Vec<Emoji>>>;

type Lines<'a> = iter::Peekable<str::Lines<'a>>;

fn fetch_emoji_data() -> Result<String> {
    let mut buf = Vec::new();
    let mut easy = curl::easy::Easy::new();
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
    Ok(std::char::from_u32(scalar).context("not Unicode scalar value")?)
}

impl Emoji {
    fn from_line(line: &str) -> Result<Self> {
        let (code_points, rest) = line
            .splitn(2, ';')
            .next_tuple()
            .context("expected code points")?;
        let (status, rest) = rest
            .splitn(2, '#')
            .next_tuple()
            .context("expected status")?;
        let name = rest.trim().splitn(3, ' ').nth(2).context("expected name")?;

        let chars = code_points
            .trim()
            .split(' ')
            .map(parse_code_point)
            .collect::<Result<_>>()?;
        let status = match status.trim() {
            "fully-qualified" => Status::FullyQualified,
            "minimally-qualified" => Status::MinimallyQualified,
            "unqualified" => Status::Unqualified,
            "component" => Status::Component,
            s => bail!("unrecognized status `{}`", s),
        };
        let name = name.trim().to_owned();

        Ok(Self {
            chars,
            status,
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn emoji(&self) -> String {
        self.chars.iter().collect()
    }
}

fn parse_emoji_data(data: &str) -> Result<ParsedData> {
    let mut parsed_data = ParsedData::default();
    let mut lines = data.lines().peekable();
    while let Some(group) = lines.next_group() {
        while let Some(subgroup) = lines.next_subgroup() {
            for line in &mut lines {
                if line.is_empty() {
                    break;
                }
                let emoji = Emoji::from_line(line)?;
                if !matches!(emoji.status, Status::FullyQualified)
                    || SKIN_TONES.iter().any(|c| emoji.chars.contains(c))
                {
                    continue;
                }
                let group = group.replace("&", "And").to_camel_case();
                parsed_data
                    .entry(group)
                    .or_default()
                    .entry(subgroup.clone())
                    .or_insert_with(Vec::new)
                    .push(emoji);
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
