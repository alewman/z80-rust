#!/usr/bin/env bash
# Fetch FUSE 1.6.0's Z80 core test set (z80/tests/tests.in and tests.expected,
# GPL-2.0), the same pinned tarball z80-python's scripts/fetch_fuse_tests.py uses.
set -euo pipefail
URL="https://sourceforge.net/projects/fuse-emulator/files/fuse/1.6.0/fuse-1.6.0.tar.gz/download"
SHA256=3a8fedf2ffe947c571561bac55a59adad4c59338f74e449b7e7a67d9ca047096
DEST=${1:-"$(dirname "$0")/../external/fuse_tests"}
if [ -f "$DEST/tests.in" ] && [ -f "$DEST/tests.expected" ]; then
    echo "FUSE tests already in $DEST"
    exit 0
fi
mkdir -p "$DEST"
archive="$DEST/fuse-1.6.0.tar.gz"
curl -sSL --max-time 300 -o "$archive" "$URL"
echo "$SHA256  $archive" | sha256sum -c - >/dev/null
tar -xzf "$archive" -C "$DEST" --strip-components=3 fuse-1.6.0/z80/tests/tests.in fuse-1.6.0/z80/tests/tests.expected
tar -xzf "$archive" -C "$DEST" --strip-components=1 fuse-1.6.0/COPYING
rm -f "$archive"
echo "Fetched FUSE 1.6.0 z80/tests into $DEST"
