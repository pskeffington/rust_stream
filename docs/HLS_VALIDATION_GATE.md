# HLS direct stream preflight gate

This is a pure URL and playlist reference policy validator, not a live network probe or video decoder.

1. Enforce publisher-approved origin hosts for initial playlist and all child references.
2. A playlist with no variant or segment references is not playable.
3. An HLS URL appearing in an IPTV directory remains unverified until actual media is decoded and permissions checked.
4. Native HTTP connector still needs DNS and peer-IP restrictions, redirect checks and connection-time authorization. Playlist checks alone do not prevent SSRF.
5. Al Jazeera candidate remains unverified; Boston 25 and WCVB direct native URLs are still missing. The Boston 25 Haystack website is not a direct HLS URL.
6. Never classify HTTP 200 or manifest parsing as LIVE. Require fresh decoded frames and a health TTL.
