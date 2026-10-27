# Rust Stream Literature Matrix (2026-10-26)

| Area | Authoritative reference | Engineering decision | Gate |
|---|---|---|---|
| RTSP | RFC 7826 https://www.rfc-editor.org/rfc/rfc7826 | Vetted client; timed connection and transport negotiation | P1 |
| HLS | RFC 8216 https://www.rfc-editor.org/rfc/rfc8216 | Treat playlists and segment fetches as separate security-sensitive requests | P2 |
| Camera interoperability | ONVIF https://www.onvif.org/profiles/ | Explicit opt-in for registered local camera discovery | P3 |
| Media engine | GStreamer Rust https://gstreamer.freedesktop.org/documentation/rust/ | Native decoding under Rust control; no external player | P1 |
| Decode handoff | GStreamer appsink https://gstreamer.freedesktop.org/documentation/app/appsink.html | Latest-frame queue, drop stale frames, bounded memory | P1 |
| RTSP controls | GStreamer rtspsrc https://gstreamer.freedesktop.org/documentation/rtsp/rtspsrc.html | Tuned latency, transport and timeout; independent reconnection | P1 |
| UI | egui/eframe https://docs.rs/egui/ and https://docs.rs/eframe/ | Single-screen quad and focus, frame textures, no scroll | P0/P2 |
| Alternative decoder | FFmpeg https://ffmpeg.org/documentation.html | Compare capability and redistribution licenses, not selected for v1 | Decision |
| Network security | OWASP SSRF https://cheatsheetseries.owasp.org/cheatsheets/Server_Side_Request_Forgery_Prevention_Cheat_Sheet.html | Allowlist destinations, validate resolved peer IP, redirects, HLS fragments | Security |
| Supply chain | RustSec https://rustsec.org/ | Advisory scanning, pinned lockfile and audited plugin list | Security |
| Raspberry Pi | https://www.raspberrypi.com/documentation/computers/camera_software.html | Verify hardware-specific decode, thermal and memory behavior | P4 |

## Review conclusions

GStreamer is the recommended first media backend, behind an optional crate. Source URLs are untrusted, even when curated. The initial core source validation is metadata-only and cannot prevent DNS rebinding or unsafe redirects; network adapters must enforce those rules when opening each connection and any follow-on requests. Embedded URL credentials are prohibited. Validate native GStreamer version and security advisories before deploying.

No DOT feeds or world-city IP streams are asserted live. They require validated direct endpoints, usage rights, operator provenance, expiry, and uptime testing. Raspberry Pi support and four-channel decode capacity remain hardware test requirements. This repo is MIT; native libraries and codec redistribution may carry separate licensing conditions.
