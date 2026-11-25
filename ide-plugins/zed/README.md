# KMIPKit Zed Extension (`ide-plugins/zed/`)

Rust -> WebAssembly (`wasm32-wasip1`) extension for Zed connecting to `kmip-lsp`.

## Build
```bash
rustup target add wasm32-wasip1
cargo build --manifest-path ide-plugins/zed/Cargo.toml --target wasm32-wasip1 --release
```
