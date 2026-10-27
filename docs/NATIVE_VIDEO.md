# Native video

## Local development

Install Rust 1.95+ for the eframe 0.37 demo. On macOS, `brew install gstreamer` provides the runtime, development libraries and bundled plugins. `pkg-config --modversion gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0` should succeed. The first GStreamer initialization may scan installed plugins and take considerably longer than subsequent runs. Homebrew's scanner can emit Python GI/GTK warnings unrelated to the CPU appsink pipeline; do not treat those warnings alone as a successful or failed playback test.

On Debian/Ubuntu, install `pkg-config libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-libav`.

```sh
RUST_STREAM_TEST_PATTERN=1 cargo run -p rust-stream-demo --features native-video
```

The test-pattern command starts four network-free video patterns. Without that flag the player opens NEWS first and attempts explicitly configured or saved HTTPS HLS feeds. Use **+ STREAM** to add a named stream in the player; **STREAMS** manages all four slots. Click a tile to focus it, and **GRID** to return to the wall. No public broadcaster is considered verified just because its URL is configured.

## Startup NEWS and channel naming

NEWS is the initial tab. At startup the UI assigns the first four populated HTTPS HLS environment inputs to boxes 1-4, respectively, and immediately starts their native player sessions. Use a descriptive name for each input instead of generic source numbering:

```sh
RUST_STREAM_HLS_NAME_1="Local News" \
RUST_STREAM_HLS_URL_1="https://publisher.example/live.m3u8" \
cargo run -p rust-stream-demo --features native-video
```

Repeat `RUST_STREAM_HLS_NAME_2` through `_4` with their matching URL variables. These are operator-managed inputs, not automatically discovered or verified news stations. Missing inputs stay off and display a configuration hint; no fake live station is substituted. On selecting FREE TV, Box 1 selects COMET's candidate HLS URL, with other boxes off. Changing tabs resets the displayed wall for that category; customized choices in a category do not persist after leaving it. This is a deliberate interim behavior pending persistent per-category presets and working-stream health history.

The header provides a Box 1 quick picker and a STREAMS window for managing all four boxes. Operator-supplied names show in the channel picker and video label. The active status remains driven by playback state, not feed labels or a successful manifest lookup.

## Inline named stream input

Click **+ STREAM** in the player's main toolbar. Select Box 1-4, provide a human-readable **Name**, enter a trusted HTTPS HLS stream URL and click **PLAY** (or press Enter in the URL field). The player uses the supplied name for the video overlay and source selector. Clicking a video to focus it before opening + STREAM preselects that box. Custom names and URLs are saved locally in `~/.config/rust-stream/streams.json` (or `$XDG_CONFIG_HOME/rust-stream/streams.json`), with restrictive file permissions on Unix. PLAY saves the entry and starts playback. STREAMS offers Save + Play and Clear. Saved entries are restored on launch; NEWS prefers an explicitly configured environment feed for each box, then its saved custom stream. The CUSTOM tab starts saved custom streams. There is no public URL discovery. A successful click is not proof of a decoded stream: inspect the tile status and frames.

## Primary feeds and source selection

The header provides the **BOX 1** quick selector. **STREAMS** opens all four selectors, where users can restart, save, play or clear a custom feed. The available options are Off, COMET TV (candidate), four operator-configured environment feeds, named custom HLS, and Test pattern. Custom names and URLs persist locally; do not include credentials or private network addresses. Changing sections currently resets active playback assignments to that section's defaults.

COMET is selected when opening FREE TV, but live public-network playback and publisher terms require separate verification. The media backend currently does not enforce the complete nested HLS request graph, so the URL editor is for trusted operator-selected feeds, not untrusted remote submissions.

## Decoder and worker lifecycle

`GstBackend::trusted_hls(source)` binds a backend to an exact enabled, public HTTPS HLS source, validating its definition. `open` rejects another source. Construction is an explicit assertion of operator trust in the publisher and all child resources; it does not establish authorization or network safety. `GstBackend::test_pattern(index)` creates a network-free backend through the same raw-frame pipeline.

