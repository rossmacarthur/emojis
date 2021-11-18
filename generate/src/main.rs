mod github;
mod unicode;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::unicode::SkinTone;

fn generate_group_enum(unicode_data: &unicode::ParsedData) -> String {
    let mut group = String::new();
    group.push_str("/// A category for an emoji.\n");
    group.push_str("///\n");
    group.push_str("/// Based on Unicode CLDR data.\n");
    group.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\n");
    group.push_str("pub enum Group {\n");
    for name in unicode_data.keys() {
        if name == "Component" {
            continue;
        }
        group.push_str(&format!("   {},\n", name));
    }
    group.push_str("}\n\n");
    group
}

fn generate_emoji_struct(
    github_data: &github::ParsedData,
    id: usize,
    group: &str,
    emoji: &unicode::Emoji,
    default_skin_tone_id: usize,
) -> String {
    let variations = emoji.variations().to_vec();
    let mut s = format!(
        "Emoji {{ id: {}, emoji: \"{}\", name: \"{}\", group: Group::{}",
        id,
        emoji.as_string(),
        emoji.name(),
        group,
    );
    match emoji.skin_tone() {
        Some(tone) => s.push_str(&format!(
            ", skin_tone: Some(({}, SkinTone::{:?}))",
            default_skin_tone_id, tone
        )),
        None => s.push_str(", skin_tone: None"),
    }
    match &github_data.get(emoji.as_string()) {
        Some(github) => s.push_str(&format!(", aliases: Some(&{:?})", github.aliases())),
        None => s.push_str(", aliases: None"),
    }
    s.push_str(&format!(", variations: &{:?} }}", variations));
    s
}

fn generate_emojis_array(
    unicode_data: &unicode::ParsedData,
    github_data: &github::ParsedData,
) -> String {
    let mut id = 0;
    let mut default_skin_tone_id = 0;
    let mut emojis = String::from("pub const EMOJIS: &[Emoji] = &[\n");
    for (group, subgroups) in unicode_data {
        for subgroup in subgroups.values() {
            for emoji in subgroup {
                if matches!(emoji.skin_tone(), Some(SkinTone::Default)) {
                    default_skin_tone_id = id;
                }
                emojis.push_str("    ");
                emojis.push_str(&generate_emoji_struct(
                    github_data,
                    id,
                    group,
                    emoji,
                    default_skin_tone_id,
                ));
                emojis.push_str(",\n");
                id += 1;
            }
        }
    }
    emojis.push_str("];\n");
    emojis
}

fn generate(unicode_data: unicode::ParsedData, github_data: github::ParsedData) -> String {
    let mut module = String::new();
    module.push_str("#![cfg_attr(rustfmt, rustfmt::skip)]\n\n");
    module.push_str("use crate::{Emoji, SkinTone};\n\n");
    module.push_str(&generate_group_enum(&unicode_data));
    module.push_str(&generate_emojis_array(&unicode_data, &github_data));
    module
}

fn main() -> Result<()> {
    let unicode_data = unicode::fetch_and_parse_emoji_data()?;
    let github_data = github::fetch_and_parse_emoji_data()?;
    let module = generate(unicode_data, github_data);
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "src", "generated.rs"]
        .iter()
        .collect();
    fs::write(&path, module)?;
    Ok(())
}
