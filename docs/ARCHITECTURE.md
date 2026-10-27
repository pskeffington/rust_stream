# Architecture and Rust-Boy integration

- `rust-stream-core`: typed source definitions and catalog policy, quad/focus state, bounded RGBA frames, backend/session traits and reusable `Playback` workers. No egui dependency. Optional `native-video` adds GStreamer.
- `rust-stream-demo`: eframe 0.37 native wall with one independent player and texture per slot, source controls, focus/grid and offline status. Default builds require no media libraries. The desktop UI starts in NEWS, offers a direct named HTTPS HLS editor, and stores four user-local custom presets; category changes currently reassign those four active slots.
- `rust-stream-catalog`: candidate discovery/import tooling. Discovery and metadata validation do not prove decoding or authorization.

Operator-approved configuration → source-bound backend → per-slot worker → replaceable RGBA mailbox → egui texture → wall/focus surface.

`Playback` owns decoder lifetime, generation-based cancellation, retry budget and frame delivery. Decoder calls and cleanup never run in an egui update. A single worker per player coalesces rapid source changes; old generations cannot publish stale frames. Drop joins workers. Backend connection/teardown timeouts remain necessary because Rust cannot kill blocked native calls.

The integration boundary is the core API, not the desktop executable. Rust-Boy can own four `Playback` values, start a configured `MediaBackend`, poll the latest frame into its own renderer and stop/drop sessions on navigation or source revocation. Source authorization, protected credentials and global configuration belong to the host. The current egui adapter is in the demo; extraction into a reusable egui component and a real Rust-Boy integration test are still pending.

The current native output target is four 640×360 RGBA feeds at 15 fps. One appsink sample and one worker mailbox frame are retained per player, plus current rendering/texture storage. These limits do not bound all compressed network and decoder allocations. Pi CPU/memory/hardware decode budgets require device measurement.

The authoritative native-backend limitations and test record are in [NATIVE_VIDEO](NATIVE_VIDEO.md). Connection-time DNS, redirects and nested HLS request restrictions remain the next security milestone. Public and organizational camera policy must remain separate; no LAN discovery or internal camera access is enabled.
