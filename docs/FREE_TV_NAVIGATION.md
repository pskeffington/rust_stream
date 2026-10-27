# Free TV selection and playback contract

The Free TV page now defaults to Pluto TV with a single bounded viewer rectangle. A service selector changes the selected provider among Pluto TV, Plex, Roku Channel, Tubi and Samsung TV+. The surface remains inside Rust-Boy-style screen dimensions.

**Current status:** Only the service selector and viewer placeholder are implemented. It does not play Pluto TV or any other service inside the app. The OPEN OFFICIAL SITE link remains available as an explicit separate action.

Pluto TV documents supported browsers and devices at https://pluto.tv/where-to-watch and https://cbsi.my.site.com/plutotv/s/article/PTV-Authorized-Pluto-TV-Platforms?language=en_US . These documents do not constitute an embeddable/redistributable native media SDK license. Do not scrape protected manifests or bypass DRM, client checks or publisher restrictions.

## Required for actual native playback

1. Obtain a supported in-app playback integration or direct media authorization from the service.
2. If using a browser-based embedded view, validate WebView compatibility, content-security embedding rules, cookies, DRM, fullscreen, touch, audio and region restrictions on target hardware.
3. If using a direct HLS/DRM playback adapter, use an authorized manifest and compliant media/DRM path.
4. Suspend previous provider when switching; do not leave hidden decoders running.
5. Verify the 320x240, 480x320, 640x360 and 800x480 usable media rectangles with tests.
6. No News RSS, YouTube, or publisher page pretending to be a native stream.
