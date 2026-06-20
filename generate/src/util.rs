use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use anyhow::Context as _;
use anyhow::Result;
use sha2::{Digest, Sha256};

const CACHE_MAX_AGE: Duration = Duration::from_secs(24 * 60 * 60);

pub fn cached_download(url: &str) -> Result<String> {
    let ext = url
        .rsplit_once('/')
        .map(|(_, part)| part)
        .and_then(|part| part.rsplit_once('.').map(|(_, ext)| ext))
        .unwrap_or("html");

    // Check if we have a cached version of the file.
    let checksum = hex::encode(Sha256::digest(url.as_bytes()));
    let mut path = PathBuf::from_iter([env!("CARGO_WORKSPACE_DIR"), "target/generate", &checksum]);
    path.set_extension(ext);

    let cwd = env::current_dir()?;

    if cache_is_fresh(&path)? {
        let data = fs::read_to_string(&path)?;
        eprintln!(
            "using cached: {url}\n    at {}",
            path.strip_prefix(&cwd)?.display()
        );
        return Ok(data);
    }

    let data = download(url).with_context(|| format!("failed to download {url}"))?;
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, &data)?;
    eprintln!(
        "downloaded: {url}\n    to {}",
        path.strip_prefix(&cwd)?.display()
    );

    Ok(data)
}

fn cache_is_fresh(path: &Path) -> Result<bool> {
    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(err) => return Err(err.into()),
    };
    Ok(SystemTime::now()
        .duration_since(metadata.modified()?)
        .map(|age| age <= CACHE_MAX_AGE)
        .unwrap_or(false))
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
