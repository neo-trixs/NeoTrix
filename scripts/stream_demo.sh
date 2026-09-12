#!/usr/bin/env bash
# Streaming download + playback demo
# Usage: ./stream_demo.sh <url> [output_dir]

set -euo pipefail

URL="${1:?Usage: $0 <url> [output_dir]}"
DIR="${2:-/tmp/neotrix-stream}"

mkdir -p "$DIR"

# Extract filename from URL
FILENAME=$(basename "$URL" | sed 's/?.*//')
OUTPUT="$DIR/$FILENAME"

echo "=== NeoTrix Streaming Download + Playback ==="
echo "URL:     $URL"
echo "Output:  $OUTPUT"
echo ""

# Download via Rust binary
echo "[1/2] Starting stream download..."
STREAM_BIN="./target/debug/neotrix-stream-demo"

# Fallback to curl if binary not available
if [ ! -f "$STREAM_BIN" ]; then
    echo "Using curl fallback..."
    # curl -L --compressed -o "$OUTPUT" "$URL" &
    # DL_PID=$!

    # Use Python for streaming download with progress
    python3 -c "
import urllib.request
import sys
import time

url = sys.argv[1]
out = sys.argv[2]

print(f'Downloading: {url}')

req = urllib.request.Request(url, headers={'Accept-Encoding': 'identity'})
resp = urllib.request.urlopen(req)

total = int(resp.headers.get('Content-Length', 0))
downloaded = 0
start = time.time()
chunk_size = 256 * 1024

with open(out, 'wb') as f:
    while True:
        chunk = resp.read(chunk_size)
        if not chunk:
            break
        f.write(chunk)
        downloaded += len(chunk)
        elapsed = time.time() - start
        speed = downloaded / elapsed if elapsed > 0 else 0
        pct = (downloaded / total * 100) if total else 0
        sys.stdout.write(f'\r  {downloaded/(1024*1024):.1f} MB / {total/(1024*1024):.1f} MB ({pct:.0f}%) | {speed/(1024*1024):.1f} MiB/s')
        sys.stdout.flush()

print(f'\nDone: {downloaded/(1024*1024):.1f} MB in {elapsed:.1f}s')
" "$URL" "$OUTPUT" &
    DL_PID=$!

    # Monitor file growth
    echo "[2/2] Waiting for buffer (1MB)..."
    while [ ! -f "$OUTPUT" ] || [ $(stat -f%z "$OUTPUT" 2>/dev/null || echo 0) -lt 1048576 ]; do
        sleep 0.5
    done

    echo "Buffer ready. Starting playback..."
    if command -v ffplay &>/dev/null; then
        ffplay -autoexit -loop 0 "$OUTPUT" &
        PLAYER_PID=$!
    elif command -v mpv &>/dev/null; then
        mpv --loop=inf --keep-open=yes "$OUTPUT" &
        PLAYER_PID=$!
    else
        echo "No player found. Install ffplay: brew install ffmpeg"
        PLAYER_PID=""
    fi

    # Wait for download to finish
    wait $DL_PID 2>/dev/null || true
    echo "Download complete."

    # Kill player
    [ -n "$PLAYER_PID" ] && kill $PLAYER_PID 2>/dev/null || true
else
    "$STREAM_BIN" "$URL" "$OUTPUT"
fi
