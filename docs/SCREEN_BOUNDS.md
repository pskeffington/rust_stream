# Rust-Boy screen geometry contract

Verified from pip_boy/crates/pipboy/assets/micro.css:
- Outer micro device: at most 900 × 520 CSS pixels, with a smaller viewport inside.
- Actual panel is responsive and may be much smaller than the outer device.
- The header/navigation consume vertical space; do not assume the full device area belongs to video.
- The surrounding shell must not acquire horizontal or vertical scrolling just to display a media grid.

Rust Stream sizing rules:
1. Query allocated width/height every frame; use the remaining central-panel region, not fixed 16:9 assumptions.
2. Quad tiles are 2 × 2 and fit in the remaining rectangle; focused mode uses one tile.
3. Controls collapse to short labels for narrow widths. Free TV links remain inside a dropdown, not a growing toolbar.
4. Feed label and status overlays shrink or disappear when tiles are small; status is never represented as LIVE without decoded frames.
5. Focus return remains a single visible action. Never expose a full settings form in the video viewport.
6. No independent scrolling inside the four-screen viewer. Split UI settings and media display.
7. Target validation widths/heights: 320 × 240, 480 × 320, 640 × 360, 800 × 480, and the 900 × 520 shell outer bound. Measure the **inner** allocated media rectangle, too.
8. If available size cannot accommodate controls plus four usable tiles, degrade to a focused source or explicit insufficient-space message rather than overflowing.

This pass corrects the demo's eframe App entrypoint and makes viewport layout bounded. Actual screenshot validation, Rust compilation and embedded micro integration remain pending.
