# Feature Specification: Standalone Rust KMIP 2.1 Validator

**Feature Branch**: `feature/KMIPKIT-0059-interop-rust-validator`
**Status**: Approved for implementation by maintainer plan authorization
**Owning identifier**: KMIPKIT-0059

Provide a standalone, independent Rust project (`interop/rust-kmip-validator/`)
that executes the exhaustive KMIP 2.1 test catalog against arbitrary KMIP servers
using capability matching, anti-collision isolation, and graphical report generation.

## Purpose

1. Isolated standalone Rust project with independent workspace (`[workspace]` root).
2. Native parser for server capabilities and connection profiles.
3. Capability matching engine identifying executable vs unsupported (`➖`) cases.
4. AttemptContext anti-collision state machine (`setup` -> `target` -> `teardown`).
5. Decoupled `KmipDriver` interface (mock driver for standalone execution, ready for KMIPKit).
6. Native HTML/Markdown/ANSI graphical matrix reporter (`✅` / `❌` / `➖`).
7. Isolated GitHub Actions workflow `.github/workflows/interop-rust.yml`.

## Requirements

| ID | Requirement | Normative source / project decision | Verification |
|---|---|---|---|
| KMIPKIT-0059-FR-001 | The Rust validator SHALL declare its own independent Cargo workspace and build independently of KMIPKit crates. | Project build isolation decision | `cargo build` and `cargo test` in `interop/rust-kmip-validator/` |
| KMIPKIT-0059-FR-002 | The suite SHALL load `server-capabilities.json` and match against the test catalog to classify cases as executable or unsupported (`➖`). | Project interoperability architecture decision | Unit tests in `interop/rust-kmip-validator/src/matcher.rs` |
| KMIPKIT-0059-FR-003 | An attempt context SHALL generate unique object names and batch IDs per attempt and enforce setup/teardown isolation. | Project anti-collision decision | Unit tests in `interop/rust-kmip-validator/src/isolation.rs` |
| KMIPKIT-0059-FR-004 | The CLI SHALL support `run`, `plan`, and `render` commands and output `results.json`, `report.html`, and `report.md`. | Project CLI & visual reporting decision | Integration tests in `interop/rust-kmip-validator/tests/cli_tests.rs` |
| KMIPKIT-0059-FR-005 | A dedicated GitHub Actions workflow `.github/workflows/interop-rust.yml` SHALL validate formatting, clippy, and tests for the Rust validator. | Project CI automation decision | Local workflow check and CI execution |
