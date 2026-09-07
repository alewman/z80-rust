#!/usr/bin/env bash
# Rung 6: FUSE 1.6.0's Z80 core test set, 1,356 emulator-derived cases with
# six strict expected divergences (see z80-python's tests/test_fuse_suite.py).
# Usage: scripts/rung6.sh [directory holding tests.in and tests.expected]
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
DIR=${1:-"$ROOT/external/fuse_tests"}
cargo build --release -q --manifest-path "$ROOT/Cargo.toml"
exec "$ROOT/target/release/z80-fuse" "$DIR"
