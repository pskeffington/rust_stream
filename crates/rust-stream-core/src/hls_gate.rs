use crate::{HlsError, HlsManifest, parse_hls};
use std::collections::BTreeSet;
use url::Url;
#[derive(Debug, Clone)]
pub struct HlsHostPolicy {
    pub approved_hosts: BTreeSet<String>,
    pub allow_cross_origin: bool,
}
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HlsPolicyError {
    #[error("invalid HLS manifest")]
    InvalidManifest(#[from] HlsError),
    #[error("unapproved host")]
    UnapprovedHost,
    #[error("unapproved cross-origin reference")]
    CrossOrigin,
    #[error("no playable references")]
    Empty,
}
pub fn preflight_hls(
    base: &str,
    body: &[u8],
    policy: &HlsHostPolicy,
) -> Result<HlsManifest, HlsPolicyError> {
    let base = Url::parse(base).map_err(|_| HlsPolicyError::UnapprovedHost)?;
    let origin = base
        .host_str()
        .ok_or(HlsPolicyError::UnapprovedHost)?
        .to_ascii_lowercase();
    if !policy.approved_hosts.contains(&origin) {
        return Err(HlsPolicyError::UnapprovedHost);
    }
    let manifest = parse_hls(base.as_str(), body)?;
    if manifest.references.is_empty() {
        return Err(HlsPolicyError::Empty);
    }
    for candidate in &manifest.references {
        let url = Url::parse(candidate).map_err(|_| HlsPolicyError::UnapprovedHost)?;
        let host = url
            .host_str()
            .ok_or(HlsPolicyError::UnapprovedHost)?
            .to_ascii_lowercase();
        if !policy.approved_hosts.contains(&host) {
            return Err(HlsPolicyError::UnapprovedHost);
        }
        if !policy.allow_cross_origin && host != origin {
            return Err(HlsPolicyError::CrossOrigin);
        }
    }
    Ok(manifest)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn policy() -> HlsHostPolicy {
        HlsHostPolicy {
            approved_hosts: ["media.example".into()].into(),
            allow_cross_origin: false,
        }
    }
    #[test]
    fn valid_same_origin() {
        assert!(
            preflight_hls(
                "https://media.example/live.m3u8",
                b"#EXTM3U\n#EXTINF:2,\nseg.ts\n",
                &policy()
            )
            .is_ok()
        );
    }
    #[test]
    fn reject_unlisted() {
        assert!(matches!(
            preflight_hls(
                "https://media.example/live.m3u8",
                b"#EXTM3U\n#EXTINF:2,\nhttps://other.example/seg.ts\n",
                &policy()
            ),
            Err(HlsPolicyError::UnapprovedHost)
        ));
    }
    #[test]
    fn reject_empty() {
        assert!(matches!(
            preflight_hls("https://media.example/live.m3u8", b"#EXTM3U\n", &policy()),
            Err(HlsPolicyError::Empty)
        ));
    }
}
