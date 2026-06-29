#!/usr/bin/env bash
# =============================================================================
# NeoTrix Animated Flow-Field Particle Overlay Compositor
# Generates a time-varying sinusoidal fluid/flow-field using ffmpeg's geq
# filter and composites it onto an input video via screen blend.
# Pure ffmpeg — zero Python dependencies.
#
# Usage: bash overlay_flow.sh <input.mp4> [output.mp4] [duration]
#
# The overlay is generated on-the-fly in lavfi at 320×180 using per-pixel
# geq expressions with sinusoidal waves whose phase shifts over time (via N
# = frame number), then scaled up to 2560×1440 with bilinear interpolation.
#
# Color palette: dark purple → cyan (#1a0f2e → #00141a range)
# Opacity: ~6–8% (controlled by low geq values, amplified by screen blend)
# Fade in: 1.5 seconds
# =============================================================================
set -euo pipefail

INPUT="${1:?Usage: overlay_flow.sh <input.mp4> [output.mp4] [duration]}"
OUTPUT="${2:-${INPUT%.*}-flow.mp4}"
DURATION="${3:-}"

# Detect duration if not provided
if [ -z "$DURATION" ]; then
  DURATION=$(ffprobe -v error -show_entries format=duration \
    -of csv=p=0 "$INPUT" 2>/dev/null || echo "60")
  DURATION=$(printf "%.0f" "$DURATION" 2>/dev/null || echo 60)
fi

# Sanity: duration must be a positive integer
[ "$DURATION" -gt 0 ] 2>/dev/null || DURATION=60

echo "✦ NeoTrix Flow-Field Overlay Compositor"
echo "  Input:    $INPUT"
echo "  Duration: ${DURATION}s"
echo "  Output:   $OUTPUT"

# Build overlay filterchain
# Stage 1: color source → format=gbrp (planar GBR for geq RGB expressions)
# Stage 2: geq with time-varying sinusoidal waves (N/30 ≈ time in seconds at 30fps)
# Stage 3: bilinear scale to 2560×1440
# Stage 4: fade in over 1.5 s
# Stage 5: screen blend with input video (also converted to gbrp)
OVERLAY_FILTER="
  [1:v]format=gbrp,
    geq=r='12+16*sin(0.5*X/320*2*PI + 0.3*Y/180*2*PI + 0.7*N/30)':
        g='6+8*sin(0.7*X/320*2*PI + 0.5*Y/180*2*PI + 1.1*N/30)':
        b='20+30*sin(0.3*X/320*2*PI + 0.8*Y/180*2*PI + 0.5*N/30)',
    scale=2560:1440:flags=bilinear,
    fade=t=in:st=0:d=1.5[fg];
  [0:v]format=gbrp[bg];
  [bg][fg]blend=screen,format=yuv420p
"

# Re-encode with overlay + copy audio stream
ffmpeg -y \
  -i "$INPUT" \
  -f lavfi -i "color=c=black:s=320x180:r=30" \
  -filter_complex "$OVERLAY_FILTER" \
  -t "$DURATION" \
  -c:v libx264 -crf 20 -pix_fmt yuv420p \
  -c:a copy \
  "$OUTPUT"

echo "  ✓ → $OUTPUT ($(du -h "$OUTPUT" | cut -f1))"
