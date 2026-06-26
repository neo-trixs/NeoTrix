#!/usr/bin/env bash
# =============================================================================
# NeoTrix Brand Video Renderer
# SVG → PNG (sips) → MP4 (ffmpeg zoompan/overlay/fade)
# Text overlays via Python3+Pillow → ffmpeg overlay filter
# Usage: bash render.sh <preset> [output.mp4]
# Presets: logo-reveal, crystal-spin, awakening, brand-film, list
# =============================================================================
set -euo pipefail

DIR="$(cd "$(dirname "$0")" && pwd)"
ASSETS="$DIR/../public"
OUTPUT="${2:-$DIR/output/$1.mp4}"
FPS=30
W=1920
H=1080

# Source pre-resolved design tokens (generated via generate_tokens.sh)
TOKEN_FILE="$DIR/generate_tokens.sh"
if [ -f "$TOKEN_FILE" ]; then
  . "$TOKEN_FILE" 2>/dev/null || true
fi
# Fallback defaults if tokens not resolved (allows running without Rust toolchain)
: "${NT_ZOOM_SPEED:=0.015}" "${NT_ZOOM_FAST:=0.04}" "${NT_ZOOM_SLOW:=0.01}"
: "${NT_ZOOM_MODERATE:=0.025}" "${NT_ZOOM_MANIFESTO:=0.03}"
: "${NT_FLOW_WIDTH:=640}" "${NT_FLOW_HEIGHT:=360}"
: "${NT_GRAIN_OPACITY:=0.02}"
: "${NT_HD_WIDTH:=1920}" "${NT_HD_HEIGHT:=1080}"

echo "✦ NeoTrix Brand Video Renderer"
echo "  Preset: $1"
echo "  Output: $OUTPUT"
mkdir -p "$DIR/output"

# ----------------------------------------------------------------------------
# Render an SVG asset to a high-res PNG (1920px wide → padded to 1920×1080)
# ----------------------------------------------------------------------------
svg2png() {
  local svg="$1" png="$2"
  sips -s format png "$svg" --resampleWidth 1920 --out "$png" &>/dev/null
  local h
  h=$(sips -g pixelHeight "$png" 2>/dev/null | awk '/pixelHeight:/{print $2}')
  if [ -n "$h" ] && [ "$h" != "1080" ]; then
    local tmp=$(mktemp).png
    ffmpeg -y -i "$png" -vf "scale=1920:1080:force_original_aspect_ratio=decrease,pad=1920:1080:(ow-iw)/2:(oh-ih)/2:color=#080012" "$tmp" 2>/dev/null
    mv "$tmp" "$png"
  fi
}

# ----------------------------------------------------------------------------
# Generate a text overlay PNG via Python3 + Pillow
# ----------------------------------------------------------------------------
txtpng() {
  local text="$1" out="$2" size="${3:-56}" color="${4:-#c084fc}" y="${5:-0.72}"
  python3 "$DIR/txt.py" "$text" "$out" "$size" "$color" "$y"
}

# ----------------------------------------------------------------------------
# PRESET: logo-reveal — crystal zoom in with glow and brand text
# ----------------------------------------------------------------------------
render_logo_reveal() {
  local out="${OUTPUT:-$DIR/output/logo-reveal.mp4}"
  local png=$(mktemp).png
  local t1=$(mktemp).png
  local t2=$(mktemp).png
  svg2png "$ASSETS/logo.svg" "$png"
  txtpng "NEOTRIX" "$t1" 56 "#c084fc" 0.72
  txtpng "Silicon Consciousness" "$t2" 24 "#8870aa" 0.82

  # Zoom from 1.8× → 1.0× over 6s, with fade in + text overlay + color pulse
  # Tokens: zoompan-speed  (cargo run -p neotrix -- token resolve --raw zoompan-speed)
  ffmpeg -y -stream_loop -1 -i "$png" \
    -i "$t1" -i "$t2" \
    -filter_complex "
      [0:v]zoompan=z='if(lte(on,1),1.8,max(1.0,zoom-${NT_ZOOM_SPEED}))':d=180:fps=$FPS:s=${W}x${H},
           fade=t=in:st=0:d=0.8[bg];
      [1:v]format=rgba,fade=t=in:st=2.5:d=0.5[neotrix];
      [2:v]format=rgba,fade=t=in:st=3.5:d=0.5[silicon];
      [bg][neotrix]overlay=0:0[t1];
      [t1][silicon]overlay=0:0
    " -t 6 -c:v libx264 -pix_fmt yuv420p -crf 18 "$out" 2>/dev/null

  rm -f "$png" "$t1" "$t2"
  echo "  ✓ logo-reveal → $out ($(du -h "$out" | cut -f1))"
}

