# Rust Stream

MIT-licensed modular four-screen Rust IP-streaming component for Rust-Boy 2000. Planned: direct IP/RTSP, public city and DOT cameras, explicitly authorized vault cameras, single-TV screen and multimedia screen. Integrated Rust rendering without launching external media applications.

## P0 delivered

- Rust workspace and reusable rust-stream-core with four independent slots, source manifest and safe frame size cap.
- Offline-only egui demonstration UI with 2x2 tiles and click-to-focus.
- Typed definitions for RTSP, MJPEG, HLS and snapshot; network/decoder adapters remain TODO.
- Metadata allowlist and local scope checks. These ARE NOT sufficient for connection-time SSRF protection.
- Unit tests, example disabled manifest, initial technical literature matrix and phased roadmap.

## Commands (not yet executed or validated)

    cargo fmt --all -- --check
    cargo test -p rust-stream-core
    cargo run -p rust-stream-demo

Native graphics system libraries may be required. Currently no Rust compiler exists in this execution environment.

## Navigation

- [Architecture](docs/ARCHITECTURE.md)
- [Literature and best practices](docs/LITERATURE_MATRIX.md)
- [Roadmap](docs/ROADMAP.md)
- [Disabled example sources](examples/sources.example.json)

Project code uses MIT; check all media plugin/codec licenses separately. No source is considered live or authorized merely because it appears in a manifest.
