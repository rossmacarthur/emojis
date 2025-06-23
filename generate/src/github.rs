//! Parse GitHub emoji information.

use std::collections::HashMap;

use anyhow::Result;
use serde::Deserialize;

use crate::util;

const URL: &str = "https://github.com/github/gemoji/raw/v4.1.0/db/emoji.json";

pub type ParsedData = HashMap<String, Vec<String>>;

#[derive(Debug, Deserialize)]
pub struct Emoji {
    pub emoji: String,
    pub aliases: Vec<String>,
}

pub fn build() -> Result<ParsedData> {
    let buf = util::cached_download(URL)?;
    let emojis: Vec<Emoji> = serde_json::from_str(&buf)?;
    Ok(emojis.into_iter().map(|e| (e.emoji, e.aliases)).collect())
}
