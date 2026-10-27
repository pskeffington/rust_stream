# Single-page MEDIA navigation

The standalone Rust Stream viewer now exposes five top-level tabs on one screen:

| Tab | Presentation | Source policy |
|---|---|---|
| SECURITY | Four-camera wall; tap one tile to focus, GRID to return | Authorized security cameras |
| NEWS | One full panel with Al Jazeera, DW, Boston 25 and WCVB selectors | Direct live video only; all remain OFFLINE until decoded |
| TRAFFIC | Four-camera wall | Verified official traffic-camera streams |
| FREE TV | Compact single-page list of provider navigation links | External viewing links, not direct IP feeds |
| CUSTOM | Four-camera wall | Explicitly approved user sources |

Design constraints: adapt to the allocated Rust-Boy micro inner viewport, keep source editing behind universal Settings, never show unverified streams as LIVE, do not embed YouTube or introduce RSS/text news. Current standalone UI is a layout shell, not a native video decoder. Switching tabs does not yet load independent camera assignments; integration is a later gate.

The desktop preview uses egui tabs and bounded rectangles. Compilation and on-device screenshots remain required; in very small allocations, wrapped controls may consume more room than desired. Final handheld acceptance requires no clipping or scrolling at the actual 320x240, 480x320, 640x360 and 800x480 viewport budgets.

Next engineering gate: per-tab source assignments and native frame rendering, then Rust-Boy micro integration.
