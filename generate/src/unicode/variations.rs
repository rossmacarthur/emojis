use anyhow::Context as _;
use anyhow::Result;

use crate::util;

const URL: &str = "https://unicode.org/Public/16.0.0/ucd/emoji/emoji-variation-sequences.txt";

pub fn parse() -> Result<Vec<String>> {
    let data = util::cached_download(URL)?;
    let mut entries = Vec::new();
    for line in data.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let (code_points, _) = line.split_once(';').context("expected code points")?;
        let code_points = parse_code_points(code_points)?;
        if !code_points.ends_with(&['\u{fe0f}']) {
            continue;
        }

        match code_points.strip_suffix(&['\u{fe0f}']) {
            Some(code_points) => {
                entries.push(String::from_iter(code_points));
            }
            None => todo!(),
        }
    }
    Ok(entries)
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
