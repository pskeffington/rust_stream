# Development roadmap

- [x] P0 Core workspace, source definitions, frame checks, four slots, offline preview, tests.
- [ ] P0.1 Build/format/clippy verified with Rust toolchain and generated Cargo.lock.
- [ ] P1 Feature-gated GStreamer RTSP/RTSPS worker adapter with bounded appsink.
- [ ] P1.1 Connection-time destination checks, redirect checks, TLS and protected credentials.
- [ ] P2 Bind decoded frames to egui textures, live per-tile metrics and reconnection.
- [ ] P2.1 Add MJPEG, HTTPS still frames and optional HLS.
- [ ] P3 Curate and verify licensed direct Boston, NYC, DC, SF and DOT endpoints; never assume a feed is live.
- [ ] P3.1 Explicit authorized internal/vault camera registration and revocation.
- [ ] P4 macOS stress test, Raspberry Pi hardware decode/capability tests.
- [ ] P5 Rust-Boy native embedding, cross compilation and deployment hardening.

Acceptance: one broken stream must not hang other tiles; stale frames never overwrite new ones; bounded backoff; policy retained over reconnects, DNS changes and redirects; all four tiles gracefully degrade; no scrolling on compact HUDs; use no external media player.

- [ ] NEWS policy: direct authorized LIVE VIDEO only; no RSS/Atom/headline pipelines, article rendering, or web embeds. See [NEWS_VIDEO_POLICY](NEWS_VIDEO_POLICY.md).
