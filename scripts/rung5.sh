#!/usr/bin/env bash
# Rung 5: the ten interrupt scenarios as manifests, diffed against the reference.
# Usage: scripts/rung5.sh [python]
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
PYTHON=${1:-"$ROOT/external/z80-python/.venv/bin/python"}
cargo build --release -q --manifest-path "$ROOT/Cargo.toml"
status=0
for manifest in "$ROOT"/conformance/interrupts/*.json; do
    "$ROOT/target/release/z80-trace" "$manifest" 2>/dev/null \
        | "$PYTHON" -m z80_python.conformance diff "$manifest" - || status=1
done
exit $status
