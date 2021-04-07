mod github;
mod unicode;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;

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
) -> String {
    let variations: Vec<_> = emoji
        .variations()
        .iter()
        .map(|e| format!("\"{}\"", e))
        .collect();

    let skin_tones: Vec<_> = emoji
        .skin_tones()
        .iter()
        .map(|e| format!("\"{}\"", e))
        .collect();

    match &github_data.get(&emoji.emoji()) {
        Some(github) => {
            format!(
                "Emoji {{ id: {}, emoji: \"{}\", name: \"{}\", group: Group::{}, aliases: Some(&{:?}), variations: &[{}], skin_tones: &[{}] }},\n",
                id,
                emoji.emoji(),
                emoji.name(),
                group,
                github.aliases(),
                variations.join(", "),
                skin_tones.join(", "),
            )
        }
        None => {
            format!(
                "Emoji {{ id: {}, emoji: \"{}\", name: \"{}\", group: Group::{}, aliases: None, variations: &[{}], skin_tones: &[{}] }},\n",
                id,
                emoji.emoji(),
                emoji.name(),
                group,
                variations.join(", "),
                skin_tones.join(", "),
            )
        }
    }
}

fn generate_emojis_array(
    unicode_data: &unicode::ParsedData,
    github_data: &github::ParsedData,
) -> String {
    let mut id = 0;
    let mut emojis = String::new();
    emojis.push_str("pub const EMOJIS: &[Emoji] = &[\n");
    for (group, subgroups) in unicode_data {
        for subgroup in subgroups.values() {
            for emoji in subgroup {
                emojis.push_str("    ");
                emojis.push_str(&generate_emoji_struct(github_data, id, group, emoji));
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
    module.push_str("use crate::Emoji;\n\n");
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
