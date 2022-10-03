//! Parse GitHub emoji information.

use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

const URL: &str = "https://github.com/github/gemoji/raw/v4.0.0.rc3/db/emoji.json";

#[derive(Debug, Deserialize)]
pub struct Emoji {
    emoji: String,
    aliases: Vec<String>,
}

pub type ParsedData = HashMap<String, Emoji>;

pub fn fetch_and_parse_emoji_data() -> Result<ParsedData> {
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

    let emojis: Vec<Emoji> = serde_json::from_slice(&buf)?;
    Ok(emojis
        .into_iter()
        .map(|emoji| (emoji.emoji.clone(), emoji))
        .collect())
}

impl Emoji {
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }
}
