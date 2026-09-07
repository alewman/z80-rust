#!/usr/bin/env bash
# Rung 4: raxoft/z80test 1.2a natively (z80full, z80ccf, z80memptr).
# Usage: scripts/rung4.sh [directory holding the .tap files]
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
TAPS=${1:-"$ROOT/external/z80test"}
cargo build --release -q --manifest-path "$ROOT/Cargo.toml"
exec "$ROOT/target/release/z80-z80test" "$TAPS/z80full.tap" "$TAPS/z80ccf.tap" "$TAPS/z80memptr.tap"
