# Repository index

This is the entrypoint for humans, code-search tools, and coding agents. Use these paths instead of guessing which executable owns a feature.

| Area | Canonical path | Responsibility |
|---|---|---|
| Native desktop player | `crates/rust-stream-demo/src/main.rs` | NEWS-first egui layout, navigation, named stream editor, saved user presets, four slot assignments |
| Native player adapter | `crates/rust-stream-demo/src/native_preview.rs` | Texture painting and operator-selected HLS playback adapter |
| Core library | `crates/rust-stream-core/src/lib.rs` | Public reusable core API |
| Source trust definitions | `crates/rust-stream-core/src/source.rs` | Source types and definition-time validation; not connection-time SSRF protection |
| GStreamer integration | `crates/rust-stream-core/src/gst_video.rs` | Native video decoding and pipeline handling |
| Catalog executable | `crates/rust-stream-catalog/src/main.rs` | Candidate metadata discovery/cache, **not** a video player |
| Native launch | `scripts/launch-native-ui.sh` | Build and run local desktop UI explicitly |
| Owned HLS fixture | `crates/rust-stream-core/tests/fixtures/hls/` | Deterministic decoder validation |

## Documentation routes

- [README](../README.md): current entrypoint, build, and launch.
- [Native video](NATIVE_VIDEO.md): controls, decoding, tests, known limitations.
- [Architecture](ARCHITECTURE.md): crate separation and Rust-Boy embedding boundary.
- [Roadmap](ROADMAP.md): milestone status, not proof of network playback.
- [News video policy](NEWS_VIDEO_POLICY.md): live moving video only.
- [Direct feed catalog](DIRECT_FEED_CATALOG.md): evidence and approval boundaries.
- [Rust-Boy integration](RUST_BOY_MEDIA_INTEGRATION.md): integration with the host.
- [IPTV-org adapter](IPTV_ORG_ADAPTER.md): discovery vs approval.
- [Maryland CHART adapter](CHART_ADAPTER.md): camera catalog constraints.

## Critical invariants

1. NEWS is the default section, not the catalog executable.
2. Each slot has its own decoder; label/status must come from actual slot state.
3. The UI's selected source or a successful manifest download is not evidence of decoded live frames.
4. Custom named URLs are user-local; no credentials should be embedded in them.
5. Never confuse catalog candidates with verified live streams, nor assume a broadcaster grants redistribution rights.
6. `source.rs` validation does **not** constrain DNS resolution, redirects, nested manifests or segment requests.
7. The app's audio-owner behavior needs an independently verified single-source test; do not infer it from video-only test success.
8. Do not overwrite divergent or modified local work to fix an old-UI screenshot; inspect its branch and executable first.

## Indexing conventions

Treat this file and the README as the navigation roots. Keep code identifiers, file paths, and executable names stable in docs. Whenever moving a module, update both the index and links in the README. Clearly separate **implemented**, **locally tested**, **verified live**, and **candidate** claims. Avoid duplicating full source catalogs in multiple documentation files.
