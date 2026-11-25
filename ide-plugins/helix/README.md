# KMIPKit Helix Integration (`ide-plugins/helix/`)

Native LSP integration for the **Helix** editor using `kmip-lsp`.

## Setup
Ensure `kmip-lsp` is on your `PATH`, then merge [languages.toml](languages.toml) into `~/.config/helix/languages.toml`:

```bash
mkdir -p ~/.config/helix
cat ide-plugins/helix/languages.toml >> ~/.config/helix/languages.toml
```

## Capabilities
- Hover documentation for OASIS KMIP 1.0–3.0 tags and enumerations.
- Contextual completion and diagnostics for `*.kmip.json`, `*.kmip-config.json`, `*.kmip-policy.json`, Rust, C, Java, and Python.
- Quick-fixes for weak algorithms (`3DES`, `RC4`, `MD5`, `SHA-1`) and deprecated KMIP 1.x tags (`TemplateAttribute`).
