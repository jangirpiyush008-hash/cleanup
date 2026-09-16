#!/usr/bin/env bash
# Wraps the SwiftPM executable into a proper MacCleanup.app bundle.
# Usage:
#     ./scripts/build-app.sh           # release build → build/MacCleanup.app
#     ./scripts/build-app.sh debug     # debug build
#
# Requires Command Line Tools (swift). No full Xcode needed.

set -euo pipefail

CONFIG="${1:-release}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
APP_NAME="MacCleanup"
BUNDLE_NAME="MacCleanup.app"
OUT_DIR="$ROOT/build"
BUNDLE_DIR="$OUT_DIR/$BUNDLE_NAME"

echo "▶ swift build -c $CONFIG"
cd "$ROOT"
swift build -c "$CONFIG"

BIN_PATH=$(swift build -c "$CONFIG" --show-bin-path)/$APP_NAME
if [[ ! -f "$BIN_PATH" ]]; then
  echo "✗ binary not found at $BIN_PATH"
  exit 1
fi

echo "▶ Assembling $BUNDLE_NAME"
rm -rf "$BUNDLE_DIR"
mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

cp "$BIN_PATH" "$BUNDLE_DIR/Contents/MacOS/$APP_NAME"
cp "$ROOT/Sources/MacCleanup/Resources/Info.plist" "$BUNDLE_DIR/Contents/Info.plist"

# Ad-hoc sign so macOS will let a local user launch it without
# "unidentified developer" every single time. For distribution to another
# Mac, replace this with a proper Developer ID signature + notarization.
echo "▶ Ad-hoc code-signing (local use only)"
codesign --force --deep --sign - "$BUNDLE_DIR" 2>&1 | sed 's/^/  /'

echo ""
echo "✓ Built $BUNDLE_DIR"
echo ""
echo "Run it:      open '$BUNDLE_DIR'"
echo "Copy it:     the whole $BUNDLE_NAME folder is portable."
echo ""
echo "For distribution to another Mac WITHOUT Gatekeeper warnings,"
echo "re-sign with a Developer ID and notarize (see README)."
