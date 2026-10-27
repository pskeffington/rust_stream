# Native video

## Local development

Install Rust 1.95+ for the eframe 0.37 demo. On macOS, `brew install gstreamer` provides the runtime, development libraries and bundled plugins. `pkg-config --modversion gstreamer-1.0 gstreamer-app-1.0 gstreamer-video-1.0` should succeed. The first GStreamer initialization may scan installed plugins and take considerably longer than subsequent runs. Homebrew's scanner can emit Python GI/GTK warnings unrelated to the CPU appsink pipeline; do not treat those warnings alone as a successful or failed playback test.

On Debian/Ubuntu, install `pkg-config libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-libav`.

```sh
RUST_STREAM_TEST_PATTERN=1 cargo run -p rust-stream-demo --features native-video
```

This starts four network-free GStreamer patterns in the native window. Otherwise players start offline. In **SOURCES**, independently select synthetic video or configured HLS 1–4, then **Apply / Restart**. **Remove** stops and clears that slot. **STOP ALL** clears textures and cancels all sessions. Click a video to enlarge it; **GRID** returns to the wall. Category tabs currently share the same operator wall; provider integration and category-specific source catalogs are pending.

For trusted HLS, set `RUST_STREAM_HLS_URL_1` through `_4` before launch. `RUST_STREAM_HLS_URL` remains a fallback for channel 1. No URL or credentials are displayed in the UI. No source is scraped or auto-approved.

## Decoder and worker lifecycle

`GstBackend::trusted_hls(source)` binds a backend to an exact enabled, public HTTPS HLS source, validating its definition. `open` rejects another source. Construction is an explicit assertion of operator trust in the publisher and all child resources; it does not establish authorization or network safety. `GstBackend::test_pattern(index)` creates a network-free backend through the same raw-frame pipeline.

The decoder uses `uridecodebin3`, selects video streams only, links dynamic video pads using current caps or a caps query, and negotiates CPU RGBA through `videoconvert`, `videoscale`, `videorate` and appsink. Queried caps are required when GStreamer exposes a pad before assigning current caps. Accepting native raw-memory formats during decode and negotiating against the CPU consumer avoids the previous macOS VideoToolbox negotiation failure.

Output is fixed to 640×360 square pixels and 15 fps, with a clock-synchronized, one-buffer dropping appsink. Actual sample dimensions, offsets and strides come from `VideoInfo` / `VideoFrameRef`; row padding is removed before creating a bounded `FrameRgba`. Negative strides and invalid dimensions/layouts fail safely. Bus errors and EOS propagate to the worker; bus messages are drained rather than accumulated. Audio output is available for operator-configured native HLS sources, with one active audio owner for the entire wall. The default is panel 1. The AUDIO 1–4 selector transfers audio ownership; the other three streams continue rendering silent video. Audio output depends on the OS-selected GStreamer `autoaudiosink` and on the publisher actually supplying a playable audio track.

Each `Playback` owns one worker and one replaceable frame mailbox. Network open, decoding and pipeline teardown run outside egui. Start/stop/source replacement update a generation and coalesce to one desired source. A retired generation cannot publish into its replacement. Slow consumers receive the latest frame without blocking the producer or accumulating history. Source replacement clears the old UI texture immediately. Hidden slots continue being polled and rendered into their own texture handles.

States are Offline, Loading, Live, Stalled, Reconnecting and Failed. Defaults: stalled after 5 seconds without a frame, reconnect after 15 seconds without a frame, and at most three reconnects (0.5, 1 and 2 seconds delay) per operator start. Successful frames do not reset this lifetime retry budget; an explicit restart resets it. Source HTTP elements receive a 10-second timeout when supported. Nested GStreamer requests and pipeline teardown remain subject to framework behavior. Rust cannot forcibly terminate a blocked native call. Drop signals shutdown and joins the worker; no decoder thread is intentionally detached.

Decoded frames are capped; compressed buffers and hardware decoder allocations are not a complete process memory budget. Resolution/fps configuration, bandwidth budgets, memory-pressure metrics and hardware acceleration tuning remain pending.

## Trust boundary

Definition-time URL checks are not SSRF protection. GStreamer controls DNS, sockets, redirected requests, variant playlists, keys and media segments. This backend cannot enforce destination restrictions on that entire request graph. **Only operator-configured publishers and child resources already trusted and authorized may be used.** No arbitrary URL editor, public feed auto-connect, private camera or credential support is provided. An operator environment variable is a trust decision, not proof of licensing.

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

## Curated live news feed selection (experimental)

The native SOURCES menu now includes the two unverified direct-HLS news candidates already tracked in `live_video.rs` (Al Jazeera English and DW English), in addition to synthetic streams and the four operator HLS slots. These candidates are **disabled unless explicitly approved by ID**. Select a candidate in any slot and use Apply / Restart. Approval uses the comma-separated `RUST_STREAM_APPROVED_FEED_IDS` environment variable. For example, after reviewing the publisher's permission and all HLS child resources:

```sh
RUST_STREAM_APPROVED_FEED_IDS=aljazeera-english-candidate cargo run -p rust-stream-demo --features native-video
```

This example is an opt-in configuration, **not** a claim that the stream is available, safe, licensed for redistribution, or successfully decoded. Candidate IDs and publisher evidence are in `crates/rust-stream-core/src/live_video.rs`. Do not ship auto-approved feeds. Source URLs and child-resource behavior can change. Explicit approval does not resolve DNS rebinding, nested-request or redirect safety, and current GStreamer network handling is not a policy-enforcing transport. An operator should only use these feeds in an environment where the publisher and child requests are trusted.

The next verification gate must test public HTTPS manifest retrieval, variants, segments, advancing frames, stability, authorization and regional restrictions. Record status as unavailable, manifest reachable, decoded, or explicitly authorized independently; none of these states implies the others.

## Native single-owner audio (pending device verification)

The four-slot wall defaults to **AUDIO 1**. Select AUDIO 2, 3, or 4 in the top toolbar to transfer the only audible track without restarting video. All other panel audio buffers are dropped by a GStreamer pad probe using the shared atomic owner. An empty or stalled selected panel results in silence; the application does not automatically select a different source. STOP ALL stops every session. On startup no sound plays until panel 1 has a source with audio. Synthetic video fixtures do not supply audio.

Validate on macOS with two operator-authorized HLS sources containing audio: confirm panel 1 is audible, switch to panel 2 and confirm panel 1 is silent, switch repeatedly while both videos continue moving, stop panel 2 and verify silence rather than fallback, then STOP ALL. Test system output/Bluetooth routing separately. This implementation has **not yet been compiled or device-tested** via the GitHub connector; it is a proposed audio pipeline change awaiting local Cargo validation. `autoaudiosink` availability varies by platform.