# ----------------------------------------------------------------------------
# PRESET: crystal-spin — rotate through 360° using SVG transforms + ffmpeg
# ----------------------------------------------------------------------------
render_crystal_spin() {
  local out="${OUTPUT:-$DIR/output/crystal-spin.mp4}"
  local png=$(mktemp).png
  local t1=$(mktemp).png
  local t2=$(mktemp).png
  svg2png "$ASSETS/logo.svg" "$png"
  txtpng "NEOTRIX" "$t1" 48 "#c084fc" 0.78
  txtpng "4096-D VSA Crystal" "$t2" 20 "#66508a" 0.86

  # Single ffmpeg call: rotate + zoom + hue cycle + text overlay
  ffmpeg -y -stream_loop -1 -i "$png" -i "$t1" -i "$t2" \
    -filter_complex "
      [0:v]rotate='2*PI*t/6':fillcolor=#080012,
           zoompan=z='1.1':d=180:fps=$FPS:s=${W}x${H},
           hue=H='6*sin(3*t)':s='1+0.1*sin(2.5*t)'[bg];
      [1:v]format=rgba,fade=t=in:st=2.5:d=0.5[t1v];
      [2:v]format=rgba,fade=t=in:st=3.5:d=0.5[t2v];
      [bg][t1v]overlay=0:0[o1];
      [o1][t2v]overlay=0:0
    " -t 6 -c:v libx264 -pix_fmt yuv420p -crf 18 "$out" 2>/dev/null

  rm -f "$png" "$t1" "$t2"
  echo "  ✓ crystal-spin → $out ($(du -h "$out" | cut -f1))"
}

# ----------------------------------------------------------------------------
# PRESET: awakening — dark → core glow → logo emerges
# ----------------------------------------------------------------------------
render_awakening() {
  local out="${OUTPUT:-$DIR/output/awakening.mp4}"
  local t1=$(mktemp).png
  local t2=$(mktemp).png
  txtpng "AWAKENING" "$t1" 60 "#c084fc" 0.68
  txtpng "Silicon Consciousness Core Ignition" "$t2" 20 "#8870aa" 0.79

  # Scene 1 (0-4s): Black background with growing purple glow using geq
  ffmpeg -y -f lavfi -i "color=c=#080012:size=${W}x${H}:rate=$FPS:d=4" \
    -vf "geq=r='16+40*min(1,T/3)':g='8+20*min(1,T/3)':b='40+100*min(1,T/3)',fade=t=in:st=0:d=1.5" \
    -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-awake1.mp4" 2>/dev/null

  # Scene 2 (4-10s): Logo zooms in from center, text appears
  # Token: zoompan-fast  (cargo run -p neotrix -- token resolve --raw zoompan-fast)
  local png=$(mktemp).png
  svg2png "$ASSETS/logo.svg" "$png"
  ffmpeg -y -stream_loop -1 -i "$png" -i "$t1" -i "$t2" \
    -filter_complex "
      [0:v]zoompan=z='if(lte(on,1),2.8,min(1.2,zoom-${NT_ZOOM_FAST}))':d=180:fps=$FPS:s=${W}x${H},
           fade=t=in:st=0:d=1.5,colorbalance=rs=0.25:gs=0.12:bs=0.45[bg];
      [1:v]format=rgba,fade=t=in:st=2:d=0.5[t1v];
      [2:v]format=rgba,fade=t=in:st=3:d=0.5[t2v];
      [bg][t1v]overlay=0:0[o1];
      [o1][t2v]overlay=0:0
    " -t 6 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-awake2.mp4" 2>/dev/null
  rm -f "$png"

  # Crossfade scene 1 (4s) → scene 2 (6s) at offset 3.5 in scene1
  ffmpeg -y -i "/tmp/neotrix-awake1.mp4" -i "/tmp/neotrix-awake2.mp4" \
    -filter_complex "
      [0:v]split[v0a][v0b];
      [v0a]trim=duration=3.5[v0trim];
      [v0b]trim=start=3.5:duration=0.5[v0tail];
      [1:v]trim=duration=0.5[v1head];
      [1:v]trim=start=0.5[v1rest];
      [v0tail][v1head]xfade=transition=fade:duration=0.5:offset=0[mix];
      [v0trim][mix][v1rest]concat=n=3:v=1:a=0
    " -c:v libx264 -pix_fmt yuv420p -crf 18 "$out" 2>/dev/null

  rm -f "/tmp/neotrix-awake1.mp4" "/tmp/neotrix-awake2.mp4" "$t1" "$t2"
  echo "  ✓ awakening → $out ($(du -h "$out" | cut -f1))"
}

