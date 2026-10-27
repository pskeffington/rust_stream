//! Bounded, fail-closed IPTV-org snapshot fetch and atomic local cache.
//! This updater does not approve or play any discovered media.
use reqwest::blocking::Client;
use reqwest::redirect::Policy;
use rust_stream_core::{CHANNELS_URL, STREAMS_URL, SourcePolicy, import_iptv_org};
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

const MAX_CHANNELS: usize = 12 * 1024 * 1024;
const MAX_STREAMS: usize = 24 * 1024 * 1024;

fn fetch_bounded(
    client: &Client,
    url: &str,
    limit: usize,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut response = client.get(url).send()?.error_for_status()?;
    if response
        .content_length()
        .is_some_and(|size| size > limit as u64)
    {
        return Err("catalog exceeds configured size limit".into());
    }
    let mut output = Vec::new();
    let mut bounded = (&mut response).take(limit as u64 + 1);
    bounded.read_to_end(&mut output)?;
    if output.len() > limit {
        return Err("catalog exceeds configured size limit".into());
    }
    Ok(output)
}

fn atomic_write(path: &Path, payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
    let parent = path.parent().ok_or("cache path has no parent")?;
    fs::create_dir_all(parent)?;
    let temporary = path.with_extension("tmp");
    let result = (|| {
        let mut f = fs::File::create(&temporary)?;
        f.write_all(payload)?;
        f.sync_all()?;
        fs::rename(&temporary, path)?;
        Ok::<(), std::io::Error>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 || args[1] != "--cache-dir" {
        return Err(
            "usage: cargo run -p rust-stream-catalog -- --cache-dir ./data/iptv-org".into(),
        );
    }
    let path = Path::new(&args[2]);
    let client = Client::builder()
        .timeout(Duration::from_secs(20))
        .connect_timeout(Duration::from_secs(8))
        .redirect(Policy::none())
        .build()?;
    // Explicit canonical endpoints only; never fetch stream URLs in this updater.
    let channels = fetch_bounded(&client, CHANNELS_URL, MAX_CHANNELS)?;
    let streams = fetch_bounded(&client, STREAMS_URL, MAX_STREAMS)?;
    let channels_text = std::str::from_utf8(&channels)?;
    let streams_text = std::str::from_utf8(&streams)?;
    // JSON is parsed fully before either cache file can be touched.
    let _: serde_json::Value = serde_json::from_str(channels_text)?;
    let _: serde_json::Value = serde_json::from_str(streams_text)?;
    let discovered = import_iptv_org(
        channels_text,
        streams_text,
        &SourcePolicy {
            allowed_hosts: BTreeSet::new(),
            ..Default::default()
        },
        &["US".to_string()].into(),
    )?;
    // Zero candidates is intentional: runtime operators must first approve host policies.
    // Retain the raw discovery catalogs for subsequent local review and policy application.
    atomic_write(&path.join("channels.json"), &channels)?;
    atomic_write(&path.join("streams.json"), &streams)?;
    println!(
        "Cached IPTV-org discovery JSON. US auto-approved sources: {}; skipped: {}. Playback not enabled.",
        discovered.candidates.len(),
        discovered.skipped
    );
    Ok(())
}

// Compile-time invariants; no runtime assertion of constants.
const _: () = {
    assert!(MAX_CHANNELS < MAX_STREAMS);
    assert!(MAX_STREAMS <= 24 * 1024 * 1024);
};
