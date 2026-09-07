#!/usr/bin/env bash
# Rung 2: all 1,604 SingleStepTests files, registers + RAM + ports + T-states.
# Usage: scripts/rung2.sh [corpus v1 directory]
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
CORPUS=${1:-"$ROOT/external/z80_test_vectors/v1"}
cargo build --release -q --manifest-path "$ROOT/Cargo.toml"
exec "$ROOT/target/release/z80-vectors" "$CORPUS"
