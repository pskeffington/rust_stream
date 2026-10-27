# Open-source native player lifecycle

Rust-Boy and rust_stream remain free MIT-licensed code. The Free TV panel defaults to Pluto TV, but does not claim Pluto video playback without an approved/compatible playback interface. Being open source does not grant rights to third-party stream URLs, DRM keys or commercial playback APIs.

Implemented core:
- FreeTvService names and official URLs.
- FreeTvPlayer selection, configuration, status, stop and generation token.
- Selecting a new provider clears the old source and invalidates old-session updates.
- A caller may configure an explicitly allowed Source; the native media backend is still not implemented.
- Unit tests for defaults and switching.

Integration contract:
1. Pause and tear down the previous decoder when service switches or Media tab closes.
2. Load only an approved per-service Source from settings, never derive manifests from provider webpages.
3. Discard any delayed frame/status with a stale generation value.
4. If not configured, show the service name, OFFLINE, and a separate link to the official website.
5. Implement direct playback using reviewed native media components and authorization where applicable.
6. Keep all UI within the Rust-Boy handheld's allocated viewport.

Next gate: implement real MediaSession lifecycle wiring, GStreamer decode and texture updates, after obtaining permitted test HLS sources. No scraping, DRM bypass or hidden third-party API reliance.
