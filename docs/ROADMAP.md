# Development roadmap

| Gate | State | Evidence / remaining work |
| --- | --- | --- |
| G1 Native decoder compiles | Verified on macOS | Feature check, synthetic/HLS smoke tests; Ubuntu native CI configured |
| G2 Video rendered in egui | Verified for synthetic feeds | Four visible native patterns; owned HTTP HLS fixture decoded separately; authorized live HTTPS publisher still pending |
| G3 Start/stop/switch/cleanup | Verified with fixtures | Generation cancellation, latest-frame mailbox, repeated lifecycle tests, visual stop/shutdown; arbitrary native-call hangs remain a framework limitation |
| G4 Four independent players | Verified with synthetic feeds | Four real decoder workers, per-slot source/restart/removal, failure-isolation tests, focus/grid |
| G5 Approved feed catalog | Pending | Enforce connection-time and nested HLS request policy, then establish authorization and end-to-end publisher evidence |
| G6 Automatic recovery | Partial | Bounded retries and stalled/no-frame tests pass; live network interruption/recovery tests pending |
| G7 Compact UI | Partial | Geometry tests at 320×240/480×320/800×480; touch sizing and settings layout review pending |
| G8 Rust-Boy embedding | Pending | Core playback is reusable; reusable egui adapter and host integration test needed |
| G9 Raspberry Pi | Pending | Real-device decoding and CPU/memory/hardware measurements needed |
| G10 Release candidate | Pending | Security, integration, platform and long-duration reliability gates |

Next development pass: constrain the complete HLS request graph, including resolved IPs, redirects, variants, keys and segments. Do not broaden arbitrary URL or internal-camera access ahead of that work. Then validate an explicitly authorized live HTTPS publisher and populate category-specific channel catalogs.

Protocol order: HLS/HTTPS, authorized RTSP/RTSPS, MJPEG/HTTPS, snapshots. Audio, persistent channel selection, commercial provider integrations, configurable decode limits and memory-pressure metrics remain planned.

See [native validation record](NATIVE_VIDEO.md#validation-record), [source catalog](DIRECT_FEED_CATALOG.md) and [news policy](NEWS_VIDEO_POLICY.md). No candidate is operational merely because a manifest or URL validates.
