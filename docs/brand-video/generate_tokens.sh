#!/usr/bin/env bash
# =============================================================================
# NeoTrix Design Token Resolver — pre-resolves NT tokens as shell variables.
# Source this file:  source docs/brand-video/generate_tokens.sh
#
# Auto-generated token list from: cargo run -p neotrix -- token -- list
# =============================================================================
set -euo pipefail

NT_CLI="cargo run -q -p neotrix -- token --"

# Show available tokens (useful for debugging)
echo "✦ NT Design Tokens:" >&2
$NT_CLI list 2>/dev/null | sed 's/^/  /' >&2
echo >&2

# Helper: resolve a token, return raw value (no units, no metadata)
nt_resolve() {
  $NT_CLI resolve --raw "$1" 2>/dev/null
}

# Helper: resolve and strip trailing "px" if present
nt_resolve_px() {
  local val
  val=$($NT_CLI resolve --raw "$1" 2>/dev/null)
  echo "${val%px}"
}

# ── Zoompan decay rates ──────────────────────────────────────────────────────
#   cargo run -p neotrix -- token resolve --raw zoompan-speed
#   cargo run -p neotrix -- token resolve --raw zoompan-fast
#   cargo run -p neotrix -- token resolve --raw zoompan-slow
#   cargo run -p neotrix -- token resolve --raw zoompan-moderate
#   cargo run -p neotrix -- token resolve --raw zoompan-manifesto
NT_ZOOM_SPEED=$(nt_resolve zoompan-speed)
NT_ZOOM_FAST=$(nt_resolve zoompan-fast)
NT_ZOOM_SLOW=$(nt_resolve zoompan-slow)
NT_ZOOM_MODERATE=$(nt_resolve zoompan-moderate)
NT_ZOOM_MANIFESTO=$(nt_resolve zoompan-manifesto)

# ── Flow field / Conway life dimensions ──────────────────────────────────────
#   cargo run -p neotrix -- token resolve --raw flow-field-width
#   cargo run -p neotrix -- token resolve --raw flow-field-height
NT_FLOW_WIDTH=$(nt_resolve_px flow-field-width)
NT_FLOW_HEIGHT=$(nt_resolve_px flow-field-height)

# ── Grain opacity ────────────────────────────────────────────────────────────
#   cargo run -p neotrix -- token resolve --raw grain-opacity
NT_GRAIN_OPACITY=$(nt_resolve grain-opacity)

# ── Resolution tokens (HD) ───────────────────────────────────────────────────
#   cargo run -p neotrix -- token resolve --raw resolution-hd-width
#   cargo run -p neotrix -- token resolve --raw resolution-hd-height
NT_HD_WIDTH=$(nt_resolve_px resolution-hd-width)
NT_HD_HEIGHT=$(nt_resolve_px resolution-hd-height)

# ── Resolution tokens (2K) ───────────────────────────────────────────────────
#   cargo run -p neotrix -- token resolve --raw resolution-2k-width
#   cargo run -p neotrix -- token resolve --raw resolution-2k-height
NT_2K_WIDTH=$(nt_resolve_px resolution-2k-width)
NT_2K_HEIGHT=$(nt_resolve_px resolution-2k-height)

# ── 2K zoompan parameters ────────────────────────────────────────────────────
#   cargo run -p neotrix -- token resolve --raw zoompan-2k-base
#   cargo run -p neotrix -- token resolve --raw zoompan-2k-factor
NT_ZOOM_2K_BASE=$(nt_resolve zoompan-2k-base)
NT_ZOOM_2K_FACTOR=$(nt_resolve zoompan-2k-factor)

export NT_ZOOM_SPEED NT_ZOOM_FAST NT_ZOOM_SLOW NT_ZOOM_MODERATE NT_ZOOM_MANIFESTO
export NT_FLOW_WIDTH NT_FLOW_HEIGHT
export NT_GRAIN_OPACITY
export NT_HD_WIDTH NT_HD_HEIGHT
export NT_2K_WIDTH NT_2K_HEIGHT
export NT_ZOOM_2K_BASE NT_ZOOM_2K_FACTOR