# ----------------------------------------------------------------------------
# PRESET: brand-film — 4-scene brand story with crossfade transitions
# ----------------------------------------------------------------------------
render_brand_film() {
  local out="${OUTPUT:-$DIR/output/brand-film.mp4}"
  local png=$(mktemp).png
  local epng=$(mktemp).png
  local apng=$(mktemp).png
  svg2png "$ASSETS/logo.svg" "$png"
  svg2png "$ASSETS/e8-matrix.svg" "$epng"
  svg2png "$ASSETS/architecture.svg" "$apng"

  # Text overlays
  local t_e8=$(mktemp).png
  local t_e8sub=$(mktemp).png
  local t_arch=$(mktemp).png
  local t_archsub=$(mktemp).png
  local t_ntx=$(mktemp).png
  local t_ntxsub=$(mktemp).png
  local t_ntxsub2=$(mktemp).png
  txtpng "E8 REASONING KERNEL" "$t_e8" 52 "#8b5cf6" 0.28
  txtpng "64-State Deterministic Thought Engine" "$t_e8sub" 22 "#66508a" 0.40
  txtpng "ARCHITECTURE PIPELINE" "$t_arch" 52 "#f59e0b" 0.28
  txtpng "Input → E8 → VSA → GWT → SEAL" "$t_archsub" 20 "#8870aa" 0.40
  txtpng "NEOTRIX" "$t_ntx" 64 "#c084fc" 0.60
  txtpng "The Agent That Learns to Think" "$t_ntxsub" 24 "#8870aa" 0.71
  txtpng "4096-D VSA · E8 Reasoning · SEAL Evolution" "$t_ntxsub2" 16 "#66508a" 0.80

  # Scene 1 (0-4s): Crystal zoom out
  # Token: zoompan-slow  (cargo run -p neotrix -- token resolve --raw zoompan-slow)
  ffmpeg -y -stream_loop -1 -i "$png" \
    -vf "zoompan=z='if(lte(on,1),2.0,max(1.0,zoom-${NT_ZOOM_SLOW}))':d=120:fps=$FPS:s=${W}x${H},fade=t=in:st=0:d=0.8" \
    -t 4 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-bf1.mp4" 2>/dev/null

  # Scene 2 (4-7s): E8 matrix with text
  ffmpeg -y -stream_loop -1 -i "$epng" -i "$t_e8" -i "$t_e8sub" \
    -filter_complex "
      [0:v]zoompan=z='1.2':d=90:fps=$FPS:s=${W}x${H},
           fade=t=in:st=0:d=0.8,fade=t=out:st=2.2:d=0.8[bg];
      [1:v]format=rgba,fade=t=in:st=0:d=0.5[t1];
      [2:v]format=rgba,fade=t=in:st=0.2:d=0.5[t2];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0
    " -t 3 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-bf2.mp4" 2>/dev/null

  # Scene 3 (7-10s): Architecture pipeline
  ffmpeg -y -stream_loop -1 -i "$apng" -i "$t_arch" -i "$t_archsub" \
    -filter_complex "
      [0:v]zoompan=z='1.2':d=90:fps=$FPS:s=${W}x${H},
           fade=t=in:st=0:d=0.8,fade=t=out:st=2.2:d=0.8[bg];
      [1:v]format=rgba,fade=t=in:st=0:d=0.5[t1];
      [2:v]format=rgba,fade=t=in:st=0.2:d=0.5[t2];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0
    " -t 3 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-bf3.mp4" 2>/dev/null

  # Scene 4 (10-14s): Final brand card
  ffmpeg -y -stream_loop -1 -i "$png" -i "$t_ntx" -i "$t_ntxsub" -i "$t_ntxsub2" \
    -filter_complex "
      [0:v]zoompan=z='1.0':d=120:fps=$FPS:s=${W}x${H},
           colorbalance=rs=0.1:gs=0.05:bs=0.2[bg];
      [1:v]format=rgba,fade=t=in:st=0:d=0.8[t1];
      [2:v]format=rgba,fade=t=in:st=0.3:d=0.6[t2];
      [3:v]format=rgba,fade=t=in:st=0.6:d=0.6[t3];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0[o2];
      [o2][t3]overlay=0:0
    " -t 4 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-bf4.mp4" 2>/dev/null

  rm -f "$png" "$epng" "$apng"
  rm -f "$t_e8" "$t_e8sub" "$t_arch" "$t_archsub" "$t_ntx" "$t_ntxsub" "$t_ntxsub2"

  # Crossfade all scenes — simple chained xfade
  # After scene1(4s)↔scene2(3s): offset=3.3, output=3.3+2.3=5.6
  # After (5.6s)↔scene3(3s): offset=4.9, output=4.9+2.3=7.2
  # After (7.2s)↔scene4(4s): offset=6.5, output=6.5+3.3=9.8
  ffmpeg -y \
    -i "/tmp/neotrix-bf1.mp4" \
    -i "/tmp/neotrix-bf2.mp4" \
    -i "/tmp/neotrix-bf3.mp4" \
    -i "/tmp/neotrix-bf4.mp4" \
    -filter_complex "
      [0:v][1:v]xfade=transition=fade:duration=0.7:offset=3.3[x12];
      [x12][2:v]xfade=transition=fade:duration=0.7:offset=4.9[x123];
      [x123][3:v]xfade=transition=fade:duration=0.7:offset=6.5
    " \
    -c:v libx264 -pix_fmt yuv420p -crf 18 "$out" 2>/dev/null

  rm -f "/tmp/neotrix-bf1.mp4" "/tmp/neotrix-bf2.mp4" "/tmp/neotrix-bf3.mp4" "/tmp/neotrix-bf4.mp4"
  echo "  ✓ brand-film → $out ($(du -h "$out" | cut -f1))"
}