The decoder uses `uridecodebin3`, selects video streams only, links dynamic video pads using current caps or a caps query, and negotiates CPU RGBA through `videoconvert`, `videoscale`, `videorate` and appsink. Queried caps are required when GStreamer exposes a pad before assigning current caps. Accepting native raw-memory formats during decode and negotiating against the CPU consumer avoids the previous macOS VideoToolbox negotiation failure.

Output is fixed to 640×360 square pixels and 15 fps, with a clock-synchronized, one-buffer dropping appsink. Actual sample dimensions, offsets and strides come from `VideoInfo` / `VideoFrameRef`; row padding is removed before creating a bounded `FrameRgba`. Negative strides and invalid dimensions/layouts fail safely. Bus errors and EOS propagate to the worker; bus messages are drained rather than accumulated. Audio output is not implemented.

Each `Playback` owns one worker and one replaceable frame mailbox. Network open, decoding and pipeline teardown run outside egui. Start/stop/source replacement update a generation and coalesce to one desired source. A retired generation cannot publish into its replacement. Slow consumers receive the latest frame without blocking the producer or accumulating history. Source replacement clears the old UI texture immediately. Hidden slots continue being polled and rendered into their own texture handles.

States are Offline, Loading, Live, Stalled, Reconnecting and Failed. Defaults: stalled after 5 seconds without a frame, reconnect after 15 seconds without a frame, and at most three reconnects (0.5, 1 and 2 seconds delay) per operator start. Successful frames do not reset this lifetime retry budget; an explicit restart resets it. Source HTTP elements receive a 10-second timeout when supported. Nested GStreamer requests and pipeline teardown remain subject to framework behavior. Rust cannot forcibly terminate a blocked native call. Drop signals shutdown and joins the worker; no decoder thread is intentionally detached.

Decoded frames are capped; compressed buffers and hardware decoder allocations are not a complete process memory budget. Resolution/fps configuration, bandwidth budgets, memory-pressure metrics and hardware acceleration tuning remain pending.

## Trust boundary

Definition-time URL checks are not SSRF protection. GStreamer controls DNS, sockets, redirected requests, variant playlists, keys and media segments. This backend cannot enforce destination restrictions on that entire request graph. **Only operator-configured publishers and child resources already trusted and authorized may be used.** The in-player custom URL editor is intended only for trusted operator-selected public HTTPS HLS streams; public untrusted submissions, private cameras, and credential support are not provided. An operator environment variable is a trust decision, not proof of licensing.

The test-only HTTP server binds loopback and serves only embedded, owned fixtures. Its private helper bypasses production URL selection only inside `#[cfg(test)]`; the public HLS constructor continues to reject loopback/private literals. No network policy is weakened to run the tests. Diagnostic `GST_DEBUG` output can contain URLs and should be handled accordingly.

## Validation record

Validated on macOS arm64, Rust 1.99.1, eframe/egui 0.37.0, GStreamer 1.28.8 (2026-10-27):

- Default workspace format, check, tests and warning-free Clippy.
- Native demo feature check; all-feature workspace tests and warning-free Clippy.
- Owned three-segment H.264 HLS fixture over a test-only HTTP server: real playlist and segment retrieval, decoding, advancing PTS, nonempty colored RGBA frames and EOS.
- Missing HLS segments surface decoder errors.
- Four simultaneous native synthetic sessions, repeated start/stop/restart and cleanup.
- Mock backend tests: failure isolation, bounded retries, no-frame timeout, restart, source replacement and latest-frame replacement under a slow consumer.
- Packed/padded frame validation, malformed/overflow/truncated buffers and aspect fitting.
- Compact wall layout at 320×240, 480×320 and 800×480.
- Native window visually showed four independent synthetic feeds, moving ball, focus enlargement, return to grid, stop-all clearing frames and application shutdown. Visual testing also identified and corrected missing default fonts.

The HLS fixture validates decoder behavior over local HTTP; it is not a test of public HTTPS certificates, broadcaster authorization or real live-network recovery. No public broadcaster is claimed operational. Linux native CI is configured but its result must be read from the associated workflow. Raspberry Pi, Rust-Boy runtime embedding, long-duration resource testing and production network enforcement remain unvalidated.

Next priority: constrain every GStreamer HLS network request (or introduce a policy-enforcing fetch path), then validate an explicitly authorized live HTTPS publisher end to end and curate approved channels.
