//! Rust Stream core. Safe source definitions, deterministic quad state and a media backend contract.
//! No network I/O or media decoding is implemented in this crate.

mod catalog;
mod media;
mod quad;
mod retry;
mod source;

pub use catalog::{DiscoveryState, FeedHealth, FeedRecord, Provider, PROVIDERS};
pub use media::{FeedCollection, MediaView, ViewMode};
pub use quad::{FrameRgba, MediaBackend, MediaSession, Quad, Slot, SlotStatus, StreamError, QUAD_SLOTS};
pub use retry::RetryBackoff;
pub use source::{Source, SourceKind, SourceManifest, SourcePolicy, SourceScope, SourceValidationError};
