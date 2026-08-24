#!/usr/bin/env bash
set -euo pipefail

# setup-ffmpeg.sh — Download & verifikasi FFmpeg/FFprobe build BtbN (lgpl)
# untuk digunakan sebagai Tauri sidecar di src-tauri/binaries/.
# Replikasi logika CI: .github/workflows/build-verify.yml
#
# Usage:  bash scripts/setup-ffmpeg.sh     (atau:  npm run setup:ffmpeg)

FFMPEG_RELEASE_TAG="autobuild-2026-08-19-19-21"
BASE_URL="https://github.com/BtbN/FFmpeg-Builds/releases/download/${FFMPEG_RELEASE_TAG}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BINARIES_DIR="${PROJECT_ROOT}/src-tauri/binaries"
WORKDIR="${PROJECT_ROOT}/.ffmpeg-setup-tmp"

OS_NAME="$(uname -s)"
case "$OS_NAME" in
  Linux*)
    ARCH_TAG="linux64"
    FFMPEG_BIN="ffmpeg-x86_64-unknown-linux-gnu"
    FFPROBE_BIN="ffprobe-x86_64-unknown-linux-gnu"
    ;;
  Darwin*)
    ARCH_TAG="macos64"
    if [ "$(uname -m)" = "arm64" ]; then
      FFMPEG_BIN="ffmpeg-aarch64-apple-darwin"
      FFPROBE_BIN="ffprobe-aarch64-apple-darwin"
    else
      FFMPEG_BIN="ffmpeg-x86_64-apple-darwin"
      FFPROBE_BIN="ffprobe-x86_64-apple-darwin"
    fi
    ;;
  *)
    echo "ERROR: Platform tidak didukung: $OS_NAME"
    echo "Gunakan scripts/setup-ffmpeg.ps1 untuk Windows."
    exit 1
    ;;
esac

COMMIT_HASH="N-126217-ge1e325235e"
ASSET_NAME="ffmpeg-${COMMIT_HASH}-${ARCH_TAG}-lgpl.tar.xz"

echo ">> Platform : $OS_NAME"
echo ">> Tag      : $FFMPEG_RELEASE_TAG"
echo ">> Asset    : $ASSET_NAME"
echo ""

rm -rf "$WORKDIR"
mkdir -p "$WORKDIR" "$BINARIES_DIR"
cd "$WORKDIR"

echo ">> [1/4] Downloading FFmpeg archive..."
curl -fL --retry 3 -o "$ASSET_NAME" "${BASE_URL}/${ASSET_NAME}"

echo ">> [2/4] Verifying sha256 against release checksums..."
curl -fL --retry 3 -o checksums.sha256 "${BASE_URL}/checksums.sha256"

EXPECTED_LINE="$(grep "$ASSET_NAME" checksums.sha256 || true)"
if [ -z "$EXPECTED_LINE" ]; then
  echo "WARNING: Asset tidak ditemukan di checksums.sha256 rilis ini."
  echo "Jangan lanjutkan tanpa verifikasi manual. sha256 hasil download:"
  sha256sum "$ASSET_NAME"
  rm -rf "$WORKDIR"
  exit 1
fi

echo "$EXPECTED_LINE" | sha256sum -c -
echo ">> Checksum OK."

echo ">> [3/4] Extracting..."
mkdir -p ffmpeg-extracted
tar -xf "$ASSET_NAME" -C ffmpeg-extracted --strip-components=1

echo ">> [4/4] Installing binaries..."
cp "ffmpeg-extracted/bin/ffmpeg" "$BINARIES_DIR/$FFMPEG_BIN"
cp "ffmpeg-extracted/bin/ffprobe" "$BINARIES_DIR/$FFPROBE_BIN"
chmod +x "$BINARIES_DIR/$FFMPEG_BIN"
chmod +x "$BINARIES_DIR/$FFPROBE_BIN"

cd "$PROJECT_ROOT"
rm -rf "$WORKDIR"

echo ""
echo ">> Selesai! Binaries tersedia di:"
echo "   $BINARIES_DIR/$FFMPEG_BIN"
echo "   $BINARIES_DIR/$FFPROBE_BIN"
echo ">> Jalankan: cargo tauri dev"
