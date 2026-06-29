#!/usr/bin/env bash
# =============================================================================
# NeoTrix Dynamic Life Overlay Compositor
# Composites ffmpeg lavfi "life" (Conway's Game of Life) cellular automaton
# overlays onto an existing video. Pure ffmpeg — zero Python dependencies.
#
# Usage: bash overlay_life.sh <input.mp4> [output.mp4] [duration]
#
# The overlay is generated on-the-fly in lavfi, scaled up with nearest-neighbor
# for a pixel-art "digital life" aesthetic, then composited via overlay + alpha.
#
# Two layers for depth:
#   Layer 1 (foreground): larger cells, purple, sparse, 8% opacity
#   Layer 2 (background): smaller cells, cyan, denser, 5% opacity
# =============================================================================
set -euo pipefail

INPUT="${1:?Usage: overlay_life.sh <input.mp4> [output.mp4] [duration]}"
OUTPUT="${2:-${INPUT%.*}-life.mp4}"
DURATION="${3:-""}"

# Detect duration if not provided
if [ -z "$DURATION" ]; then
  DURATION=$(ffprobe -v error -show_entries format=duration \
    -of csv=p=0 "$INPUT" 2>/dev/null || echo "60")
  DURATION=$(printf "%.0f" "$DURATION" 2>/dev/null || echo 60)
fi

# Sanity: duration must be a positive integer
[ "$DURATION" -gt 0 ] 2>/dev/null || DURATION=60

echo "✦ NeoTrix Life Overlay Compositor"
echo "  Input:    $INPUT"
echo "  Duration: ${DURATION}s"
echo "  Output:   $OUTPUT"

# Build overlay filterchain
# Layer 1: large life at 640x360 (~1/4 res), sparse purple cells, long mold trail
# Layer 2: small life at 320x180 (~1/8 res), denser cyan cells, shorter mold
# Both use colorkey=black + colorchannelmixer=aa for transparent overlay
OVERLAY_FILTER="
  [0:v]format=rgba[bg];
  life=size=640x360:rate=30:ratio=0.06:mold=200:life_color=#a855f7:death_color=#000000,
    scale=2560:1440:flags=neighbor,
    colorkey=0x000000:0.02:0.0,
    format=rgba,
    colorchannelmixer=aa=0.08[layer1];
  life=size=320x180:rate=30:ratio=0.12:mold=80:life_color=#00d4ff:death_color=#000000,
    scale=2560:1440:flags=neighbor,
    colorkey=0x000000:0.02:0.0,
    format=rgba,
    colorchannelmixer=aa=0.05[layer2];
  [layer1][layer2]blend=all_mode=screen,format=rgba[merged];
  [bg][merged]overlay=0:0
"

# Re-encode with overlay + copy audio stream
ffmpeg -y \
  -i "$INPUT" \
  -f lavfi -i "life=size=640x360:rate=30:ratio=0.06:mold=200:life_color=#a855f7:death_color=#000000" \
  -f lavfi -i "life=size=320x180:rate=30:ratio=0.12:mold=80:life_color=#00d4ff:death_color=#000000" \
  -filter_complex "$OVERLAY_FILTER" \
  -t "$DURATION" \
  -c:v libx264 -crf 20 -pix_fmt yuv420p \
  -c:a copy \
  "$OUTPUT" 2>/dev/null

echo "  ✓ → $OUTPUT ($(du -h "$OUTPUT" | cut -f1))"
