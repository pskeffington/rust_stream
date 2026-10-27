//! Rust Stream core. Safe source definitions, deterministic quad state and a media backend contract.
//! Decoding is optional; the native-video feature requires system GStreamer libraries.

mod catalog;
mod chart;
mod free_tv;
mod hls;
mod hls_gate;
mod iptv_org;
mod live_video;
mod media;
mod playback;
mod quad;
mod retry;
mod source;
pub use playback::{Playback, PlaybackConfig, PlaybackSnapshot, PlaybackStatus};

pub use catalog::{DiscoveryState, FeedHealth, FeedRecord, PROVIDERS, Provider};
pub use chart::{ChartCamera, ChartError, parse_chart_cameras};
pub use free_tv::{FreeTvPlayer, FreeTvService, PlayerStatus};
pub use hls::{HlsError, HlsManifest, parse_hls};
pub use hls_gate::{HlsHostPolicy, HlsPolicyError, preflight_hls};
pub use iptv_org::{
    CHANNELS_URL, IptvCandidate, IptvImport, IptvImportError, STREAMS_URL, import_iptv_org,
};
pub use live_video::{
    CandidateStatus, LIVE_NEWS_CANDIDATES, LiveVideoCandidate, VideoEndpointError,
    validate_direct_hls_url,
};
pub use media::{FeedCollection, MediaView, ViewMode};
pub use quad::{
    FrameRgba, MediaBackend, MediaSession, QUAD_SLOTS, Quad, Slot, SlotStatus, StreamError,
};
pub use retry::RetryBackoff;
pub use source::{
    Source, SourceKind, SourceManifest, SourcePolicy, SourceScope, SourceValidationError,
    is_non_global_ip,
};

#[cfg(feature = "native-video")]
mod gst_video;
#[cfg(feature = "native-video")]
pub use gst_video::GstBackend;
