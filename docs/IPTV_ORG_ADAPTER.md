# IPTV-org direct HLS discovery adapter

Source: https://github.com/iptv-org/api (API project is Unlicense; channel media rights are separate).

## Inputs

- https://iptv-org.github.io/api/channels.json
- https://iptv-org.github.io/api/streams.json

The `rust_stream_core::import_iptv_org` function accepts downloaded JSON snapshots and an operator-curated `SourcePolicy`. It performs NO outbound requests. A host must be explicitly allowlisted; all discoveries remain disabled. Recommended default countries: `US`; operators may change this in Global Settings.

## Acceptance rules

- Match each stream to a known channel ID.
- Exclude NSFW and closed channels; exclude streams with browser header overrides, geo-blocked and intermittent flags.
- Accept only direct HTTPS paths ending in `.m3u8`, without embedded credentials or URL fragments.
- Reject private/special purpose IPs, unknown hosts, and duplicate channel/URL pairs.
- Attach channel category and country for News/Free TV filtering.
- Do not treat a fetched URL or response 200 as proof of healthy live playback.

## Remaining integration gates

1. Add a bounded updater to fetch and cache snapshots with size limits, provenance, expiry and schema error handling; never blindly fetch arbitrary URLs.
2. Apply channel-level broadcaster rights/terms review and operator approval; catalog publication alone does not grant retransmission rights.
3. Perform connection-time DNS validation, TLS and redirect enforcement, HLS child URL validation and actual decoded frame confirmation.
4. Promote eligible approved sources through the existing source registry; preserve existing vetted feeds if the remote catalog fails.
5. Use the native GStreamer backend, a single-page Free TV/News interface and status TTL; no web pages or YouTube embeds.
6. Add offline fixtures, resilient updater tests, macOS and Raspberry Pi decode tests.

The API is discovery metadata, not a functioning video player and not an authorization service.
