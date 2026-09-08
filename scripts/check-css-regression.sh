#!/bin/bash
# pre-commit: verify critical CSS properties exist in dist
# Run after `npx vite build` to catch regressions

set -e

CSS_FILE=$(ls /Users/neo/Downloads/neotrix/neocodex-frontend/dist/assets/index-*.css 2>/dev/null | head -1)

if [ -z "$CSS_FILE" ]; then
  echo "❌ No dist CSS found. Run 'npx vite build' first."
  exit 1
fi

FAIL=0

# 1. html must have overflow:hidden + border-radius:12px (Tauri window corners)
if ! grep -q 'html{[^}]*overflow:hidden[^}]*border-radius:12px' "$CSS_FILE"; then
  echo "❌ REGRESSION: html missing overflow:hidden + border-radius:12px"
  FAIL=1
fi

# 2. body must use --color-canvas variable (theme consistency)
if ! grep -q 'var(--color-canvas' "$CSS_FILE"; then
  echo "❌ REGRESSION: body missing var(--color-canvas)"
  FAIL=1
fi

# 3. glass-side must use --color-panel variable (sidebar theme)
if ! grep -q 'var(--color-panel' "$CSS_FILE"; then
  echo "❌ REGRESSION: .glass-side missing var(--color-panel)"
  FAIL=1
fi

# 4. :root must define --color-canvas
if ! grep -q ':root{[^}]*--color-canvas' "$CSS_FILE"; then
  echo "❌ REGRESSION: :root missing --color-canvas definition"
  FAIL=1
fi

if [ $FAIL -eq 0 ]; then
  echo "✅ CSS regression check passed"
else
  exit 1
fi
