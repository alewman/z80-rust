#!/usr/bin/env bash
# Fetch the exact SingleStepTests/z80 revision z80-python is certified against.
set -euo pipefail
REVISION=ebe1875d48f374bcfd4b505d8eb8ee751568b5f7
DEST=${1:-"$(dirname "$0")/../external/z80_test_vectors"}
if [ -e "$DEST/.revision" ] && [ "$(cat "$DEST/.revision")" = "$REVISION" ]; then
    echo "SingleStepTests/z80 already at $REVISION in $DEST"
    exit 0
fi
rm -rf "$DEST"
mkdir -p "$DEST"
git -C "$DEST" init -q
git -C "$DEST" fetch -q --depth 1 https://github.com/SingleStepTests/z80.git "$REVISION"
git -C "$DEST" checkout -q --detach FETCH_HEAD
actual=$(git -C "$DEST" rev-parse HEAD)
[ "$actual" = "$REVISION" ] || { echo "revision mismatch: $actual" >&2; exit 1; }
echo "$REVISION" > "$DEST/.revision"
echo "Fetched SingleStepTests/z80 at $REVISION into $DEST"
