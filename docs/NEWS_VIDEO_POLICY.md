# Live video only — Rust-Boy News media policy

News in rust_stream means **live moving-picture video**, not headlines, articles, RSS, Atom, news summaries, podcasts, or text feeds.

Permitted transport targets: authorized direct HLS, RTSP/RTSPS, or another vetted native video transport. The approved source must yield decoded moving frames and a stream-health signal. A webpage, YouTube embed, video preview image, static JPEG, or RSS enclosure is not itself a qualifying news source.

No video source is approved merely because an endpoint has been discovered. Require publisher identity, authorization/terms, direct playable media URL, supported codec/container, destination controls, TLS/redirect/segment validation and actual decoded-frame verification.

Operational UI: MEDIA > NEWS displays four video tiles; tap to focus; show explicit OFFLINE / NOT CONFIGURED / STALLED. Feed setup and credentials remain in universal Settings. Do not launch external apps.

This policy applies to rust_stream. Other Rust-Boy modules may independently own text-news functions; they are not imported here.

## Next gates

- [ ] Identify direct licensed broadcaster video streams not dependent on YouTube.
- [ ] Native HLS/RTSP decoder backend and per-feed bounded frame queue.
- [ ] Connection-time destination and redirect protection, HLS segment checks.
- [ ] Prove continuous decoded moving frames, not merely HTTP 200.
- [ ] Four-feed native integration with resource constraints on Mac/Pi.