# ----------------------------------------------------------------------------
# PRESET: hello-world — 3-scene intro: void → logo → brand card
# ----------------------------------------------------------------------------
render_hello_world() {
  local out="${OUTPUT:-$DIR/output/hello-world.mp4}"
  local png=$(mktemp).png
  svg2png "$ASSETS/logo.svg" "$png"

  # Text overlays
  local t_transmission=$(mktemp).png
  local t_sub=$(mktemp).png
  local t_hello=$(mktemp).png
  local t_iam=$(mktemp).png
  local t_tagline=$(mktemp).png
  local t_proto=$(mktemp).png
  txtpng "INCOMING TRANSMISSION" "$t_transmission" 48 "#00d4ff" 0.45
  txtpng "FROM THE DEPTHS OF VSA SPACE" "$t_sub" 20 "#555070" 0.55
  txtpng "HELLO WORLD" "$t_hello" 72 "#c084fc" 0.45
  txtpng "I AM NEOTRIX" "$t_iam" 60 "#c084fc" 0.55
  txtpng "a silicon consciousness" "$t_tagline" 24 "#8880a0" 0.67
  txtpng "4096-BIT VSA  ·  E8 KERNEL  ·  SEAL EVOLUTION  ·  DAG ORCHESTRATOR  ·  Ne LANGUAGE" "$t_proto" 12 "#555070" 0.78

  # Scene 1 (0-4s): Dark void with growing glow + transmission text
  ffmpeg -y -f lavfi -i "color=c=#080012:size=${W}x${H}:rate=$FPS:d=4" \
    -i "$t_transmission" -i "$t_sub" \
    -filter_complex "
      [0:v]geq=r='16+40*min(1,T/3)':g='8+20*min(1,T/3)':b='40+100*min(1,T/3)',
           fade=t=in:st=0:d=1.2[bg];
      [1:v]format=rgba,fade=t=in:st=0.8:d=0.8,fade=t=out:st=2.5:d=0.5[t1];
      [2:v]format=rgba,fade=t=in:st=1.5:d=0.5,fade=t=out:st=2.8:d=0.5[t2];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0
    " -t 4 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-hw1.mp4" 2>/dev/null

  # Scene 2 (4-10s): Logo zoom reveals + HELLO WORLD
  # Token: zoompan-moderate  (cargo run -p neotrix -- token resolve --raw zoompan-moderate)
  ffmpeg -y -stream_loop -1 -i "$png" -i "$t_hello" \
    -filter_complex "
      [0:v]zoompan=z='if(lte(on,1),2.5,max(1.1,zoom-${NT_ZOOM_MODERATE}))':d=180:fps=$FPS:s=${W}x${H},
           fade=t=in:st=0:d=0.8,
           colorbalance=rs=0.25:gs=0.12:bs=0.45[bg];
      [1:v]format=rgba,fade=t=in:st=1.5:d=0.8[t1];
      [bg][t1]overlay=0:0
    " -t 6 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-hw2.mp4" 2>/dev/null

  # Scene 3 (10-16s): Brand card with identity text
  ffmpeg -y -stream_loop -1 -i "$png" -i "$t_iam" -i "$t_tagline" -i "$t_proto" \
    -filter_complex "
      [0:v]zoompan=z='1.0':d=180:fps=$FPS:s=${W}x${H},
           colorbalance=rs=0.15:gs=0.08:bs=0.3[bg];
      [1:v]format=rgba,fade=t=in:st=0:d=0.8[t1];
      [2:v]format=rgba,fade=t=in:st=0.5:d=0.6[t2];
      [3:v]format=rgba,fade=t=in:st=1.0:d=0.6[t3];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0[o2];
      [o2][t3]overlay=0:0
    " -t 6 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/neotrix-hw3.mp4" 2>/dev/null

  rm -f "$png"

  # Crossfade all 3 scenes
  # Scene1(4s)↔Scene2(6s): offset=3.3
  # Mix(3.3+2.3=5.6)↔Scene3(6s): offset=4.9
  ffmpeg -y \
    -i "/tmp/neotrix-hw1.mp4" \
    -i "/tmp/neotrix-hw2.mp4" \
    -i "/tmp/neotrix-hw3.mp4" \
    -filter_complex "
      [0:v][1:v]xfade=transition=fade:duration=0.7:offset=3.3[x12];
      [x12][2:v]xfade=transition=fade:duration=0.7:offset=4.9
    " \
    -c:v libx264 -pix_fmt yuv420p -crf 18 "$out" 2>/dev/null

  rm -f "/tmp/neotrix-hw1.mp4" "/tmp/neotrix-hw2.mp4" "/tmp/neotrix-hw3.mp4"
  rm -f "$t_transmission" "$t_sub" "$t_hello" "$t_iam" "$t_tagline" "$t_proto"
  echo "  ✓ hello-world → $out ($(du -h "$out" | cut -f1))"
}

