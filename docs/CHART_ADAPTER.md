# Maryland CHART adapter — ingest gate

Verified public documentation: https://chart.maryland.gov/DataFeeds/GetDataFeeds ; public catalog URL https://chart.maryland.gov/DataFeeds/GetCamerasJson (linked in Maryland GIS metadata https://www.arcgis.com/sharing/rest/content/items/21d54438d8f8409d8bc6db8207cfaace/info/metadata/metadata.xml?format=default&output=html).

CHART documents id, description, lat/lon, opStatus, publicVideoURL. The Rust parser recognizes a JSON array or common wrapper keys and yields untrusted candidate camera records. This is deliberately transport-free; the actual JSON response schema and individual media URLs require network validation.

Workflow:

1. Explicit periodic GET of official HTTPS catalog, with size 8 MiB cap, timeout and no credential forwarding.
2. Parse records with parse_chart_cameras; only opStatus OK cameras qualify for probing, but this is NOT proof of a live picture.
3. Require registration of camera endpoint hosts in provider policy. Do not open arbitrary returned URLs.
4. Pre-resolve and validate addresses; pin socket destination/verify peer; block unexpected redirects and internal address space; re-evaluate HLS segments and reconnections.
5. Probe media using a native decoder with bounded data/time. Only mark HEALTHY after successfully decoded fresh frames.
6. Rate-limit probes, record health TTL, and select geographically nearest HEALTHY camera.
7. Drop or mark stale on loss, and keep cached source metadata separate from credentials.

Important: CHART public cameras are for observation of current conditions. Do not create archival footage, bulk redistribution or recording without permission.

TODO: connect HTTP catalog fetch, verify actual payload structure in integration tests, implement GStreamer adapter, feed-specific policies and metrics.
