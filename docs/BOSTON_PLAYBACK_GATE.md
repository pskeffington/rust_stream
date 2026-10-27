# Boston HLS qualification — October 27, 2026

## Operator Mac observations

At 2026-10-27T05:46:38Z the operator ran `scripts/probe_new_england_hls.py` against three Boston candidates. WCVB (`ma-wcvb`) returned HTTP 200 and an HLS master manifest, CBS News Boston (`ma-cbsboston`) returned HTTP 200 and an HLS master manifest, and Boston 25 NOW (`ma-wfxt`) returned HTTP 204 without a playlist. Evidence is an operator-supplied command transcript, not an independent second probe. These are point-in-time results, not proof of availability, segment decoding, source rights or permission to redistribute. Auto-connect remains disabled for all entries.

## Native playback gate

Use `cargo run -p rust-stream-demo --features native-video` with `RUST_STREAM_HLS_URL_1` and `RUST_STREAM_HLS_URL_2` set to the approved exact URLs from `data/new_england_sources.json`. In SOURCES select Configured HLS 1 and Configured HLS 2, then Apply / Restart. Do not set `RUST_STREAM_TEST_PATTERN=1` in this test. Do not publish endpoint URLs in logs or screenshots.

Record: current UTC timestamp; manifest response; decoded video frame count increasing across at least 30 seconds; whether both video panes render moving content; recovery after transient network loss; stop/clear behavior; and permitted use/territory. A reachable master playlist is not sufficient. If decoding fails, record the specific GStreamer error privately without disclosing signed URLs or credentials. Do not promote the source to default until rights and nested HLS network security are addressed.

## Next implementation

Add a native feature-gated command-line decoder smoke test that uses the existing Rust `GstBackend` and `Playback` APIs, verifies advancing frame PTS in a bounded interval, emits sanitized results, and never runs automatically against third-party sources in CI. Retire the Boston 25 candidate only after checking whether the HTTP 204 is persistent or a transient geographic/platform condition.
