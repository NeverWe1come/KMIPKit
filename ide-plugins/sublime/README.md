# KMIPKit Sublime Text Package (`ide-plugins/sublime/`)

`LSP-kmip` package for **Sublime Text 4** integrating with the `LSP` package and `kmip-lsp`.

## Features
- Automatic LSP attachment for JSON, Rust, C, Java, and Python files.
- Context menu & Command Palette: **KMIP: Decode TTLV Selection** (with default sensitive-material redaction) and **KMIP: Lookup OASIS Specification**.

## Tests
```bash
python3 -m unittest discover -s ide-plugins/sublime/tests
```
