# Tasks: Standalone Multi-Version KMIP Test Server (KMIPKIT-0065)

- [ ] `TASK-065-001`: Workspace configuration and package scaffold (`crates/kmipkit-test-server/Cargo.toml`, `Cargo.toml`).
- [ ] `TASK-065-002`: Auto-PKI generator with `rcgen` (`src/pki.rs`) exporting CA, server cert, client cert/key PEM.
- [ ] `TASK-065-003`: File-based JSON storage engine (`src/store/`) persisting managed objects and metadata to disk.
- [ ] `TASK-065-004`: Multi-version router and normalizer (`src/v1_compat.rs`) bridging 1.0-1.4 to 2.1 representations.
- [ ] `TASK-065-005`: Multi-format codec layer (`src/codec/`) handling TTLV 8-byte framing, JSON, and XML.
- [ ] `TASK-065-006`: Core operation implementations and realistic crypto engine (`src/ops/`) using `aws-lc-rs`.
- [ ] `TASK-065-007`: Raw TLS 1.3 mTLS listener and HTTPS/1.1 listener (`src/transport/`).
- [ ] `TASK-065-008`: Standalone CLI binary (`src/main.rs`) with `--listen`, `--data-dir`, `--export-pki`, `--clean`, `--print-env`.
- [ ] `TASK-065-009`: Integration test suite and test harness support (`crates/kmipkit-test-support/src/lib.rs`).
- [ ] `TASK-065-010`: Verification passes (formatting, clippy, unit tests, integration tests, 0% API drift).
