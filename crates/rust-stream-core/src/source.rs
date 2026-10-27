use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use thiserror::Error;
use url::{Host, Url};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Rtsp,
    Mjpeg,
    Hls,
    Snapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceScope {
    Public,
    LocalApproved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub label: String,
    pub endpoint: String,
    pub kind: SourceKind,
    pub scope: SourceScope,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool { true }

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SourceManifest {
    pub sources: Vec<Source>,
}

/// A definition-time gate; NOT a connection-time DNS/redirect/peer-IP validator.
/// A production adapter MUST independently enforce network policy when opening sockets,
/// on redirects, for HLS segments, and on each reconnect.
#[derive(Debug, Default, Clone)]
pub struct SourcePolicy {
    pub allowed_hosts: BTreeSet<String>,
    pub allow_cleartext: bool,
    pub allow_local_networks: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SourceValidationError {
    #[error("invalid source identifier or label")]
    BadIdentity,
    #[error("source URL is invalid")]
    BadUrl,
    #[error("source protocol and URL scheme do not match")]
    UnsupportedScheme,
    #[error("embedded credentials are not permitted; use protected credential storage")]
    CredentialsInUrl,
    #[error("URL fragments are not permitted")]
    Fragment,
    #[error("host is not explicitly allowlisted")]
    HostNotAllowed,
    #[error("cleartext transport needs explicit permission")]
    CleartextDenied,
    #[error("private or special-purpose addresses require an approved local scope")]
    PrivateAddressDenied,
    #[error("local scope requires explicit operator approval")]
    LocalScopeDenied,
}

impl Source {
    /// Validate configuration metadata only. Do not use as a network security boundary.
    pub fn validate_definition(&self, policy: &SourcePolicy) -> Result<(), SourceValidationError> {
        if self.id.is_empty() || self.id.len() > 64
            || !self.id.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self.label.trim().is_empty() || self.label.len() > 100
        {
            return Err(SourceValidationError::BadIdentity);
        }

        if self.endpoint.len() > 2048 || self.endpoint.chars().any(char::is_control) {
            return Err(SourceValidationError::BadUrl);
        }
        let url = Url::parse(&self.endpoint).map_err(|_| SourceValidationError::BadUrl)?;
        if !url.username().is_empty() || url.password().is_some() {
            return Err(SourceValidationError::CredentialsInUrl);
        }
        if url.fragment().is_some() {
            return Err(SourceValidationError::Fragment);
        }
        let allowed_scheme = match self.kind {
            SourceKind::Rtsp => matches!(url.scheme(), "rtsp" | "rtsps"),
            SourceKind::Mjpeg | SourceKind::Hls | SourceKind::Snapshot =>
                matches!(url.scheme(), "http" | "https"),
        };
        if !allowed_scheme {
            return Err(SourceValidationError::UnsupportedScheme);
        }
        if matches!(url.scheme(), "http" | "rtsp") && !policy.allow_cleartext {
            return Err(SourceValidationError::CleartextDenied);
        }
        if self.scope == SourceScope::LocalApproved && !policy.allow_local_networks {
            return Err(SourceValidationError::LocalScopeDenied);
        }
        let host = url.host().ok_or(SourceValidationError::BadUrl)?;
        let canonical_host = match &host {
            Host::Domain(domain) => domain.to_ascii_lowercase(),
            Host::Ipv4(ip) => ip.to_string(),
            Host::Ipv6(ip) => ip.to_string(),
        };
        if !policy.allowed_hosts.contains(&canonical_host) {
            return Err(SourceValidationError::HostNotAllowed);
        }
        if let Host::Ipv4(ip) = host {
            if non_global_v4(ip) && self.scope != SourceScope::LocalApproved {
                return Err(SourceValidationError::PrivateAddressDenied);
            }
        } else if let Host::Ipv6(ip) = host {
            if non_global_v6(ip) && self.scope != SourceScope::LocalApproved {
                return Err(SourceValidationError::PrivateAddressDenied);
            }
        }
        Ok(())
    }
}

fn non_global_v4(ip: Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_private() || ip.is_loopback() || ip.is_link_local()
        || ip.is_unspecified() || ip.is_multicast() || ip.is_broadcast()
        || o[0] == 0 || o[0] >= 240 || o[0] == 100 && (64..=127).contains(&o[1])
        || o[0] == 192 && o[1] == 0 && o[2] == 0
        || o[0] == 192 && o[1] == 0 && o[2] == 2
        || o[0] == 198 && (o[1] == 18 || o[1] == 19 || o[1] == 51 && o[2] == 100)
        || o[0] == 203 && o[1] == 0 && o[2] == 113
}
fn non_global_v6(ip: Ipv6Addr) -> bool {
    ip.is_loopback() || ip.is_unspecified() || ip.is_multicast()
        || (ip.segments()[0] & 0xfe00) == 0xfc00
        || (ip.segments()[0] & 0xffc0) == 0xfe80
        || ip.to_ipv4_mapped().is_some_and(non_global_v4)
}
pub fn is_non_global_ip(ip: IpAddr) -> bool {
    match ip { IpAddr::V4(v4) => non_global_v4(v4), IpAddr::V6(v6) => non_global_v6(v6) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(endpoint: &str) -> Source {
        Source { id: "cam-01".into(), label: "Approved feed".into(),
            endpoint: endpoint.into(), kind: SourceKind::Snapshot,
            scope: SourceScope::Public, enabled: true }
    }
    fn policy(host: &str) -> SourcePolicy {
        SourcePolicy { allowed_hosts: [host.to_string()].into(), ..Default::default() }
    }
    #[test]
    fn disallows_unknown_hosts() {
        assert_eq!(sample("https://unknown.example/image.jpg").validate_definition(&policy("other.example")),
            Err(SourceValidationError::HostNotAllowed));
    }
    #[test]
    fn disallows_embedded_passwords() {
        assert_eq!(sample("https://user:pass@camera.example/still.jpg").validate_definition(&policy("camera.example")),
            Err(SourceValidationError::CredentialsInUrl));
    }
    #[test]
    fn denies_private_ip_in_public_scope() {
        assert_eq!(sample("https://192.168.10.4/still.jpg").validate_definition(&policy("192.168.10.4")),
            Err(SourceValidationError::PrivateAddressDenied));
    }
    #[test]
    fn local_network_requires_explicit_opt_in() {
        let mut s = sample("http://192.168.10.4/still.jpg");
        s.scope = SourceScope::LocalApproved;
        let p = SourcePolicy { allowed_hosts: ["192.168.10.4".into()].into(),
            allow_cleartext: true, allow_local_networks: true };
        assert!(s.validate_definition(&p).is_ok());
    }
    #[test]
    fn rejects_insecure_public_feed_by_default() {
        assert_eq!(sample("http://camera.example/still.jpg").validate_definition(&policy("camera.example")),
            Err(SourceValidationError::CleartextDenied));
    }
    #[test]
    fn rejects_wrong_scheme() {
        assert_eq!(sample("rtsp://camera.example/video").validate_definition(&policy("camera.example")),
            Err(SourceValidationError::UnsupportedScheme));
    }
    #[test]
    fn rejects_metadata_service_address() {
        let p = SourcePolicy { allowed_hosts: ["169.254.169.254".into()].into(),
            allow_cleartext: true, ..Default::default() };
        assert_eq!(sample("http://169.254.169.254/latest/meta-data").validate_definition(&p),
            Err(SourceValidationError::PrivateAddressDenied));
    }
}
