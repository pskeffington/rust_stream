# Native embedded video decoder (first implementation)

Feature: cargo check -p rust-stream-core --features native-video

The GstBackend implements MediaBackend for HLS with in-process GStreamer uridecodebin, videoconvert and appsink producing bounded 640x360 RGBA frames. No browser, iframe, YouTube or external player process is involved. MediaSession::poll_latest_frame reads nonblocking samples; close stops decoding. Only explicitly authorized public HLS streams belong in this backend.

**Not yet complete:** UI texture upload, audio output, playback controls, live broadcaster compatibility, provider authorization, runtime tests, URL redirect/IP policy, resource pressure testing on Raspberry Pi. GStreamer fetches segments internally; do not pass untrusted or arbitrary URLs. Frame extraction currently rejects padded/strided samples of unexpected length. Do not enable arbitrary local/private URLs; a network-policy enforcement mechanism is required before deploying with user supplied sources.

Mac development requires GStreamer runtime and developer packages. Target Pi requires matching plugins. A free-to-watch commercial service does not necessarily offer a playable native HLS stream. Pluto access remains unavailable until a supported integration is identified.
