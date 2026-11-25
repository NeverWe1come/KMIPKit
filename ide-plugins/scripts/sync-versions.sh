#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(tr -d '[:space:]' < "$ROOT_DIR/VERSION")"

if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-].+)?$ ]]; then
  echo "Invalid SemVer in $ROOT_DIR/VERSION: $VERSION" >&2
  exit 1
fi

echo "Syncing ide-plugins version -> $VERSION"

# core/Cargo.toml
sed -i -E "s/^version = \"[^\"]+\"/version = \"$VERSION\"/" "$ROOT_DIR/core/Cargo.toml"

# webview-ui/package.json and vscode/package.json
for pkg in "$ROOT_DIR/webview-ui/package.json" "$ROOT_DIR/vscode/package.json"; do
  node -e "
    const fs = require('fs');
    const p = '$pkg';
    const data = JSON.parse(fs.readFileSync(p, 'utf8'));
    data.version = '$VERSION';
    fs.writeFileSync(p, JSON.stringify(data, null, 2) + '\n');
  "
done

# jetbrains/gradle.properties
sed -i -E "s/^pluginVersion=.*/pluginVersion=$VERSION/" "$ROOT_DIR/jetbrains/gradle.properties"

echo "Version sync complete."
