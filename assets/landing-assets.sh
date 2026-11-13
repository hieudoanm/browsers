#!/usr/bin/env bash
# Sync branding SVG sources into packages/landing/public/.
#
#   assets/branding/svg/logo.svg -> packages/landing/public/logo.svg + favicon.svg
#   assets/branding/svg/og.svg   -> packages/landing/public/og.png (1200x630)
#   packages/landing/public/logo.svg -> favicon.ico (16/32/48/256)
#
# Requires rsvg-convert (librsvg); ImageMagick (magick) for favicon.ico.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BRANDING_SVG="$REPO_ROOT/assets/branding/svg"
PUBLIC_DIR="$REPO_ROOT/packages/landing/public"

command -v rsvg-convert >/dev/null 2>&1 || {
  echo "rsvg-convert is required but not installed" >&2
  echo "  macOS:  brew install librsvg" >&2
  exit 1
}

mkdir -p "$PUBLIC_DIR"

if [ -f "$BRANDING_SVG/logo.svg" ]; then
  cp "$BRANDING_SVG/logo.svg" "$PUBLIC_DIR/logo.svg"
  cp "$BRANDING_SVG/logo.svg" "$PUBLIC_DIR/favicon.svg"
  echo "copied $BRANDING_SVG/logo.svg -> $PUBLIC_DIR/{logo,favicon}.svg"
fi

if [ -f "$BRANDING_SVG/og.svg" ]; then
  rsvg-convert -w 1200 -h 630 -o "$PUBLIC_DIR/og.png" "$BRANDING_SVG/og.svg"
  echo "rendered $BRANDING_SVG/og.svg -> $PUBLIC_DIR/og.png"
fi

if command -v magick >/dev/null 2>&1 && [ -f "$PUBLIC_DIR/logo.svg" ]; then
  magick -background none "$PUBLIC_DIR/logo.svg" \
    \( -clone 0 -resize 16x16 \) \
    \( -clone 0 -resize 32x32 \) \
    \( -clone 0 -resize 48x48 \) \
    \( -clone 0 -resize 256x256 \) \
    -delete 0 "$PUBLIC_DIR/favicon.ico"
  echo "rendered $PUBLIC_DIR/favicon.ico (16/32/48/256)"
elif [ -f "$PUBLIC_DIR/logo.svg" ]; then
  echo "note: 'magick' not found — favicon.ico left unchanged" >&2
fi
