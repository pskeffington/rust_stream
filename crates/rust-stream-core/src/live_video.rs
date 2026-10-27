//! Curated *candidate* live news video streams. Never describe these as verified live
//! without runtime HLS + decoded-frame validation and source permission review.
use crate::is_non_global_ip;
use serde::Serialize;
use std::net::IpAddr;
use url::{Host, Url};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateStatus {
    Unverified,
    Approved,
    Retired,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct LiveVideoCandidate {
    pub id: &'static str,
    pub name: &'static str,
    pub publisher: &'static str,
    pub endpoint: &'static str,
    pub evidence: &'static str,
    pub status: CandidateStatus,
}

/// This stream is listed in public directories, not yet independently decoded
/// or licensed for packaging. Disabled by default and never auto-played.
pub const LIVE_NEWS_CANDIDATES: &[LiveVideoCandidate] = &[
    LiveVideoCandidate {
        id: "aljazeera-english-candidate",
        name: "Al Jazeera English",
        publisher: "Al Jazeera Media Network",
        endpoint: "https://live-hls-apps-aje-fa.getaj.net/AJE/index.m3u8",
        evidence: "https://www.aljazeera.com/video/live/",
        status: CandidateStatus::Unverified,
    },
    LiveVideoCandidate {
        id: "dw-english-candidate",
        name: "DW English",
        publisher: "Deutsche Welle",
        endpoint: "https://dwamdstream102.akamaized.net/hls/live/2015525/dwstream102/index.m3u8",
        evidence: "https://database.freetuxtv.net/webStream/index?WebStreamSearchForm%5BType%5D=1",
        status: CandidateStatus::Unverified,
    },
];

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum VideoEndpointError {
    #[error("direct HTTPS HLS playlist required")]
    NotDirectHls,
    #[error("embedded player domains are forbidden")]
    EmbeddedPlatform,
    #[error("credentials and fragments are not permitted")]
    UnsafeUrl,
    #[error("non-global IP addresses and localhost are not public video endpoints")]
    NonGlobalAddress,
}

pub fn validate_direct_hls_url(input: &str) -> Result<(), VideoEndpointError> {
    let url = Url::parse(input).map_err(|_| VideoEndpointError::NotDirectHls)?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err(VideoEndpointError::NotDirectHls);
    }
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err(VideoEndpointError::UnsafeUrl);
    }
    let host = url
        .host_str()
        .unwrap_or_default()
        .trim_end_matches('.')
        .to_ascii_lowercase();
    match url.host() {
        Some(Host::Ipv4(ip)) if is_non_global_ip(IpAddr::V4(ip)) => {
            return Err(VideoEndpointError::NonGlobalAddress);
        }
        Some(Host::Ipv6(ip)) if is_non_global_ip(IpAddr::V6(ip)) => {
            return Err(VideoEndpointError::NonGlobalAddress);
        }
        Some(Host::Domain(domain))
            if domain.eq_ignore_ascii_case("localhost")
                || domain.to_ascii_lowercase().ends_with(".localhost") =>
        {
            return Err(VideoEndpointError::NonGlobalAddress);
        }
        _ => {}
    }
    if [
        "youtube.com",
        "youtu.be",
        "youtube-nocookie.com",
        "googlevideo.com",
        "twitch.tv",
        "vimeo.com",
    ]
    .iter()
    .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
    {
        return Err(VideoEndpointError::EmbeddedPlatform);
    }
    let path = url.path().to_ascii_lowercase();
    if !path.ends_with(".m3u8") {
        return Err(VideoEndpointError::NotDirectHls);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_embedded_pages_and_text_feeds() {
        assert!(validate_direct_hls_url("https://www.youtube.com/watch?v=test").is_err());
        assert!(validate_direct_hls_url("https://publisher.example/news/rss.xml").is_err());
        assert!(validate_direct_hls_url("https://publisher.example/live").is_err());
    }
    #[test]
    fn rejects_private_and_loopback_streams() {
        for endpoint in [
            "https://127.0.0.1/live.m3u8",
            "https://192.168.1.2/live.m3u8",
            "https://169.254.169.254/live.m3u8",
            "https://[::1]/live.m3u8",
            "https://camera.localhost/live.m3u8",
        ] {
            assert_eq!(
                validate_direct_hls_url(endpoint),
                Err(VideoEndpointError::NonGlobalAddress),
                "{endpoint}"
            );
        }
    }
    #[test]
    fn accepts_candidate_shape_without_claiming_stream_health() {
        assert!(validate_direct_hls_url(LIVE_NEWS_CANDIDATES[0].endpoint).is_ok());
        assert_eq!(LIVE_NEWS_CANDIDATES[0].status, CandidateStatus::Unverified);
    }
}
