# Tasks: File-backed TLS Hot-Reload and Resumption Invalidation (KMIPKIT-0063)

## Phase 1: RED (Failing Tests)

- [ ] `T001`: Add failing unit test in `crates/kmipkit-transport` asserting `TlsSessionStore::clear()` purges cached tickets and resets ticket count to zero.
- [ ] `T002`: Add failing unit test in `crates/kmipkit-transport` asserting file-backed `CertificateInput` and `PrivateKeyInput` record and retain their file tracking descriptors with redacted paths.
- [ ] `T003`: Add failing integration test in `tests/raw_tls.rs` demonstrating client mTLS certificate and private key rotation on disk without client reconstruction.
- [ ] `T004`: Add failing integration test in `tests/https.rs` demonstrating credential rotation on disk invalidating active Keep-Alive connections and completing handshake under the new identity.
- [ ] `T005`: Add failing test asserting mismatched/corrupt key file during rotation fails closed as `NotSent` before network I/O and preserves zeroization.

## Phase 2: GREEN (Implementation)

- [ ] `T006`: Implement `TlsSessionStore::clear()` in `crates/kmipkit-transport/src/tls.rs`.
- [ ] `T007`: Implement `TrackedFileSource` and file tracking metadata on `CertificateInput`, `PrivateKeyInput`, `RevocationListInput`, and `TransportConfig` in `crates/kmipkit-transport/src/config.rs`.
- [ ] `T008`: Implement `TlsClientConfig::current_or_refresh()` with generation counter (`tls_generation: u64`) and atomic revalidation in `crates/kmipkit-transport/src/tls.rs`.
- [ ] `T009`: Wire refresh into `RawTlsTransport::exchange_with_options()` in `crates/kmipkit-transport/src/raw_tls.rs`.
- [ ] `T010`: Wire refresh into `HttpsTransport::exchange_with_options()` in `crates/kmipkit-transport/src/https.rs`, matching `conn.tls_generation` and aborting/reconnecting if generation advanced.

## Phase 3: REFACTOR & Verification

- [ ] `T011`: Refactor common file-reading/hashing helpers, ensuring zeroize on old key buffers and strictly redacted debug formatting.
- [ ] `T012`: Verify backward compatibility with existing tests (`cargo test -p kmipkit-transport --all-features`).
- [ ] `T013`: Verify `cargo clippy --workspace --all-targets --all-features -- -D warnings` and `cargo fmt --all --check`.
- [ ] `T014`: Confirm `specification/api/public-api.json` is completely unchanged via `python3 -B tools/api_manifest/generate.py --repo-root . --check`.
