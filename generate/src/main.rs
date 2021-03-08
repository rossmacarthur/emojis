mod unicode;

use std::fs;
use std::path::PathBuf;

use anyhow::Result;

use crate::unicode::{fetch_and_parse_emoji_data, ParsedData};

fn generate_group_enum(parsed_data: &ParsedData) -> String {
    let mut group = String::new();
    group.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]\n");
    group.push_str("pub enum Group {\n");
    for name in parsed_data.keys() {
        if name == "Component" {
            continue;
        }
        group.push_str(&format!("   {},\n", name));
    }
    group.push_str("}\n\n");
    group
}

fn generate_emojis_array(parsed_data: &ParsedData) -> String {
    let mut id = 0;
    let mut emojis = String::new();
    emojis.push_str("pub const EMOJIS: &[Emoji] = &[\n");
    for (group, subgroups) in parsed_data {
        for subgroup in subgroups.values() {
            for emoji in subgroup {
                emojis.push_str(&format!(
                    "    Emoji {{ id: {}, emoji: \"{}\", name: \"{}\", group: Group::{} }},\n",
                    id,
                    emoji.emoji(),
                    emoji.name(),
                    group,
                ));
                id += 1;
            }
        }
    }
    emojis.push_str("];\n");
    emojis
}

fn generate(parsed_data: ParsedData) -> String {
    let mut module = String::new();
    module.push_str("#![cfg_attr(rustfmt, rustfmt::skip)]\n\n");
    module.push_str("use crate::Emoji;\n\n");
    module.push_str(&generate_group_enum(&parsed_data));
    module.push_str(&generate_emojis_array(&parsed_data));
    module
}

fn main() -> Result<()> {
    let parsed_data = fetch_and_parse_emoji_data()?;
    let module = generate(parsed_data);
    let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "..", "src", "generated.rs"]
        .iter()
        .collect();
    fs::write(&path, module)?;
    Ok(())
}
