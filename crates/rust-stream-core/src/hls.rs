//! HLS manifest preflight. Parsing only: this module performs no network requests.
//! Production transport must enforce policy on every playlist/key/segment request.
use thiserror::Error;
use url::Url;

pub const MAX_MANIFEST_BYTES: usize = 1024 * 1024;
pub const MAX_REFERENCES: usize = 2048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HlsManifest {
    pub is_master: bool,
    pub references: Vec<String>,
    pub has_media_segments: bool,
    pub end_list: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum HlsError {
    #[error("manifest too large")]
    TooLarge,
    #[error("not valid UTF-8 or not an HLS playlist")]
    Invalid,
    #[error("invalid or disallowed media reference")]
    UnsafeReference,
    #[error("too many media references")]
    TooManyReferences,
}

fn resolve(base: &Url, reference: &str) -> Result<String, HlsError> {
    let url = base
        .join(reference.trim())
        .map_err(|_| HlsError::UnsafeReference)?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(HlsError::UnsafeReference);
    }
    Ok(url.to_string())
}

/// Checks a playlist's shape and extracts follow-up URLs. URL parsing is NOT
/// a network authorization boundary: verify DNS, redirects, peer IP and hosts
/// for the initial request AND all returned references.
pub fn parse_hls(base: &str, bytes: &[u8]) -> Result<HlsManifest, HlsError> {
    if bytes.len() > MAX_MANIFEST_BYTES {
        return Err(HlsError::TooLarge);
    }
    let base = Url::parse(base).map_err(|_| HlsError::UnsafeReference)?;
    if base.scheme() != "https"
        || base.host_str().is_none()
        || !base.username().is_empty()
        || base.password().is_some()
    {
        return Err(HlsError::UnsafeReference);
    }
    let body = std::str::from_utf8(bytes).map_err(|_| HlsError::Invalid)?;
    let mut lines = body.lines().map(str::trim);
    if lines.next() != Some("#EXTM3U") {
        return Err(HlsError::Invalid);
    }
    let mut references = Vec::new();
    let mut is_master = false;
    let mut has_media_segments = false;
    let mut end_list = false;
    let mut expect_variant = false;
    for line in lines {
        if line.is_empty() {
            continue;
        }
        if line == "#EXT-X-ENDLIST" {
            end_list = true;
        }
        if line.starts_with("#EXT-X-STREAM-INF:") {
            is_master = true;
            expect_variant = true;
            continue;
        }
        if line.starts_with("#EXT-X-MEDIA:")
            || line.starts_with("#EXT-X-KEY:")
            || line.starts_with("#EXT-X-MAP:")
            || line.starts_with("#EXT-X-SESSION-KEY:")
        {
            if let Some(uri) = extract_quoted_uri(line) {
                if references.len() >= MAX_REFERENCES {
                    return Err(HlsError::TooManyReferences);
                }
                references.push(resolve(&base, uri)?);
            }
            continue;
        }
        if line.starts_with('#') {
            continue;
        }
        if references.len() >= MAX_REFERENCES {
            return Err(HlsError::TooManyReferences);
        }
        references.push(resolve(&base, line)?);
        if !expect_variant {
            has_media_segments = true;
        }
        expect_variant = false;
    }
    Ok(HlsManifest {
        is_master,
        references,
        has_media_segments,
        end_list,
    })
}

fn extract_quoted_uri(line: &str) -> Option<&str> {
    let at = line.find("URI=\"")? + 5;
    let tail = &line[at..];
    Some(&tail[..tail.find('"')?])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn master_variant_urls() {
        let m = parse_hls(
            "https://media.example/live/index.m3u8",
            b"#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH=10000\n720/index.m3u8\n",
        )
        .unwrap();
        assert!(m.is_master);
        assert!(!m.has_media_segments);
        assert_eq!(m.references[0], "https://media.example/live/720/index.m3u8");
    }
    #[test]
    fn media_and_key_urls() {
        let m = parse_hls(
            "https://media.example/live/index.m3u8",
            b"#EXTM3U\n#EXT-X-KEY:METHOD=AES-128,URI=\"key.bin\"\n#EXTINF:4,\nchunk.ts\n",
        )
        .unwrap();
        assert!(m.has_media_segments);
        assert_eq!(m.references.len(), 2);
    }
    #[test]
    fn reject_cross_scheme() {
        assert_eq!(
            parse_hls(
                "https://media.example/index.m3u8",
                b"#EXTM3U\nfile:///etc/passwd\n"
            )
            .unwrap_err(),
            HlsError::UnsafeReference
        );
    }
    #[test]
    fn reject_html_and_large() {
        assert_eq!(
            parse_hls("https://media.example/index.m3u8", b"<html>video</html>").unwrap_err(),
            HlsError::Invalid
        );
        assert_eq!(
            parse_hls(
                "https://media.example/index.m3u8",
                &vec![0; MAX_MANIFEST_BYTES + 1]
            )
            .unwrap_err(),
            HlsError::TooLarge
        );
    }
}
