//! Offline provider registry. Discovery endpoints are NOT playable video URLs.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryState {
    Documented,
    RequiresKey,
    MetadataOnly,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Provider {
    pub id: &'static str,
    pub region: &'static str,
    pub agency: &'static str,
    pub discovery_url: &'static str,
    pub state: DiscoveryState,
    pub media_field: Option<&'static str>,
}

pub const PROVIDERS: &[Provider] = &[
    Provider {
        id: "md-chart",
        region: "Maryland",
        agency: "Maryland CHART",
        discovery_url: "https://chart.maryland.gov/DataFeeds/GetDataFeeds",
        state: DiscoveryState::Documented,
        media_field: Some("publicVideoURL"),
    },
    Provider {
        id: "ny-511",
        region: "New York",
        agency: "511NY",
        discovery_url: "https://prod-ny.ibi511.com/developers/help",
        state: DiscoveryState::RequiresKey,
        media_field: Some("VideoUrl"),
    },
    Provider {
        id: "ca-caltrans",
        region: "California",
        agency: "Caltrans QuickMap",
        discovery_url: "https://publish.data.ca.gov/dataset/closed-circuit-television",
        state: DiscoveryState::MetadataOnly,
        media_field: None,
    },
    Provider {
        id: "dc-ddot",
        region: "District of Columbia",
        agency: "DDOT",
        discovery_url: "https://maps2.dcgis.dc.gov/dcgis/rest/services/DDOT/TrafficOperations/FeatureServer/layers",
        state: DiscoveryState::MetadataOnly,
        media_field: None,
    },
    Provider {
        id: "ma-massdot",
        region: "Massachusetts",
        agency: "MassDOT",
        discovery_url: "https://gisstg.massdot.state.ma.us/rh/rest/services/Hosted/MassDOT_Traffic_Cams/FeatureServer",
        state: DiscoveryState::MetadataOnly,
        media_field: None,
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeedHealth {
    Discovered,
    Verifying,
    Healthy,
    Stale,
    Unavailable,
    Unauthorized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedRecord {
    pub id: String,
    pub provider_id: String,
    pub label: String,
    pub region: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    /// Filled only after policy checks and media validation.
    pub approved_source_id: Option<String>,
    pub health: FeedHealth,
    pub verified_at_unix: Option<u64>,
}

impl FeedRecord {
    pub fn is_usable(&self, now_unix: u64, ttl_seconds: u64) -> bool {
        self.health == FeedHealth::Healthy
            && self.approved_source_id.is_some()
            && self
                .verified_at_unix
                .is_some_and(|at| at <= now_unix && now_unix - at <= ttl_seconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_is_not_a_live_feed() {
        let mut feed = FeedRecord {
            id: "test".into(),
            provider_id: "md-chart".into(),
            label: "Test".into(),
            region: "MD".into(),
            latitude: None,
            longitude: None,
            approved_source_id: None,
            health: FeedHealth::Discovered,
            verified_at_unix: None,
        };
        assert!(!feed.is_usable(100, 60));
        feed.health = FeedHealth::Healthy;
        feed.approved_source_id = Some("source-1".into());
        feed.verified_at_unix = Some(100);
        assert!(feed.is_usable(110, 60));
        assert!(!feed.is_usable(161, 60));
        assert!(!feed.is_usable(99, 60));
    }
}
