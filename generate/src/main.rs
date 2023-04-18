mod github;
mod unicode;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write as _;
use std::path::PathBuf;

use anyhow::Result;

use crate::unicode::SkinTone;

fn write_group_enum<W: io::Write>(w: &mut W, unicode_data: &unicode::ParsedData) -> Result<()> {
    writeln!(w, "/// A category for an emoji.")?;
    writeln!(w, "///")?;
    writeln!(w, "/// Based on Unicode CLDR data.")?;
    writeln!(
        w,
        "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]"
    )?;
    writeln!(w, "pub enum Group {{")?;
    for name in unicode_data.keys() {
        if name == "Component" {
            continue;
        }
        writeln!(w, "   {name},")?;
    }
    writeln!(w, "}}")?;
    Ok(())
}

fn write_emoji_struct<W: io::Write>(
    w: &mut W,
    github_data: &github::ParsedData,
    group: &str,
    emoji: &unicode::Emoji,
    default_skin_tone_index: usize,
    skin_tone_count: usize,
) -> Result<()> {
    let e = emoji.as_str();
    let name = emoji.name();
    let uv = emoji.unicode_version();
    write!(
        w,
        "Emoji {{ emoji: \"{e}\", name: \"{name}\", unicode_version: {uv:?}, group: Group::{group}",
    )?;
    match emoji.skin_tone() {
        Some(tone) => write!(
            w,
            ", skin_tone: Some(({default_skin_tone_index}, {skin_tone_count}, SkinTone::{tone:?}))",
        )?,
        None => write!(w, ", skin_tone: None")?,
    }
    match &github_data.get(e) {
        Some(github) => write!(w, ", aliases: Some(&{:?}) }}", github.aliases())?,
        None => write!(w, ", aliases: None }}")?,
    }
    Ok(())
}

fn write_emojis_slice<W: io::Write>(
    w: &mut W,
    unicode_data: &unicode::ParsedData,
    github_data: &github::ParsedData,
    unicode_map: &mut HashMap<String, String>,
    shortcode_map: &mut HashMap<String, String>,
) -> Result<()> {
    let mut i = 0;
    let mut default_skin_tone_index = 0;
    let mut skin_tone_count = 0;

    writeln!(w, "pub const EMOJIS: &[Emoji] = &[")?;
    for (group, subgroups) in unicode_data {
        for subgroup in subgroups.values() {
            for emoji in subgroup {
                if matches!(emoji.skin_tone(), Some(SkinTone::Default)) {
                    default_skin_tone_index = i;
                    skin_tone_count = emoji.skin_tones();
                }
                write!(w, "    ")?;
                write_emoji_struct(
                    w,
                    github_data,
                    group,
                    emoji,
                    default_skin_tone_index,
                    skin_tone_count,
                )?;
                writeln!(w, ",")?;

                unicode_map.insert(emoji.as_str().to_owned(), i.to_string());
                for v in emoji.variations() {
                    assert!(unicode_map.insert(v.to_owned(), i.to_string()).is_none());
                }

                if let Some(github) = &github_data.get(emoji.as_str()) {
                    for alias in github.aliases() {
                        assert!(shortcode_map
                            .insert(alias.to_owned(), i.to_string())
                            .is_none());
                    }
                }
                i += 1;
            }
        }
    }
    writeln!(w, "];")?;
    Ok(())
}

fn write_phf_map<W: io::Write>(w: &mut W, map: HashMap<String, String>) -> Result<()> {
    write!(w, "pub static MAP: phf::Map<&'static str, usize> = ")?;
    let mut gen = phf_codegen::Map::new();
    for (key, value) in &map {
        gen.entry(key, value);
    }
    writeln!(w, "{};", gen.build())?;
    Ok(())
}

fn main() -> Result<()> {
    let dir: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "src", "gen"]
        .iter()
        .collect();

    let unicode_data = unicode::fetch_and_parse_emoji_data()?;
    let github_data = github::fetch_and_parse_emoji_data()?;
    let mut unicode_map = HashMap::new();
    let mut shortcode_map = HashMap::new();

    fs::remove_dir_all(&dir).ok();
    fs::create_dir_all(&dir)?;

    let mut f = fs::File::create(dir.join("mod.rs"))?;
    writeln!(f, "#![cfg_attr(rustfmt, rustfmt::skip)]\n")?;
    writeln!(f, "pub mod shortcode;")?;
    writeln!(f, "pub mod unicode;\n")?;
    writeln!(f, "use crate::{{Emoji, SkinTone, UnicodeVersion}};\n")?;

    write_group_enum(&mut f, &unicode_data)?;
    writeln!(f)?;
    write_emojis_slice(
        &mut f,
        &unicode_data,
        &github_data,
        &mut unicode_map,
        &mut shortcode_map,
    )?;

    let mut f = fs::File::create(dir.join("unicode.rs"))?;
    write_phf_map(&mut f, unicode_map)?;

    let mut f = fs::File::create(dir.join("shortcode.rs"))?;
    write_phf_map(&mut f, shortcode_map)?;

    Ok(())
}
