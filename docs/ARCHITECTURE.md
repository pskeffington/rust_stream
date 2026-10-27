# Rust Stream architecture and Rust-Boy integration

Standalone crates:
- rust-stream-core: typed sources, policy metadata, quad slots, bounded frames, backend/session traits.
- rust-stream-demo: offline 2x2 interface and click-to-focus preview (no media or network).
- planned rust-stream-gst: GStreamer workers, appsink and frame delivery.
- planned rust-stream-catalog: authorized public feeds and health checks.
- planned rust-stream-rustboy: embedding adapter.

Approved manifest -> operator selection -> verified network connector -> per-feed decoder -> bounded latest frame -> texture -> quad UI.

Modes: quad view, focused one-tile view, single TV/IP stream, single multimedia stream, approved vault camera and curated public/city/DOT feeds. Only offline quad/focus shell exists now.

Rust-Boy owns role authorization, location settings, protected secrets and global source configuration. The stream module renders within its existing UI. No separate media application, arbitrary LAN scan or default network camera access.

Resource controls: max one queued frame per tile, 1920x1080 RGBA input cap, individual reconnection and cancellation, bandwidth/CPU budgets. Evaluate 4x640x360 at 10-15fps as an initial test target, not a validated hardware promise.

Security: bind DNS resolution to allowed connect peers, validate every redirect and HLS playlist/segment request, block arbitrary local/metadata addresses, prevent cross-origin credential forwarding, redact query strings in logs, protect camera passwords outside manifests, and require approval/revocation for local vault feeds.
