#!/usr/bin/env bash
# Rung 3: ZEXALL (and ZEXDOC) diffed in lockstep against the reference.
#
# Place zexall.com / zexdoc.com in conformance/zex/ first. PyPy is 5-15x
# faster than CPython for the reference side.
#
# The reference side runs at a few tens of thousands of records per second
# and ZEXALL is billions of records, so by default the run is split into
# segments: z80-trace writes a checkpoint manifest every SEGMENT records
# (full memory image plus every CpuState field), and JOBS segments are diffed
# in parallel. Every segment starts from the state the previous one ended
# in, so the concatenation of the segment diffs is the diff of the whole run
# and a divergence anywhere is reported by the segment holding it.
# JOBS=1 SEGMENT=0 runs the single unsegmented pipe instead.
#
# Usage: scripts/rung3.sh [zexall|zexdoc] [python]
#   env: JOBS (default: nproc), SEGMENT (records per segment, default 50000000)
set -euo pipefail
ROOT=$(cd "$(dirname "$0")/.." && pwd)
PROGRAM=${1:-zexall}
PYTHON=${2:-"$ROOT/external/z80-python/.venv/bin/python"}
JOBS=${JOBS:-$(nproc)}
SEGMENT=${SEGMENT:-50000000}
cargo build --release -q --manifest-path "$ROOT/Cargo.toml"
TRACE="$ROOT/target/release/z80-trace"
manifest="$ROOT/conformance/zex/$PROGRAM.json"

if [ "$SEGMENT" = 0 ]; then
    exec "$TRACE" "$manifest" | "$PYTHON" -m z80_python.conformance diff "$manifest" -
fi

DIR="$ROOT/external/checkpoints/$PROGRAM"
rm -rf "$DIR"
mkdir -p "$DIR"
echo "writing checkpoints every $SEGMENT records into $DIR"
"$TRACE" "$manifest" --no-trace --checkpoint-every "$SEGMENT" --checkpoint-dir "$DIR" 2>&1 \
    | tee "$DIR/run.log"

echo "diffing $(ls "$DIR"/*.json | wc -l) segments, $JOBS at a time"
export TRACE PYTHON
ls "$DIR"/*.json | sort | xargs -P "$JOBS" -I{} sh -c \
    '"$TRACE" "{}" 2>/dev/null | "$PYTHON" -m z80_python.conformance diff "{}" - > "{}.result" 2>&1; echo $? > "{}.status"'

status=0
for result in "$DIR"/*.json.result; do
    cat "$result"
    [ "$(cat "${result%.result}.status")" = 0 ] || status=1
done
if [ $status = 0 ]; then
    echo "$PROGRAM: every segment identical"
else
    echo "$PROGRAM: DIVERGENCE (see above)"
fi
exit $status
