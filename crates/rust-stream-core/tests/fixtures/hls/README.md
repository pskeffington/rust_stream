# Owned deterministic HLS fixture

Three seconds of FFmpeg's generated `testsrc2` pattern, 160×90 at 15 fps, no audio. Generated for this repository on 2026-10-27 and distributed under the repository MIT license; no publisher content is included. The committed transport-stream segments keep native tests independent of an encoder installation.

Regenerate from the repository root with an FFmpeg build containing libx264:

```sh
ffmpeg -hide_banner -loglevel error -f lavfi -i testsrc2=size=160x90:rate=15 -t 3 -an -c:v libx264 -preset ultrafast -g 15 -pix_fmt yuv420p -hls_time 1 -hls_list_size 0 -hls_segment_filename crates/rust-stream-core/tests/fixtures/hls/segment%02d.ts crates/rust-stream-core/tests/fixtures/hls/index.m3u8
```

The test server serves only embedded fixture bytes on a randomly allocated loopback port. Production HLS URL rules are not relaxed.
