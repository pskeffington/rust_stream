//! Curated *candidate* live news video streams. Never describe these as verified live
//! without runtime HLS + decoded-frame validation and source permission review.
use serde::Serialize;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateStatus { Unverified, Approved, Retired }

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
}

pub fn validate_direct_hls_url(input: &str) -> Result<(), VideoEndpointError> {
    let url = Url::parse(input).map_err(|_| VideoEndpointError::NotDirectHls)?;
    if url.scheme() != "https" || url.host_str().is_none() {
        return Err(VideoEndpointError::NotDirectHls);
    }
    if !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err(VideoEndpointError::UnsafeUrl);
    }
    let host = url.host_str().unwrap_or_default().trim_end_matches('.').to_ascii_lowercase();
    if ["youtube.com", "youtu.be", "youtube-nocookie.com", "googlevideo.com", "twitch.tv", "vimeo.com"]
        .iter().any(|domain| host == *domain || host.ends_with(&format!(".{domain}"))) {
        return Err(VideoEndpointError::EmbeddedPlatform);
    }
    let path = url.path().to_ascii_lowercase();
    if !path.ends_with(".m3u8") { return Err(VideoEndpointError::NotDirectHls); }
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
    fn accepts_candidate_shape_without_claiming_stream_health() {
        assert!(validate_direct_hls_url(LIVE_NEWS_CANDIDATES[0].endpoint).is_ok());
        assert_eq!(LIVE_NEWS_CANDIDATES[0].status, CandidateStatus::Unverified);
    }
}
