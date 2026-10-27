#!/bin/sh
# Launch the native UI from THIS checkout, not a previously installed app.
set -eu
cd "$(dirname "$0")/.."
printf 'Checkout: %s\n' "$(pwd)"
printf 'Revision: %s\n' "$(git rev-parse --short HEAD)"
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Local modifications detected; preserving them." >&2
fi
if command -v git >/dev/null 2>&1; then
  git fetch --quiet origin main || echo "Fetch unavailable; building existing checkout." >&2
  if git merge-base --is-ancestor HEAD origin/main && [ "$(git rev-parse HEAD)" != "$(git rev-parse origin/main)" ]; then
    echo "Local checkout is behind origin/main; update using git pull --ff-only if safe." >&2
    exit 1
  fi
fi
cargo build -p rust-stream-demo --features native-video
echo "Opening the freshly built rust-stream-demo (close any old Rust Stream windows first)."
exec ./target/debug/rust-stream-demo
