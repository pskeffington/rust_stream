//! Rust Stream core. Safe source definitions, deterministic quad state and a media backend contract.
//! No network I/O or media decoding is implemented in this crate.

mod chart;
mod catalog;
mod media;
mod free_tv;
mod hls;
mod hls_gate;
mod live_video;
mod quad;
mod retry;
mod source;

pub use chart::{parse_chart_cameras, ChartCamera, ChartError};
pub use catalog::{DiscoveryState, FeedHealth, FeedRecord, Provider, PROVIDERS};
pub use hls_gate::{preflight_hls,HlsHostPolicy,HlsPolicyError};
pub use hls::{parse_hls, HlsError, HlsManifest};
pub use live_video::{validate_direct_hls_url, CandidateStatus, LiveVideoCandidate, VideoEndpointError, LIVE_NEWS_CANDIDATES};
pub use free_tv::{FreeTvPlayer, FreeTvService, PlayerStatus};
pub use media::{FeedCollection, MediaView, ViewMode};
pub use quad::{FrameRgba, MediaBackend, MediaSession, Quad, Slot, SlotStatus, StreamError, QUAD_SLOTS};
pub use retry::RetryBackoff;
pub use source::{Source, SourceKind, SourceManifest, SourcePolicy, SourceScope, SourceValidationError};

#[cfg(feature = "native-video")]
mod gst_video;
#[cfg(feature = "native-video")]
pub use gst_video::GstBackend;
