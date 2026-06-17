#!/usr/bin/env bash
# Downloads static/portable binaries for local development.
# These are the same binaries that CI bundles into the installer.
set -euo pipefail

# Allow CI to override the target triple via env var
TARGET="${RUST_TARGET:-$(rustc -vV | grep host | awk '{print $2}')}"
BINDIR="src-tauri/binaries"
mkdir -p "$BINDIR"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

echo "=== Setting up dev sidecars for $TARGET ==="

# ── FFmpeg ───────────────────────────────────────────────────────────────────
if [[ "$TARGET" == *linux* ]]; then
  echo "→ Downloading FFmpeg static (Linux)…"
  curl -# -L "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-linux64-gpl.tar.xz" \
    -o "$TMP/ffmpeg.tar.xz"
  tar -xJf "$TMP/ffmpeg.tar.xz" -C "$TMP"
  find "$TMP" -name "ffmpeg" -not -name "ffprobe" -not -name "ffplay" -type f \
    | head -1 | xargs -I{} cp {} "$BINDIR/ffmpeg-$TARGET"
elif [[ "$TARGET" == *darwin* ]]; then
  echo "→ Installing FFmpeg via Homebrew (macOS)…"
  brew install ffmpeg 2>/dev/null || brew upgrade ffmpeg 2>/dev/null || true
  cp -L "$(brew --prefix)/bin/ffmpeg" "$BINDIR/ffmpeg-$TARGET"
fi
chmod +x "$BINDIR/ffmpeg-$TARGET"
echo "  ✓ ffmpeg $("$BINDIR/ffmpeg-$TARGET" -version 2>&1 | head -1)"

# ── ImageMagick ───────────────────────────────────────────────────────────────
if [[ "$TARGET" == *linux* ]]; then
  echo "→ Downloading ImageMagick portable (Linux)…"
  curl -# -L "https://imagemagick.org/archive/binaries/magick" \
    -o "$BINDIR/magick-$TARGET"
elif [[ "$TARGET" == *darwin* ]]; then
  echo "→ Installing ImageMagick via Homebrew (macOS)…"
  brew install imagemagick 2>/dev/null || brew upgrade imagemagick 2>/dev/null || true
  cp -L "$(brew --prefix)/bin/magick" "$BINDIR/magick-$TARGET"
fi
chmod +x "$BINDIR/magick-$TARGET"
echo "  ✓ magick $("$BINDIR/magick-$TARGET" --version 2>&1 | head -1 || echo '(checking...)')"

# ── Pandoc ────────────────────────────────────────────────────────────────────
PANDOC_VERSION="3.6.4"
if [[ "$TARGET" == *linux* ]]; then
  echo "→ Downloading Pandoc $PANDOC_VERSION (Linux)…"
  curl -# -L "https://github.com/jgm/pandoc/releases/download/${PANDOC_VERSION}/pandoc-${PANDOC_VERSION}-linux-amd64.tar.gz" \
    -o "$TMP/pandoc.tar.gz"
  tar -xzf "$TMP/pandoc.tar.gz" -C "$TMP"
  find "$TMP" -name "pandoc" -type f | head -1 \
    | xargs -I{} cp {} "$BINDIR/pandoc-$TARGET"
elif [[ "$TARGET" == *darwin* ]]; then
  echo "→ Installing Pandoc via Homebrew (macOS)…"
  brew install pandoc 2>/dev/null || brew upgrade pandoc 2>/dev/null || true
  cp -L "$(brew --prefix)/bin/pandoc" "$BINDIR/pandoc-$TARGET"
fi
chmod +x "$BINDIR/pandoc-$TARGET"
echo "  ✓ pandoc $("$BINDIR/pandoc-$TARGET" --version 2>&1 | head -1)"

echo ""
echo "=== All sidecars ready ==="
ls -lh "$BINDIR/"
