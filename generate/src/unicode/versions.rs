//! Parses the "Emoji Versions" table from UTS #51 into a mapping from emoji
//! version (UTS #51) to the Unicode Standard version it was released in.
//!
//! See <https://www.unicode.org/reports/tr51/#EmojiVersions>.

use anyhow::bail;
use anyhow::Context as _;
use anyhow::Result;

use crate::util;

const URL: &str = "https://www.unicode.org/reports/tr51/";

/// A `(major, minor)` version pair.
pub type Version = (u32, u32);

/// Returns the subset of the Emoji Versions table where the emoji version
/// differs from the Unicode Standard version it was released in.
///
/// Versions are aligned from Unicode 11.0 onwards, so the result only contains
/// the historical pre-alignment rows plus any later dot releases (e.g. emoji
/// 13.1 shipped on Unicode 13.0).
pub fn parse() -> Result<Vec<(Version, Version)>> {
    let html = util::cached_download(URL)?;
    let rows = parse_table(&html)?;
    Ok(rows
        .into_iter()
        .filter(|(emoji, unicode)| emoji != unicode)
        .collect())
}

fn parse_table(html: &str) -> Result<Vec<(Version, Version)>> {
    // The table is anchored by `name="EmojiVersions"`; take everything from
    // there to the end of the enclosing table.
    let start = html
        .find("name=\"EmojiVersions\"")
        .context("missing EmojiVersions anchor")?;
    let table = &html[start..];
    let table_start = table.find("<table").context("missing table")?;
    let table_end = table.find("</table>").context("missing table end")?;
    let table = &table[table_start..table_end];

    let mut mapping = Vec::new();
    for row in table.split("<tr>").skip(1) {
        let cells: Vec<String> = row
            .split("<td>")
            .skip(1)
            .filter_map(|cell| cell.split("</td>").next())
            .map(|cell| cell.trim().to_owned())
            .collect();
        // Columns: Emoji Version, Date, Unicode Version, Data File Comment.
        let [_, _, unicode, comment] = cells.as_slice() else {
            continue; // header row or otherwise malformed
        };
        let (Some(emoji), Some(unicode)) = (
            parse_data_file_comment(comment),
            parse_unicode_version(unicode),
        ) else {
            continue; // rows like "N/A" / "various" / "E0.0"
        };
        mapping.push((emoji, unicode));
    }

    if mapping.is_empty() {
        bail!("parsed no rows from the Emoji Versions table");
    }
    Ok(mapping)
}

/// Parses a data file comment such as `E13.1` (the form used in
/// `emoji-test.txt`) into a version pair.
fn parse_data_file_comment(s: &str) -> Option<Version> {
    parse_version(s.strip_prefix('E')?)
}

/// Parses a cell such as `Unicode 13.0` into a version pair.
fn parse_unicode_version(s: &str) -> Option<Version> {
    parse_version(s.strip_prefix("Unicode ")?)
}

fn parse_version(s: &str) -> Option<Version> {
    let (major, minor) = s.split_once('.')?;
    Some((major.parse().ok()?, minor.parse().ok()?))
}
