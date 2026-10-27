# New England live-video feed inventory — 2026-10-27

Registry: [`data/new_england_sources.json`](../data/new_england_sources.json). This is **discovery evidence**, not a tested or playback-approved catalog. Six-state regional coverage: MA, CT, RI, NH, VT, ME. Sources are video only; no RSS or text news. Direct feeds are distinct from publishers' watch pages and from still-image camera portals.

## Source classifications

- `direct_hls_candidate`: a third-party index publishes a concrete HTTPS .m3u8 URL. Neither the URL's availability nor publisher permission has been confirmed.
- `publisher_live_page`: an official station/service documents video availability, but no independently verified native HLS endpoint is known. Never hand the webpage URL to GStreamer as a stream.
- `camera_directory_not_confirmed_video`: an official agency lists cameras; their media type may be snapshots rather than actual continuous video. Do not count them as verified native streams.

The first three native endpoint candidates are: Boston 25 NOW (WFXT, Amagi), WCVB (Tubi-hosted playlist), and CBS News Boston (CBS CDN). Their *third-party discovery evidence* is recorded per entry. Official broadcasters confirmed the presence of live viewing on their own services for CBS News Boston, NBC10 Boston, WHDH, WCVB (Very Local), WMUR, WMTW, WCAX and WFSB. WPRI is a station lead, not proof that its published home page exposes a stream. ME/NH/VT 511 is a public camera discovery lead; it is not evidence of HLS video.

## Validation record

| Gate | Result (2026-10-27) | Interpretation |
| --- | --- | --- |
| Registry sources and six-state market coverage | 13 entries | Discovery documented; duplicate broadcaster records intentionally distinguish direct candidate from publisher page |
| Direct HLS URLs identified | 3 | Playlist claims from third-party directories |
| Official service web presence | Documented for most broadcast leads | Does **not** validate native playback |
| Container DNS probe: CBS Boston, WFXT, WCVB | **Inconclusive**: environment could not resolve any of the three hostnames | Not evidence that broadcaster is offline |
| HTTP 200 + HLS `#EXTM3U` | 0 verified | Not run successfully |
| Master variant/media segment decode | 0 verified | Not run |
| Advancing video frames in native app | 0 verified | Not run |
| Publisher permission for third-party clients/redistribution | 0 reviewed | Do not auto-enable |
| Publicly accessible traffic **live video** | 0 verified | Still-image cameras do not satisfy native video requirement |

## Operator-side manual probe (trusted, reviewed endpoints only)

Run on a network where you have permission to test each publisher. Do **not** bulk-replay discovered, unknown, redirected or arbitrary URLs. Example for the candidate already listed in the registry:

```sh
# Inspect TLS, HTTP status and response type, without following redirects:
curl --proto '=https' --max-redirs 0 --connect-timeout 5 --max-time 12 \
  --dump-header /tmp/rust-stream-hls-headers.txt \
  --output /tmp/rust-stream-hls-body.m3u8 \
  'https://cbsn-bos.cbsnstream.cbsnews.com/out/v1/589d66ec6eb8434c96c28de0370d1326/master.m3u8'
head -c 120 /tmp/rust-stream-hls-body.m3u8
```

Treat non-200, HTML, geo-blocking, ad/DRM/token requirements, redirects and non-`#EXTM3U` as **not validated**. A reachable manifest alone is insufficient: a qualified publisher-approved run must resolve every nested segment safely, produce advancing H.264 frames, and survive 15 minutes of playback and deliberate network loss without cross-slot stale frames. Record verification timestamp, HTTP result, manifest kind, segment/decode results, rights/terms, and geographic scope separately. Do not commit signed manifest URLs, cookies, authentication headers or bearer tokens.

## Integration gate

The native player's curated picker currently includes two international candidates behind an explicit ID approval variable. **Do not** append the New England discovery entries directly to an auto-play list. They must pass both authorization review and runtime network enforcement for nested HLS resources. The existing GStreamer backend does not yet secure every redirected/subresource request; it is not appropriate for unattended arbitrary catalog auto-connect. Retain direct HTTPS media only, no YouTube or embedded player URLs.
\n## Optional bounded HLS manifest diagnostics

A deterministic offline test suite and an **opt-in** source probe are available. The script accepts only the exact IDs of direct-HLS candidates already in `data/new_england_sources.json`, sends one bounded HTTPS request per selected ID, denies redirects, limits responses to 256 KiB and records whether an `#EXTM3U` master/media manifest was received. It never fetches variants or segments, decodes frames, approves rights, updates the curated catalog, or enables auto-connect.

```sh
python3 -m unittest discover -s tests -p 'test_new_england_probe.py'
python3 scripts/probe_new_england_hls.py --ids ma-cbsboston --output /tmp/new-england-hls-check.json
```

Use only after reviewing the publisher's conditions and determining the endpoint is appropriate to probe. Exit status 0 means the selected manifests were reachable and recognizable **at that moment**, not that video works or is licensed. Reports omit URLs and sensitive request details. A failure could indicate DNS restrictions, expiration, geo-blocking or HTTP errors; do not equate this with a broadcaster being off air. The script currently uses a fixed source inventory, but its transport is not a hardened network sandbox against DNS remapping; run only in a restricted environment with trusted source hosts. Never use this script as an unattended approval gate.
