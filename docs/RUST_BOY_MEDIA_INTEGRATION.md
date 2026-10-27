# Rust-Boy media center integration contract

## User task
A handheld operator opens MEDIA, taps SECURITY to see a camera wall, taps NEWS for an authorized IP news feed, or taps TRAFFIC to check DOT / roadway conditions. A tap on one tile focuses it. Back returns to the quad. Feed editing, geolocation and credentials live behind global Settings.

## Existing Rust-Boy reality (2026-10-26)
- Repository: pskeffington/pip_boy.
- Desktop security chief extension already mounts a 2x2 camera wall using assets/cameras.js.
- Current camera catalog is four publisher YouTube URLs in pipboy-core/src/public_cameras.rs, rendered as iframe embeds. These are NOT direct IP camera feeds, and must not be represented as native streams.
- Dashboard web assets are not the same as the native egui Rust Stream demo.
- Plan a feature-flagged migration; do not silently replace the security view before native decoder readiness and tests.

## Navigation
MEDIA > SECURITY | NEWS | TRAFFIC | CUSTOM. Four channels per collection. Collections are not privileges: Security may show only cameras for which the operator has permission. News and traffic may use public approved streams. If a collection has zero configured feeds, show a clear empty state, never a misleading LIVE badge.

## Handheld UX
- One page, no scroll bars or source-entry forms.
- Four tiles with short name, connection state and last-frame age; selectable with touch-friendly targets.
- Focus enlarges a single stream. One BACK/GRID action returns.
- Auto pause/deprioritize off-screen feeds when device is constrained.
- Show OFFLINE, STALLED, or UNVERIFIED; do not infer working status from saved metadata.
- Media playback and newsroom content should not block emergency messaging, location/mapping or radio.

## Configuration and discovery
Rust-Boy Settings > Media Feeds: add/remove sources, choose category, select one of 4 tile positions, configure authorized local camera trust/credentials, verify stream, order favorites and disable feed. Store secrets in OS protected storage; a source manifest holds a reference, never a password.

Location preference is universal and not re-entered on this view. Traffic collection can use location-to-county/nearby DOT routing from the existing Rust-Boy location provider, and choose the nearest verified live direct feed when there is an active one. Never claim a DOT feed is live without frame/heartbeat evidence; fallback to known permitted static road images if appropriate.

## Adapter boundary
- rust-stream-core: FeedCollection and MediaView selection, four slots, Source/backend/session traits, and reusable Playback workers.
- Rust-Boy integration adapter: maps existing authentication, role grants, settings, location and feed catalogs into source assignments.
- Native pipelines currently live behind rust-stream-core/native-video. The demo owns its egui textures; extracting a reusable egui adapter and testing it in Rust-Boy remain pending. Browser iframe/video and YouTube embeds are legacy-only and must not become the new core path.
- Security: approval per vault camera; deny internal/LAN targets until provisioned; network-level redirect, DNS and playlist protections; revocation must tear down the worker.

## Acceptance gates
1. Four assigned feeds in selected collection; switching collection cancels or deprioritizes old workers.
2. Security cameras only for approved roles, without leaking thumbnails or URLs to unauthorized roles.
3. News stream works as direct IP media without launching a browser or external player.
4. Traffic mode uses nearest verified usable feed, not nearest URL string.
5. Display fits handheld in 2x2/focus, zero scroll.
6. Offline use shows last-known metadata without falsely showing archived video as LIVE.
7. Existing Security camera wall continues to function until replacement is ready.

## Current reusable playback API

A host can use the core without starting the desktop demo:

```rust
use std::sync::Arc;
use rust_stream_core::{GstBackend, Playback, Source};

fn start_approved(source: Source) -> Result<Playback, rust_stream_core::StreamError> {
    let backend = Arc::new(GstBackend::trusted_hls(source.clone())?);
    let mut player = Playback::default();
    player.start(backend, source);
    Ok(player)
}
```

Poll `player.poll()` during host updates; upload its optional RGBA frame into the host renderer and display its status. Call `stop()` on removal/revocation; drop the player during shutdown to join the worker. Source replacement invalidates the previous generation. See [native lifecycle and trust limitations](NATIVE_VIDEO.md) before enabling any endpoint. This API and the standalone demo have been tested; an actual Rust-Boy integration has not.
