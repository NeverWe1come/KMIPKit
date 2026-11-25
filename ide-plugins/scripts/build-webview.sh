#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "Building webview-ui..."
(cd "$ROOT_DIR/webview-ui" && npm run build)

VSCODE_DEST="$ROOT_DIR/vscode/media/webview"
JETBRAINS_DEST="$ROOT_DIR/jetbrains/src/main/resources/webview"

rm -rf "$VSCODE_DEST" "$JETBRAINS_DEST"
mkdir -p "$VSCODE_DEST" "$JETBRAINS_DEST"

cp -R "$ROOT_DIR/webview-ui/dist/." "$VSCODE_DEST/"
cp -R "$ROOT_DIR/webview-ui/dist/." "$JETBRAINS_DEST/"

echo "Synced webview-ui/dist -> vscode/media/webview and jetbrains/src/main/resources/webview"
