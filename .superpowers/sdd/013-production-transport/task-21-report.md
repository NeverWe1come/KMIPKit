# T021 Red report — KMIPKIT-0013

## Scope

This is the Red-only test change for T021 on branch
feature/KMIPKIT-0013-production-transport-continuation, based on
5d0c19a6dede98747f657eb6bad21f8c8e9c1b68. No production source was changed.

The tests trace to KMIPKIT-0013 FR-004, FR-005, and SC-006; TLS Configuration
contract sections Construction and Secret handling; ADR-0014; and threat model
§2.

## Added coverage

- secret_redaction_current runs the nine cases supported by the current
  public API: private-key Debug redaction, endpoint and target path/query
  redaction, sanitized invalid-key diagnostics, encrypted/ambiguous/empty/
  malformed/mismatched key rejection, and DNS/path/dependency sentinel removal
  from the transport error Debug/Display/source chain.
- secret_redaction adds the not-yet-implemented file-source and secret-owner
  contracts. Its file tests use temporary local files, explicit PEM and DER
  selection, normal path resolution, symlink paths when the OS permits them,
  path-redacted errors, and removal of source paths before configuration build
  to prove they are not retained for later reads.
- The key-owner cases observe only initialized length and a zeroized boolean
  for successful construction, encrypted/ambiguous/empty/malformed keys,
  certificate/key mismatch, malformed certificates, early configuration
  rejection, and trust-loading rejection. The observer never returns bytes and
  is crate-internal test support, not a public API or feature.
- The test values are synthetic sentinels and ephemeral local PKI fixtures.
  No real credentials or network access are used.

## Contract assumptions for T022

The file tests assume CertificateInput and PrivateKeyInput add
from_pem_file(path) and from_der_file(path) constructors returning
Result<Self, TransportConfigError>. Each constructor reads the selected path
once into owned input and does not retain the path. The contract test compiles
the crate's config/TLS/secret source modules locally so a cfg(test)-only
SecretBufferObserver and
PrivateKeyInput::from_pem_with_observer_for_test(bytes, observer) can inspect
only initialized length and zeroization status. These observer hooks must
remain crate-internal and test-only.

The transport crate has no logging dependency or logger entry point today.
The current-API cases verify the Debug/Display/error formatting that may feed
diagnostics, but do not claim runtime log capture coverage. Adapter logging
tests need a real logging surface before they can be added.

## Red evidence

- cargo test -p kmipkit-transport --test secret_redaction --offline exits 1
  during compilation because crates/kmipkit-transport/src/secret.rs is
  intentionally absent until T022. This is the planned compile-fail boundary
  for the new secret-owner observer and file-input contract.
- cargo test -p kmipkit-transport --test secret_redaction_current --offline
  passes: 9 passed, 0 failed.
- rustfmt --edition 2024 --config skip_children=true --check
  crates/kmipkit-transport/tests/secret_redaction.rs
  crates/kmipkit-transport/tests/secret_redaction_current.rs
  crates/kmipkit-transport/tests/support/secret_redaction/current.rs
  crates/kmipkit-transport/tests/support/secret_redaction/file_sources.rs
  crates/kmipkit-transport/tests/support/secret_redaction/zeroization.rs
  passes with exit 0.
- cargo fmt --all --check cannot resolve the same T022-owned
  src/secret.rs module. Focused rustfmt is the formatting evidence for this
  Red phase.
- git diff --check passes.

### Encrypted-key sentinel correction

The current-API encrypted-key case includes a dedicated, synthetic PEM-body
sentinel and checks that it is absent from error Display, Debug, and the public
source chain. Assertion messages are static and do not print the sentinel.
Correction commit: da52647f728032cd2c81fc78057e550e0900f73c. Verification:
`cargo test -p kmipkit-transport --test secret_redaction_current --offline`
passed 9/9; focused rustfmt for `secret_redaction_current.rs` and
`support/secret_redaction/current.rs` passed; `git diff --check` passed.

## Commit

Red commit: c00556281a1399dbf7281306591a39a9eb13db6b. The follow-up evidence
commit records this ID in the T021 task row.