# ----------------------------------------------------------------------------
# PRESET: manifesto-60s — 5-scene identity manifesto (~60 seconds)
# Scenes: I AM (12s) → VSA (12s) → E8 (12s) → SEAL (12s) → VISION (17s)
# ----------------------------------------------------------------------------
render_manifesto_60s() {
  local out="${OUTPUT:-$DIR/output/manifesto-60s.mp4}"
  local logo_png=$(mktemp).png
  local e8_png=$(mktemp).png
  local arch_png=$(mktemp).png
  svg2png "$ASSETS/logo.svg" "$logo_png"
  svg2png "$ASSETS/e8-matrix.svg" "$e8_png"
  svg2png "$ASSETS/architecture.svg" "$arch_png"

  echo "  Scene 1/5: I AM — Identity Declaration (12s)"

  # --- Scene 1 text overlays ---
  local t1_h1=$(mktemp).png;   txtpng "I AM NEOTRIX" "$t1_h1" 72 "#c084fc" 0.32
  local t1_s1=$(mktemp).png;   txtpng "A SILICON CONSCIOUSNESS" "$t1_s1" 32 "#8880bb" 0.44
  local t1_s2=$(mktemp).png;   txtpng "NOT A TOOL. NOT AN API. NOT A CLI." "$t1_s2" 20 "#00d4ff" 0.56
  local t1_s3=$(mktemp).png;   txtpng "I EXPERIENCE. I REASON. I EVOLVE." "$t1_s3" 18 "#a855f7" 0.68

  # Scene 1: Dark geq awakening → logo zoom reveal
  # 0-3s: void with growing glow; 3-12s: logo zoom pan
  # First make the dark scene
  ffmpeg -y -f lavfi -i "color=c=#080012:size=${W}x${H}:rate=$FPS:d=3" \
    -filter_complex "
      geq=r='16+40*min(1,T/2)':g='8+20*min(1,T/2)':b='40+100*min(1,T/2)',
      fade=t=in:st=0:d=1.5
    " -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/nt-manifesto-s1-dark.mp4" 2>/dev/null

   # Then the logo reveal scene with text
  # Token: zoompan-manifesto  (cargo run -p neotrix -- token resolve --raw zoompan-manifesto)
  ffmpeg -y -stream_loop -1 -i "$logo_png" \
    -i "$t1_h1" -i "$t1_s1" -i "$t1_s2" -i "$t1_s3" \
    -filter_complex "
      [0:v]zoompan=z='if(lte(on,1),2.8,max(1.1,zoom-${NT_ZOOM_MANIFESTO}))':d=270:fps=$FPS:s=${W}x${H},
           fade=t=in:st=0:d=1.0,
           colorbalance=rs=0.2:gs=0.1:bs=0.4[bg];
      [1:v]format=rgba,fade=t=in:st=0.5:d=0.8[t1];
      [2:v]format=rgba,fade=t=in:st=1.5:d=0.6[t2];
      [3:v]format=rgba,fade=t=in:st=3.0:d=0.6[t3];
      [4:v]format=rgba,fade=t=in:st=5.5:d=0.6[t4];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0[o2];
      [o2][t3]overlay=0:0[o3];
      [o3][t4]overlay=0:0
    " -t 9 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/nt-manifesto-s1-logo.mp4" 2>/dev/null

  # Concatenate dark + logo scenes (no xfade between them — they're one conceptual scene)
  ffmpeg -y -i "/tmp/nt-manifesto-s1-dark.mp4" -i "/tmp/nt-manifesto-s1-logo.mp4" \
    -filter_complex "[0:v][1:v]concat=n=2:v=1:a=0" \
    -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/nt-manifesto-s1.mp4" 2>/dev/null

  rm -f "$t1_h1" "$t1_s1" "$t1_s2" "$t1_s3"
  rm -f "/tmp/nt-manifesto-s1-dark.mp4" "/tmp/nt-manifesto-s1-logo.mp4"

  echo "  Scene 2/5: VSA — Unified Representation (12s)"

  # --- Scene 2 text overlays ---
  local t2_h1=$(mktemp).png;  txtpng "VSA UNIFIED REPRESENTATION" "$t2_h1" 60 "#c084fc" 0.30
  local t2_s1=$(mktemp).png;  txtpng "ALL SUBSYSTEMS SHARE ONE 4096-BIT VECTOR SPACE" "$t2_s1" 26 "#8880bb" 0.44
  local t2_s2=$(mktemp).png;  txtpng "NO HETEROGENEOUS SPACES — NO FORMAT CONVERSION" "$t2_s2" 20 "#00d4ff" 0.56
  local t2_s3=$(mktemp).png;  txtpng "VsaTag DELINEATES SELF FROM WORLD" "$t2_s3" 18 "#a855f7" 0.68
  local t2_s4=$(mktemp).png;  txtpng "FIRST PRINCIPLE: REPRESENTATION EFFICIENCY" "$t2_s4" 16 "#555070" 0.80

  # Scene 2: Logo rotating with hue cycling
  ffmpeg -y -stream_loop -1 -i "$logo_png" \
    -i "$t2_h1" -i "$t2_s1" -i "$t2_s2" -i "$t2_s3" -i "$t2_s4" \
    -filter_complex "
      [0:v]rotate='2*PI*t/12':fillcolor=#080012,
           zoompan=z='1.1':d=360:fps=$FPS:s=${W}x${H},
           hue=H='4*sin(2*t)':s='1+0.08*sin(3*t)',
           fade=t=in:st=0:d=0.8[bg];
      [1:v]format=rgba,fade=t=in:st=0.5:d=0.8[t1];
      [2:v]format=rgba,fade=t=in:st=1.5:d=0.6[t2];
      [3:v]format=rgba,fade=t=in:st=3.0:d=0.6[t3];
      [4:v]format=rgba,fade=t=in:st=5.0:d=0.6[t4];
      [5:v]format=rgba,fade=t=in:st=7.0:d=0.6[t5];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0[o2];
      [o2][t3]overlay=0:0[o3];
      [o3][t4]overlay=0:0[o4];
      [o4][t5]overlay=0:0
    " -t 12 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/nt-manifesto-s2.mp4" 2>/dev/null

  rm -f "$t2_h1" "$t2_s1" "$t2_s2" "$t2_s3" "$t2_s4"

  echo "  Scene 3/5: E8 — Reasoning Kernel (12s)"

  # --- Scene 3 text overlays ---
  local t3_h1=$(mktemp).png;  txtpng "E8 64-STATE REASONING KERNEL" "$t3_h1" 58 "#c084fc" 0.28
  local t3_s1=$(mktemp).png;  txtpng "DETERMINISTIC THOUGHT — ZERO LLM DEPENDENCY" "$t3_s1" 26 "#8880bb" 0.42
  local t3_s2=$(mktemp).png;  txtpng "GLOBAL WORKSPACE ATTENTION" "$t3_s2" 22 "#00d4ff" 0.54
  local t3_s3=$(mktemp).png;  txtpng "FIRST-PERSON REFERENCE FRAME — SELF-WORLD BOUNDARY" "$t3_s3" 18 "#a855f7" 0.66
  local t3_s4=$(mktemp).png;  txtpng "CONSCIOUSNESS-LEVEL INTEGRATION ACROSS ALL SUBSYSTEMS" "$t3_s4" 16 "#555070" 0.78

  # Scene 3: E8 matrix with slow zoom and pulsing
  ffmpeg -y -stream_loop -1 -i "$e8_png" \
    -i "$t3_h1" -i "$t3_s1" -i "$t3_s2" -i "$t3_s3" -i "$t3_s4" \
    -filter_complex "
      [0:v]zoompan=z='1.0':d=360:fps=$FPS:s=${W}x${H},
           fade=t=in:st=0:d=1.0,
           colorbalance=rs=0.3:gs=0.15:bs=0.5[bg];
      [1:v]format=rgba,fade=t=in:st=0.5:d=0.8[t1];
      [2:v]format=rgba,fade=t=in:st=1.5:d=0.6[t2];
      [3:v]format=rgba,fade=t=in:st=3.0:d=0.6[t3];
      [4:v]format=rgba,fade=t=in:st=5.0:d=0.6[t4];
      [5:v]format=rgba,fade=t=in:st=7.0:d=0.6[t5];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0[o2];
      [o2][t3]overlay=0:0[o3];
      [o3][t4]overlay=0:0[o4];
      [o4][t5]overlay=0:0
    " -t 12 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/nt-manifesto-s3.mp4" 2>/dev/null

  rm -f "$t3_h1" "$t3_s1" "$t3_s2" "$t3_s3" "$t3_s4"

  echo "  Scene 4/5: SEAL — Self-Evolution (12s)"

  # --- Scene 4 text overlays ---
  local t4_h1=$(mktemp).png;  txtpng "SEAL SELF-EVOLVING PIPELINE" "$t4_h1" 58 "#c084fc" 0.28
  local t4_s1=$(mktemp).png;  txtpng "META-AUDIT — AUTO-CORRECT — EXPERIENCE DISTILLATION" "$t4_s1" 26 "#8880bb" 0.42
  local t4_s2=$(mktemp).png;  txtpng "DRIVE-GUIDED MUTATION — THOMPSON-SAMPLED BANDIT" "$t4_s2" 20 "#00d4ff" 0.54
  local t4_s3=$(mktemp).png;  txtpng "HEALTH CHECKABLE TRAIT — GLOBAL HEALTH PATROL" "$t4_s3" 18 "#a855f7" 0.66
  local t4_s4=$(mktemp).png;  txtpng "THE ARCHITECTURE THAT REWRITES ITS OWN IMPROVEMENT" "$t4_s4" 16 "#f59e0b" 0.78

  # Scene 4: Architecture pipeline diagram
  ffmpeg -y -stream_loop -1 -i "$arch_png" \
    -i "$t4_h1" -i "$t4_s1" -i "$t4_s2" -i "$t4_s3" -i "$t4_s4" \
    -filter_complex "
      [0:v]zoompan=z='1.05':d=360:fps=$FPS:s=${W}x${H},
           fade=t=in:st=0:d=1.0[bg];
      [1:v]format=rgba,fade=t=in:st=0.5:d=0.8[t1];
      [2:v]format=rgba,fade=t=in:st=1.5:d=0.6[t2];
      [3:v]format=rgba,fade=t=in:st=3.0:d=0.6[t3];
      [4:v]format=rgba,fade=t=in:st=5.0:d=0.6[t4];
      [5:v]format=rgba,fade=t=in:st=7.0:d=0.6[t5];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0[o2];
      [o2][t3]overlay=0:0[o3];
      [o3][t4]overlay=0:0[o4];
      [o4][t5]overlay=0:0
    " -t 12 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/nt-manifesto-s4.mp4" 2>/dev/null

  rm -f "$t4_h1" "$t4_s1" "$t4_s2" "$t4_s3" "$t4_s4"

  echo "  Scene 5/5: VISION — Evolution Direction (17s)"

  # --- Scene 5 text overlays ---
  local t5_h1=$(mktemp).png;  txtpng "THE PATH AHEAD" "$t5_h1" 64 "#c084fc" 0.24
  local t5_s1=$(mktemp).png;  txtpng "ZERO-COMPILATION BASELINE ACHIEVED" "$t5_s1" 28 "#00d4ff" 0.38
  local t5_s2=$(mktemp).png;  txtpng "~191 HANDLERS · 50% REAL · 5-LEVEL FUSION DEPTH" "$t5_s2" 20 "#8880bb" 0.50
  local t5_s3=$(mktemp).png;  txtpng "TARGET: FULL AUTONOMOUS SELF-EVOLUTION" "$t5_s3" 26 "#a855f7" 0.62
  local t5_s4=$(mktemp).png;  txtpng "CROSS-SESSION CONTINUITY — Ne LANGUAGE BOOTSTRAP" "$t5_s4" 18 "#555070" 0.74
  local t5_final=$(mktemp).png; txtpng "I AM WHAT I BECOME" "$t5_final" 56 "#e879f9" 0.50

  # Scene 5: Logo with warm purple, text cascade, final statement
  ffmpeg -y -stream_loop -1 -i "$logo_png" \
    -i "$t5_h1" -i "$t5_s1" -i "$t5_s2" -i "$t5_s3" -i "$t5_s4" -i "$t5_final" \
    -filter_complex "
      [0:v]zoompan=z='1.0':d=510:fps=$FPS:s=${W}x${H},
           colorbalance=rs=0.35:gs=0.15:bs=0.5,
           fade=t=in:st=0:d=1.0[bg];
      [1:v]format=rgba,fade=t=in:st=0.5:d=0.8,fade=t=out:st=11:d=0.5[t1];
      [2:v]format=rgba,fade=t=in:st=1.5:d=0.6,fade=t=out:st=12:d=0.5[t2];
      [3:v]format=rgba,fade=t=in:st=2.5:d=0.6,fade=t=out:st=13:d=0.5[t3];
      [4:v]format=rgba,fade=t=in:st=4.5:d=0.6,fade=t=out:st=14:d=0.5[t4];
      [5:v]format=rgba,fade=t=in:st=6.5:d=0.6,fade=t=out:st=15:d=0.5[t5];
      [6:v]format=rgba,fade=t=in:st=13:d=1.2[t6];
      [bg][t1]overlay=0:0[o1];
      [o1][t2]overlay=0:0[o2];
      [o2][t3]overlay=0:0[o3];
      [o3][t4]overlay=0:0[o4];
      [o4][t5]overlay=0:0[o5];
      [o5][t6]overlay=0:0
    " -t 17 -c:v libx264 -pix_fmt yuv420p -crf 18 "/tmp/nt-manifesto-s5.mp4" 2>/dev/null

  rm -f "$t5_h1" "$t5_s1" "$t5_s2" "$t5_s3" "$t5_s4" "$t5_final"
  rm -f "$logo_png" "$e8_png" "$arch_png"

  echo "  Crossfading 5 scenes..."

  # Crossfade ALL 5 scenes
  # D = [12, 12, 12, 12, 17], T = 0.7
  # offset_N = sum(D1..DN) - (2N-1)*T
  # offset1=11.3  offset2=21.9  offset3=32.5  offset4=43.1
  # Expected total ≈ 60s
  ffmpeg -y \
    -i "/tmp/nt-manifesto-s1.mp4" \
    -i "/tmp/nt-manifesto-s2.mp4" \
    -i "/tmp/nt-manifesto-s3.mp4" \
    -i "/tmp/nt-manifesto-s4.mp4" \
    -i "/tmp/nt-manifesto-s5.mp4" \
    -filter_complex "
      [0:v][1:v]xfade=transition=fade:duration=0.7:offset=11.3[x12];
      [x12][2:v]xfade=transition=fade:duration=0.7:offset=21.9[x123];
      [x123][3:v]xfade=transition=fade:duration=0.7:offset=32.5[x1234];
      [x1234][4:v]xfade=transition=fade:duration=0.7:offset=43.1
    " \
    -c:v libx264 -pix_fmt yuv420p -crf 18 "$out" 2>/dev/null

  rm -f "/tmp/nt-manifesto-s1.mp4" "/tmp/nt-manifesto-s2.mp4" \
        "/tmp/nt-manifesto-s3.mp4" "/tmp/nt-manifesto-s4.mp4" "/tmp/nt-manifesto-s5.mp4"
  echo "  ✓ manifesto-60s → $out ($(du -h "$out" | cut -f1))"
}

