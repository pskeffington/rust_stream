# Direct non-YouTube feed sources — evidence and gates

Reviewed: 2026-10-26. Entries describe official discovery providers, not validated live media connections.

| Provider | Official source | Discovery facts | Status |
|---|---|---|---|
| Maryland CHART | https://chart.maryland.gov/DataFeeds/GetDataFeeds | JSON/XML catalog advertises publicVideoURL, opStatus, lat/lon, route and timestamp. | Strong P1 discovery target, individual media not verified |
| 511NY | https://prod-ny.ibi511.com/developers/help | GetCameras API, camera view VideoUrl; developer key and throttling. | Requires authorization/key |
| Caltrans | https://publish.data.ca.gov/dataset/closed-circuit-television | CCTV metadata; QuickMap stills update at different intervals. | Snapshot adapter; verify actual image URLs |
| DDOT | https://maps2.dcgis.dc.gov/dcgis/rest/services/DDOT/TrafficOperations/FeatureServer/layers | CCTV location/operational-status metadata. | Location layer only; no implied media access |
| MassDOT | https://gisstg.massdot.state.ma.us/rh/rest/services/Hosted/MassDOT_Traffic_Cams/FeatureServer | GIS camera metadata candidate. | Needs endpoint and terms validation |

## Catalog ingest requirements

1. Read official feed directory only; no guessed public camera URLs.
2. Keep canonical provider/id/coordinates/road/direction and source attribution.
3. Resolve advertised media URLs only after explicit provider host/network policy review.
4. Probe with bounded timeout, redirect restrictions, TLS validation and size caps. Require actual decodable frame and recent frame freshness; HTTP 200 alone is insufficient.
5. Record last success, content type, transport, frame-age, errors, retry backoff and verified time.
6. Rank nearest **healthy** eligible feed; only then distance. If none passes, display NO VERIFIED FEED.
7. Filter out YouTube and other embedded player pages, signed URLs intended solely for third-party browser apps, and restricted cameras.
8. Never record or relay government camera footage without explicit source permission.
9. Publisher news feeds require separate permission/licensing review; HLS/RTSP URL availability is not redistribution authorization.

## Production gates

- [ ] Extract one valid CHART media URL, verify terms, stream protocol and a decoded frame.
- [ ] Native Rust GStreamer worker for direct RTSP/HLS/MJPEG/still sources.
- [ ] Periodic health worker and run-local TTL cache; no stale LIVE indicator.
- [ ] Official 511NY key secured outside repo.
- [ ] Geography-linked ranking, county/ZIP fallback and feed selection.
- [ ] Source license/attribution registry and revocation path.

No providers above should be shown as a working camera until individually media-validated.
