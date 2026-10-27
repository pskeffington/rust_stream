//! Offline ingestion of IPTV-org channel/stream snapshots.
//! Discovery is never equivalent to playback approval or a health check.
use crate::{Source, SourceKind, SourcePolicy, SourceScope};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use url::Url;

pub const CHANNELS_URL: &str = "https://iptv-org.github.io/api/channels.json";
pub const STREAMS_URL: &str = "https://iptv-org.github.io/api/streams.json";

#[derive(Debug, Deserialize)]
struct Channel {
    id: String,
    name: String,
    country: String,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    is_nsfw: bool,
    closed: Option<String>,
}
#[derive(Debug, Deserialize)]
struct Stream {
    channel: Option<String>,
    url: String,
    #[serde(default)]
    referrer: Option<String>,
    #[serde(default)]
    user_agent: Option<String>,
    #[serde(default)]
    labels: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IptvCandidate {
    pub channel_id: String,
    pub country: String,
    pub categories: Vec<String>,
    pub source: Source,
}
#[derive(Debug, Default)]
pub struct IptvImport {
    /// Eligible, structurally valid candidates; not yet verified playable.
    pub candidates: Vec<IptvCandidate>,
    pub skipped: usize,
}
#[derive(Debug, thiserror::Error)]
pub enum IptvImportError {
    #[error("invalid channels JSON: {0}")]
    Channels(serde_json::Error),
    #[error("invalid streams JSON: {0}")]
    Streams(serde_json::Error),
}

/// Parse catalog snapshots without making requests. Only public HTTPS HLS
/// candidates with a known, open, non-NSFW channel and an approved host survive.
/// No Referer/User-Agent overrides, geoblocked or intermittent streams are imported.
/// Runtime must still apply DNS/redirect/segment checks and decode-frame validation.
pub fn import_iptv_org(
    channels_json: &str,
    streams_json: &str,
    policy: &SourcePolicy,
    countries: &BTreeSet<String>,
) -> Result<IptvImport, IptvImportError> {
    let channels: Vec<Channel> =
        serde_json::from_str(channels_json).map_err(IptvImportError::Channels)?;
    let streams: Vec<Stream> =
        serde_json::from_str(streams_json).map_err(IptvImportError::Streams)?;
    let channel_map: BTreeMap<String, Channel> =
        channels.into_iter().map(|c| (c.id.clone(), c)).collect();
    let mut result = IptvImport::default();
    let mut seen = BTreeSet::new();
    for (index, stream) in streams.into_iter().enumerate() {
        let Some(channel_id) = stream.channel.as_ref() else {
            result.skipped += 1;
            continue;
        };
        let Some(channel) = channel_map.get(channel_id) else {
            result.skipped += 1;
            continue;
        };
        if channel.is_nsfw || channel.closed.is_some()
            || (!countries.is_empty() && !countries.contains(&channel.country))
            || stream.referrer.is_some() || stream.user_agent.is_some()
            || stream.labels.iter().any(|l| l == "Geo-blocked" || l == "Not 24/7")
        {
            result.skipped += 1;
            continue;
        }
        let Ok(url) = Url::parse(&stream.url) else {
            result.skipped += 1;
            continue;
        };
        if url.scheme() != "https" || !url.path().to_ascii_lowercase().ends_with(".m3u8")
            || !seen.insert((channel_id.clone(), stream.url.clone()))
        {
            result.skipped += 1;
            continue;
        }
        let source = Source {
            id: format!("iptv-{index}"),
            label: channel.name.chars().take(100).collect(),
            endpoint: stream.url,
            kind: SourceKind::Hls,
            scope: SourceScope::Public,
            enabled: false, // Explicit approval and decoding checks required.
        };
        if source.validate_definition(policy).is_err() {
            result.skipped += 1;
            continue;
        }
        result.candidates.push(IptvCandidate {
            channel_id: channel_id.clone(),
            country: channel.country.clone(),
            categories: channel.categories.clone(),
            source,
        });
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    const CHANNELS: &str = r#"[
        {"id":"Test.us","name":"Test TV","country":"US","categories":["news"],"is_nsfw":false,"closed":null},
        {"id":"Adult.us","name":"Adult TV","country":"US","is_nsfw":true}
    ]"#;
    fn policy() -> SourcePolicy {
        SourcePolicy {
            allowed_hosts: ["media.example".into()].into(),
            ..Default::default()
        }
    }
    #[test]
    fn accepts_known_https_hls_as_disabled_candidate() {
        let streams = r#"[{"channel":"Test.us","url":"https://media.example/live.m3u8"}]"#;
        let imported = import_iptv_org(CHANNELS, streams, &policy(), &BTreeSet::new()).unwrap();
        assert_eq!(imported.candidates.len(), 1);
        assert!(!imported.candidates[0].source.enabled);
        assert_eq!(imported.candidates[0].categories, ["news"]);
    }
    #[test]
    fn rejects_unknown_private_unapproved_and_external_player_sources() {
        let streams = r#"[
            {"channel":"Test.us","url":"http://media.example/live.m3u8"},
            {"channel":"Test.us","url":"https://127.0.0.1/live.m3u8"},
            {"channel":"Test.us","url":"https://other.example/live.m3u8"},
            {"channel":"Test.us","url":"https://media.example/player"},
            {"channel":"Adult.us","url":"https://media.example/live.m3u8"},
            {"channel":"Missing.us","url":"https://media.example/live.m3u8"},
            {"channel":"Test.us","url":"https://media.example/live.m3u8","referrer":"https://other.example"},
            {"channel":"Test.us","url":"https://media.example/live.m3u8","labels":["Geo-blocked"]}
        ]"#;
        let imported = import_iptv_org(CHANNELS, streams, &policy(), &BTreeSet::new()).unwrap();
        assert!(imported.candidates.is_empty());
        assert_eq!(imported.skipped, 8);
    }
    #[test]
    fn honors_country_filter_and_deduplicates() {
        let streams = r#"[
            {"channel":"Test.us","url":"https://media.example/live.m3u8"},
            {"channel":"Test.us","url":"https://media.example/live.m3u8"}
        ]"#;
        let us = ["US".into()].into();
        assert_eq!(import_iptv_org(CHANNELS, streams, &policy(), &us).unwrap().candidates.len(), 1);
        let ca = ["CA".into()].into();
        assert!(import_iptv_org(CHANNELS, streams, &policy(), &ca).unwrap().candidates.is_empty());
    }
}