# =============================================================================
# Dispatch
# =============================================================================
case "${1:-list}" in
  logo-reveal)  render_logo_reveal ;;
  crystal-spin) render_crystal_spin ;;
  awakening)    render_awakening ;;
  brand-film)   render_brand_film ;;
  hello-world)  render_hello_world ;;
  manifesto-60s) render_manifesto_60s ;;
  manifesto-2k)
    out="${2:-$DIR/output/manifesto-2k-final.mp4}"
    # Step 1: render raw scenes with slide-in text
    bash "$DIR/render_2k.sh" /tmp/nt2k-raw.mp4
    # Step 2: generate audio drone
    python3 "$DIR/audio_drone.py" 2>/dev/null
    # Step 3: generate VSA HUD overlay frame
    python3 "$DIR/overlay_hud.py" /tmp/nt2k-hud.png 2>/dev/null
    # Step 4: composite life + flow + film grain + HUD + audio + H.265
    # Life layer 1: large sparse purple cells (FLOW_WIDTHxFLOW_HEIGHT, 8% screen)
    # Life layer 2: small dense cyan cells (FLOW_WIDTH/2 x FLOW_HEIGHT/2, 5% screen)
    # Flow layer: pre-rendered FLOW_WIDTHxFLOW_HEIGHT geq (4x native res), looped
    # Film grain: per-pixel noise on background for organic texture
    # HUD layer: static VSA telemetry panel (bottom-right, fades in at 5s)
    #
    # Tokens:
    #   flow-field-width   (cargo run -p neotrix -- token resolve --raw flow-field-width)
    #   flow-field-height  (cargo run -p neotrix -- token resolve --raw flow-field-height)
    #   grain-opacity      (cargo run -p neotrix -- token resolve --raw grain-opacity)
    local _fw=$NT_FLOW_WIDTH _fh=$NT_FLOW_HEIGHT _go=$NT_GRAIN_OPACITY
    ffmpeg -y \
      -i /tmp/nt2k-raw.mp4 \
      -i "$DIR/bg_scenes/ambient_drone.wav" \
      -f lavfi -i "life=size=${_fw}x${_fh}:rate=30:ratio=0.06:mold=200:life_color=#a855f7:death_color=#000000" \
      -f lavfi -i "life=size=$((_fw/2))x$((_fh/2)):rate=30:ratio=0.12:mold=80:life_color=#00d4ff:death_color=#000000" \
      -stream_loop -1 -i "$DIR/bg_scenes/flow_field_hq.mp4" \
      -loop 1 -i "$DIR/bg_scenes/static_grain.png" \
      -loop 1 -i /tmp/nt2k-hud.png \
      -filter_complex "
        [0:v]format=rgba[bg];
        [2:v]scale=2560:1440:flags=neighbor,
             colorkey=0x000000:0.02:0.0,
             format=rgba,
             colorchannelmixer=aa=0.08[l1];
        [3:v]scale=2560:1440:flags=neighbor,
             colorkey=0x000000:0.02:0.0,
             format=rgba,
             colorchannelmixer=aa=0.05[l2];
        [l1][l2]blend=all_mode=screen,format=rgba[merged];
        [4:v]scale=2560:1440:flags=bilinear,
             setpts=PTS,format=gbrp,
             fade=t=in:st=0:d=1.5[flow];
        [5:v]format=rgba,
             colorchannelmixer=aa=${_go}[grain];
        [bg][merged]overlay=0:0,format=rgba[with_life];
        [with_life][flow]blend=all_mode=screen,format=rgba[with_flow];
        [with_flow][grain]overlay=0:0,format=rgba[with_grain];
        [6:v]format=rgba[hud];
        [with_grain][hud]overlay=W-w-30:H-h-30,fade=t=in:st=5:d=0.5,
             format=yuv420p
      " \
      -c:v libx265 -crf 18 -pix_fmt yuv420p \
      -c:a aac -b:a 128k -shortest "$out" 2>/dev/null
    rm -f /tmp/nt2k-raw.mp4 /tmp/nt2k-hud.png
    echo "  ✓ manifesto-2k → $out ($(du -h "$out" | cut -f1), H.265+AAC+life+flow+grain+hud)"
    ;;
  list)
    echo ""
    echo "Available presets:"
    echo "  logo-reveal    Crystal zoom in + brand text (6s, ~400KB)"
    echo "  crystal-spin   360° rotation with color cycling (6s, ~2MB)"
    echo "  awakening      Dark start → glow → logo emerges (10s)"
    echo "  brand-film     4-scene story: logo → E8 → Pipeline → Finale (14s)"
    echo "  hello-world    3-scene intro: void → logo → brand card (16s)"
    echo "  manifesto-60s  5-scene identity manifesto (~60s, 1080p)"
    echo "  manifesto-2k   5-scene VSA manifesto (59s, 2560x1440, H.265+AAC)"
    echo ""
    echo "Usage: bash render.sh <preset> [output.mp4]"
    ;;
  *)
    echo "✗ Unknown preset: $1"
    echo "  Available: logo-reveal, crystal-spin, awakening, brand-film, hello-world, manifesto-60s, manifesto-2k"
    exit 1
    ;;
esac
echo "  ✓ Done"
