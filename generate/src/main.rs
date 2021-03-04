use std::iter;
use std::str;

use anyhow::{bail, Context, Result};
use indexmap::IndexMap;

const URL: &str = "https://unicode.org/Public/emoji/13.1/emoji-test.txt";

#[derive(Debug)]
enum Status {
    FullyQualified,
    MinimallyQualified,
    Unqualified,
    Component,
}

#[derive(Debug)]
struct Emoji {
    code_points: Vec<String>,
    status: Status,
    emoji: String,
    name: String,
}

type Lines<'a> = iter::Peekable<str::Lines<'a>>;

type ParsedData = IndexMap<String, IndexMap<String, Vec<Emoji>>>;

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

impl Emoji {
    fn from_line(line: &str) -> Result<Self> {
        let mut it = line.splitn(3, &[';', '#'][..]);
        let code_points = it
            .next()
            .context("expected code points")?
            .trim()
            .split(' ')
            .map(ToOwned::to_owned)
            .collect();
        let status = match it.next().context("expected status")?.trim() {
            "fully-qualified" => Status::FullyQualified,
            "minimally-qualified" => Status::MinimallyQualified,
            "unqualified" => Status::Unqualified,
            "component" => Status::Component,
            s => bail!("unrecognized status `{}`", s),
        };
        let rest = it.next().context("expected emoji name")?.trim();
        let mut it = rest.splitn(3, ' ');
        let emoji = it.next().context("expected emoji")?.trim().to_owned();
        let _version = it.next().context("expected version")?;
        let name = it.next().context("expected name")?.trim().to_owned();
        Ok(Self {
            code_points,
            status,
            emoji,
            name,
        })
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
                parsed_data
                    .entry(group.clone())
                    .or_default()
                    .entry(subgroup.clone())
                    .or_insert_with(Vec::new)
                    .push(emoji);
            }
        }
    }
    Ok(parsed_data)
}

fn main() -> Result<()> {
    let data = fetch_emoji_data()?;
    let parsed_data = parse_emoji_data(&data)?;
    println!("{:#?}", parsed_data);
    Ok(())
}
