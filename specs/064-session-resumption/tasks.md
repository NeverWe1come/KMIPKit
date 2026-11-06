# Tasks: TLS 1.3 Session Resumption and HTTPS Keep-Alive Reconnect (KMIPKIT-0064)

## Phase 1: RED (Failing Tests)

- [ ] `T001`: Add failing integration test in `crates/kmipkit-transport/tests/raw_tls.rs` asserting consecutive exchanges with `RawTlsTransport` perform TLS 1.3 session resumption (`HandshakeKind::Resumed`).
- [ ] `T002`: Add failing integration test in `crates/kmipkit-transport/tests/https.rs` asserting transparent reconnect when an idle cached Keep-Alive connection was closed by the peer before dispatch.
- [ ] `T003`: Add failing test asserting no reconnect is attempted if failure occurs after dispatch commit (`commit_dispatch()`).
- [ ] `T004`: Add failing unit test asserting 0-RTT early data is disabled on client configuration.

## Phase 2: GREEN (Implementation)

- [ ] `T005`: Implement transparent reconnect logic in `crates/kmipkit-transport/src/https.rs` (`exchange_on_worker` / `acquire_https_connection`) when a reused connection encounters an error before dispatch commit.
- [ ] `T006`: Ensure `RawTlsTransport` properly processes post-handshake TLS session tickets and performs resumption on subsequent exchanges.
- [ ] `T007`: Verify `enable_early_data` is explicitly false and session store limits are respected.

## Phase 3: REFACTOR & Verification

- [ ] `T008`: Refactor and harden reconnect error mappings and connection cleanup.
- [ ] `T009`: Run `cargo fmt --all --check`.
- [ ] `T010`: Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] `T011`: Run `cargo test --workspace --all-features`.
- [ ] `T012`: Verify `python3 -B tools/api_manifest/generate.py --repo-root . --check` confirms 0% API manifest changes.
