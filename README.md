# Rust Stream

MIT-licensed, reusable Rust video engine and compact egui video wall for Rust-Boy 2000. Video decodes inside the process and renders in the application window.

The optional GStreamer backend produces 640×360 RGBA video at up to 15 fps. Four independent players support source replacement, stop/restart, focus/grid viewing, stalled-stream detection and three bounded reconnects. The default build opens an offline shell without media libraries or network access.

## Run on macOS

The demo requires Rust **1.95 or newer** (eframe 0.37). The core declares Rust 1.85; its minimum toolchain has not been separately tested.

```sh
brew install gstreamer
# Four network-free moving patterns, decoded by GStreamer:
RUST_STREAM_TEST_PATTERN=1 cargo run -p rust-stream-demo --features native-video
```

For an **authorized, trusted public HTTPS HLS source**, set `RUST_STREAM_HLS_URL_1` through `_4` before launching. The legacy `RUST_STREAM_HLS_URL` is a fallback for channel 1. Open **SOURCES**, choose a source for each slot and press **Apply / Restart**. **Remove** stops and clears one slot; **STOP ALL** stops all four. Click a video to focus it and **GRID** to return. There are no bundled or automatically connected broadcaster endpoints.

GStreamer retrieves playlists, segments and redirects internally. Connection-time DNS, peer-address and nested-request enforcement is **not implemented**. Use only publisher-controlled endpoints and child resources you trust; do not enable arbitrary user URLs, credentials or internal cameras. See [native playback and its trust boundary](docs/NATIVE_VIDEO.md).

Category tabs currently share the configured wall. Provider integrations, category catalogs, remembered TV channel selection, audio and Raspberry Pi validation remain pending.

## Verification

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p rust-stream-demo --features native-video
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

On macOS with Rust 1.99.1 / GStreamer 1.28.8, the default and native suites passed, including loopback HTTP HLS decoding, failure propagation, four native decoders, cancellation, frame stride and compact layout tests. Four synthetic feeds, focus/grid, stop and shutdown were also visually exercised in the native window. This does not establish broadcaster permission, HTTPS broadcaster compatibility, Rust-Boy embedding or Pi performance. See [validation details](docs/NATIVE_VIDEO.md#validation-record).

CI tests both the default workspace and a native feature build on Ubuntu with GStreamer libraries/plugins installed.

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Native video, local setup and validation](docs/NATIVE_VIDEO.md)
- [Roadmap](docs/ROADMAP.md)
- [Feed catalog and approval status](docs/DIRECT_FEED_CATALOG.md)
- [Rust-Boy integration](docs/RUST_BOY_MEDIA_INTEGRATION.md)
- [Disabled example sources](examples/sources.example.json)

Project code and owned test footage use MIT. Check media plugin and codec licenses independently. An entry in a catalog or an HTTP response does not establish authorization or working video.
