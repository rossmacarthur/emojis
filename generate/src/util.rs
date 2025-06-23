use std::env;
use std::fs;
use std::io;
use std::path::Path;

use anyhow::Result;
use sha2::{Digest, Sha256};

pub fn cached_download(url: &str) -> Result<String> {
    // Check if we have a cached version of the file.
    let checksum = hex::encode(Sha256::digest(url.as_bytes()));
    let path = Path::new(concat!(env!("CARGO_WORKSPACE_DIR"), "/target/generate"))
        .join(checksum)
        .with_extension("txt");

    let cwd = env::current_dir()?;

    match fs::read_to_string(&path) {
        Ok(data) => {
            eprintln!(
                "using cached: {url}\n    at {}",
                path.strip_prefix(&cwd)?.display()
            );
            return Ok(data);
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err.into()),
    }

    let data = download(url)?;
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, &data)?;
    eprintln!(
        "downloaded: {url}\n    to {}",
        path.strip_prefix(&cwd)?.display()
    );

    Ok(data)
}

pub fn download(url: &str) -> Result<String> {
    let mut buf = Vec::new();
    let mut easy = curl::easy::Easy::new();
    easy.fail_on_error(true)?;
    easy.follow_location(true)?;
    easy.url(url)?;
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
