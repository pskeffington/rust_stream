# Boston and international live television source register

Reviewed: 2026-10-26. These are **live video only** sources. Publisher pages are evidence/discovery links, NOT directly playable native endpoints.

| Station | Publisher live page | Direct manifest | Native status |
|---|---|---|---|
| Al Jazeera English | https://www.aljazeera.com/video/live/ | https://live-hls-apps-aje-fa.getaj.net/AJE/index.m3u8 (third-party IPTV listing; not independently decoded) | Unverified HLS candidate |
| Boston 25 News / WFXT | https://www.boston25news.com/video/ | None confirmed | Official player only; research pending |
| WCVB NewsCenter 5 | https://www.wcvb.com/article/stream-live-local-news-plus-weather-original-series-more/63854871 | None confirmed | Official Very Local streaming; research pending |
| WCVB Very Local | https://www.wcvb.com/very-local | None confirmed | App/platform source; research pending |

Evidence:
- Al Jazeera official live programming: https://www.aljazeera.com/video/live/
- Al Jazeera stream URL revision history, January 2026: https://gitea.jpt.land/jpt/IPTV/commits/commit/0221fcbe3fa26428f9394d013af2100fd47f21cc
- Boston 25 official live page exposes Boston 25 Now, 24/7 News and Weather: https://www.boston25news.com/video/
- WCVB confirms live newscasts through Very Local and WCVB.com: https://www.wcvb.com/article/wcvb-launch-newscenter-5-at-1230-beginning-monday-january-5/69887424

No YouTube, RSS, news articles, browser iframe, or publisher page is eligible for the decoder.
No unconfirmed source is labelled LIVE or automatically enabled.

## Native playback approval gate

1. Confirm the publisher's rights and direct stream accessibility; do not bypass DRM or regional access controls.
2. Establish true direct HLS/RTSP endpoint and codec compatibility.
3. Fetch metadata and segments with source allowlisting, DNS/peer binding, TLS and redirect controls.
4. Decode fresh moving frames continuously; record status TTL and disconnect behavior.
5. Add to configured NEWS tiles only after the above pass.

WFXT and WCVB should remain discovery-only until their direct media endpoints and permissions are established.
