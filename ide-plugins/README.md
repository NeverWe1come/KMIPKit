# KMIPKit IDE Plugins (`ide-plugins/`)

Official IDE plugins and shared protocol tooling for **KMIPKit**, built on a **Core-First / Thin Adapters** architecture.

## Directory Layout

* `core/` — Shared Rust binary `kmip-lsp` (LSP over `stdio`, custom `kmip/*` JSON-RPC methods, CLI decoder/encoder, OASIS KMIP 1.0–3.0 catalog, mandatory sensitive-material redaction) built directly on `kmipkit-ttlv` and `kmipkit-protocol`.
* `schemas/` — Official JSON Schemas (`kmip-config.schema.json`, `kmip-message.schema.json`, `kmip-policy.schema.json`).
* `webview-ui/` — Shared static web UI (`IdeBridge`, interactive TTLV Inspector with synchronized hex view, Playground skeleton).
* `vscode/` — Official extension for VS Code, Cursor, Windsurf, and VSCodium.
* `jetbrains/` — Unified plugin (`org.kmipkit.ide`) for IntelliJ IDEA, CLion, PyCharm, RustRover, GoLand, Rider, WebStorm, and Android Studio.
* `scripts/` — Build, version-sync, and SHA-256 verification scripts.

## Quick Build & Test

### 1. Core (`kmip-lsp`)
```bash
export PATH="$HOME/.cargo/bin:$PATH"
cargo test --manifest-path ide-plugins/core/Cargo.toml
cargo build --manifest-path ide-plugins/core/Cargo.toml --release
```

### 2. Shared Webview UI
```bash
cd ide-plugins/webview-ui
npm install
npm test
npm run build
```

### 3. VS Code Extension
```bash
./ide-plugins/scripts/build-webview.sh
cd ide-plugins/vscode
npm install
npm run compile
npm test
```

### 4. JetBrains Plugin (requires JDK 21)
```bash
./ide-plugins/scripts/build-webview.sh
cd ide-plugins/jetbrains
./gradlew test verifyPlugin
```
