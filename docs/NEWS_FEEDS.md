# Rust-Boy News Feed Architecture

First pass: direct HTTPS RSS 2.0 and Atom source ingestion, independent of YouTube and any streaming-video decoder.

Initial candidate feed endpoints (disabled by default until operator enables):
- BBC Top: https://feeds.bbci.co.uk/news/rss.xml
- NPR: https://feeds.npr.org/1001/rss.xml
- Guardian World: https://www.theguardian.com/world/rss

Source review (2026-10-26): https://www.wprssaggregator.com/news-rss-feeds/ ; Atom RFC https://www.rfc-editor.org/rfc/rfc4287/ . Endpoints are documented candidates; live HTTP response retrieval was not validated from the coding environment. RSS feeds supply headlines and article links, NOT playable video. AP Media API https://api.ap.org/media/v/swagger/ requires entitlements.

## Implementation
- rust-stream-core::parse_news parses bounded XML and returns headline metadata with HTTPS article URLs.
- Never insert raw XML/description HTML into the UI. Render as text.
- No automated fetches, redirects, automatic embeds, media decoding, or background polling in this pass.
- Runtime adapter must use host allowlists, TLS, redirect and resolved-peer protections, bounded response bytes, connection timeouts, rate limits, ETag/Last-Modified, and a refresh interval with jitter.
- Security: prohibit DTD external entity resolution and remote XML references, which roxmltree does not fetch. Treat titles and summaries as untrusted display text. Guard against unsafe HTML or link spoofing.
- Article URLs are navigation destinations only. Never interpret article pages as stream URLs.
- Verify publisher reuse/attribution terms before redistribution or production aggregation.

## Planned user experience
MEDIA > NEWS: compact four-up source tiles or combined headlines with time/source; tap item to read its link and switch to a feed-specific panel. SECURITY and TRAFFIC remain separate collections. No scrolling in the default handheld media wall; page or focus views can provide headlines. Offline cache is labelled cached and timestamped.

## Live television track
A news broadcaster may offer licensed HLS or other direct-IP streaming, but that is a **separate** inventory from RSS. Require an official public stream manifest/authorization plus a native decoder and frame validation. Never scrape video URLs from embedded YouTube players.
