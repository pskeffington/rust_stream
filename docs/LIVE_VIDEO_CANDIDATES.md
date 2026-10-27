# Direct non-YouTube news video candidates

As of October 26, 2026, **no news video source has passed runtime decoding and publisher reuse review in Rust Stream**.

Candidate 1: DW English
- HLS: https://dwamdstream102.akamaized.net/hls/live/2015525/dwstream102/index.m3u8
- Public directory evidence: https://database.freetuxtv.net/webStream/index?WebStreamSearchForm%5BType%5D=1
- Independent listing: https://namifm.com/station/dw-english-hd-de
- Status: UNVERIFIED; not enabled, not asserted available or licensed for redistribution.

Do not substitute RSS, text articles, a broadcaster homepage, or YouTube embed pages for a direct video stream.

## Next gate

1. Fetch manifest through approved host-bound network client (TLS and redirect policy).
2. Validate manifest is HLS and parse all child playlist/segment/key references.
3. Enforce DNS, actual connected-peer IP, HLS host and credential policies at every request.
4. Decode fresh moving frames with GStreamer, track monotonic timestamps and dropped frames.
5. Confirm publisher terms before pre-enabling or shipping.
6. Only then transition candidate -> approved / healthy. A playlist URL alone is not proof of live video.

HLS parsing module is an offline preflight aid, not an HTTP client, decoder or complete HLS protocol implementation. Complex attributes and alternate renditions require a mature media implementation.
