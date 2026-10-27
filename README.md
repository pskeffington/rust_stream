# Rust Stream

MIT-licensed, reusable Rust video engine and compact egui video wall for Rust-Boy 2000. Video decodes inside the process and renders in the application window.

The optional GStreamer backend produces 640×360 RGBA video at up to 15 fps. Four independent players support source replacement, stop/restart, focus/grid viewing, stalled-stream detection and three bounded reconnects. The default build opens an offline shell without media libraries or network access.

## Run on macOS

Use Rust 1.95+ and GStreamer. This repository contains multiple executables; launch the native video UI explicitly rather than an older installed app.

```sh
brew install gstreamer
git fetch origin main
git pull --ff-only origin main
sh scripts/launch-native-ui.sh
```

The script builds and executes `target/debug/rust-stream-demo` from the current checkout, and refuses to launch when the checkout is known to be behind `origin/main`. It does not discard local modifications. Resolve divergent branches or uncommitted edits manually; do not use `git reset --hard` merely to update the UI.

The native player starts on **NEWS**. Use **+ STREAM** to enter a human-readable station name and trusted public HTTPS HLS URL, select Box 1–4, and press **PLAY**. Saved custom streams are kept in the user's local configuration directory and loaded on future launches. Alternatively set `RUST_STREAM_HLS_NAME_1` and `RUST_STREAM_HLS_URL_1` through `_4` in the launch environment. **FREE TV** selects COMET TV as a candidate preset; availability and live decoding are not guaranteed. For four network-free patterns, run:

```sh
RUST_STREAM_TEST_PATTERN=1 cargo run -p rust-stream-demo --features native-video
```

GStreamer handles playlists, segments and redirects internally; full connection-time DNS, peer-address and nested HLS request enforcement remains pending. Only use streams and child resources the operator trusts and is authorized to access. Do not put credentials in stream URLs. See [native playback](docs/NATIVE_VIDEO.md) and [source policy](docs/DIRECT_FEED_CATALOG.md).

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

- [Repository index](docs/INDEX.md)\n- [Architecture](docs/ARCHITECTURE.md)
- [Native video, local setup and validation](docs/NATIVE_VIDEO.md)
- [Roadmap](docs/ROADMAP.md)
- [Feed catalog and approval status](docs/DIRECT_FEED_CATALOG.md)
- [Rust-Boy integration](docs/RUST_BOY_MEDIA_INTEGRATION.md)
- [Disabled example sources](examples/sources.example.json)

Project code and owned test footage use MIT. Check media plugin and codec licenses independently. An entry in a catalog or an HTTP response does not establish authorization or working video.
