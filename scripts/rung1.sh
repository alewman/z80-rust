#!/usr/bin/env bash
# Rung 1: the example manifests diff clean against the reference.
# Usage: scripts/rung1.sh [z80-python checkout] [python]
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
Z80PY=${1:-"$ROOT/external/z80-python"}
PYTHON=${2:-"$Z80PY/.venv/bin/python"}
cargo build --release -q --manifest-path "$ROOT/Cargo.toml"
status=0
for name in flags-and-branches interrupts prefix-sequences; do
    manifest="$Z80PY/examples/conformance/$name.json"
    "$ROOT/target/release/z80-trace" "$manifest" \
        | "$PYTHON" -m z80_python.conformance diff "$manifest" - || status=1
done
exit $status
