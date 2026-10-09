# Implementation Plan: Local Cosmian KMS Integration Tests

**Branch**: `feature/KMIPKIT-0015-cosmian-kms-integration` | **Date**: 2026-10-08 | **Spec**: [spec.md](spec.md)

**Input**: [Feature specification](spec.md)

## Summary

Provide opt-in interoperability tests for the ten KMIP 2.1 operations in `spec.md` against a local, pinned Cosmian KMS 5.28.0 instance. Docker Compose generates disposable mTLS credentials, configures the binary KMIP TTLV socket on loopback, and stores server data only in the disposable container. Retain the raw-TLS Response Message root validation established by the first live run. Exercise each operation through the current typed Rust production client, assert typed response contracts when supported, and record observed blockers without logging secret values.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94; PowerShell 7-compatible runner; POSIX shell in the cert-generation container.

**Primary Dependencies**: Existing `kmipkit-client`, `kmipkit-protocol`, `kmipkit-transport`; Docker Compose; `alpine:3.22` with OpenSSL for test PKI; `ghcr.io/cosmian/kms:5.28.0`.

**Storage**: Cosmian SQLite database under the container filesystem, cleared at server startup and removed with the container; generated PKI in ignored `.local/cosmian-kms/certs`.

**Testing**: Opt-in integration target `crates/kmipkit-client/tests/cosmian_kms.rs`; it contains one ignored test per operation. Run only after starting the local pinned Cosmian KMS 5.28.0 deployment.

**Target Platform**: Local Docker Desktop or Docker Engine, with Rust 1.94+ host.

**Project Type**: Existing Rust workspace plus local integration-test infrastructure.

**Constraints**: No production API changes; no public demo access; no import or object-destroy operations; test-generated objects and attribute changes stay in container-local storage; port 5696 bound only to 127.0.0.1; no automatic retries; no committed credentials; every live test remains ignored.

**Scale/Scope**: One local server; typed tests for Discover Versions, Create, Create Key Pair, Create Split Key, and six attribute operations. Each test owns its object state and uses one configured KMIPKit client.

## Constitution Check

- **Specification and traceability**: Pass for this bounded scope. `spec.md` identifies KMIP v2.1 §6.1.16 Tables 211–213 and §9.16 Table 421, and `traceability.md` links the assertions to the test.
- **Test first and evidence**: The user subsequently authorized the live test. Its first run reached Cosmian over verified mTLS and failed because the response tag check rejected Cosmian's valid `0x42007B` Response Message frame. A focused negative regression test reproduced the defect on the old implementation. After correcting the response tag, all 90 raw-TLS tests passed, all 14 production-client tests passed (with test-support fixtures enabled), and the live test passed (1/1). Keep Red, Green and Refactor evidence in distinct development commits before feature PR review.
- **Core and protocol boundaries**: Pass. The change only calls the existing typed client; no new protocol/API behavior or operation is introduced.
- **Security**: Pass by design, subject to verification. Test-only certificates are generated locally and ignored; loopback-only bind; no persistent data mount; no request retries; the operation is read-only.
- **Human-governed workflow**: The user explicitly authorized this feature work to start from the KMIPKIT-0013 continuation branch. It does not push, merge, approve, or release.

## Design

1. Keep the version-pinned KMS configuration and Compose manifest in `tests/integration/cosmian/`.
2. Generate CA/server/client credentials into ignored `.local/cosmian-kms/certs/`; the server certificate carries `DNS:localhost` SAN and `serverAuth`, and the client certificate carries `clientAuth`.
3. Publish `5696` only on `127.0.0.1`; do not publish the unused HTTP UI or HTTP JSON TTLV port.
4. Extend the ignored Cargo test target with a shared mTLS client builder, server-generated test-object helper, and one named test per operation. Use the typed KMIPKit request and response APIs; create separate objects for each attribute test and assert operation success plus operation-specific result fields where the server supports the operation.
5. Use independently generated server-side RSA public keys for object-dependent attribute tests because Cosmian accepts the attribute operations for those objects. Use Create Key Pair for its own test and send an XOR two-of-two Create Split Key request over a test key. The initial test run exposed a Create Attributes tag defect and an incorrect Create Split Key operation enumeration; both candidate fixes are tracked separately under KMIPKIT-0014.
6. Retain Response Message root validation as defined by OASIS KMIP v2.1 §8.4, Table 397, and §11.56.
7. Update the English and Spanish-facing integration documentation and requirement traceability with the exact OASIS section/table and observed result for every operation.
8. Run the full opt-in suite against the local deployment; record failed KMIP Result Status/Reason without treating server rejection as transport success or weakening typed parsing.

## Project Structure

```text
crates/kmipkit-client/tests/cosmian_kms.rs
tests/integration/cosmian/
├── compose.yaml
├── cosmian-kms.toml
├── generate-pki.sh
└── README.md
scripts/integration/cosmian-kms.ps1
specs/015-cosmian-kms-integration/
├── plan.md
├── research.md
├── quickstart.md
├── spec.md
├── tasks.md
└── traceability.md
```

## Verification Plan

- Static/configuration-only checks may validate formatting, PowerShell syntax, shell syntax, and `docker compose config`.
- Focused raw-TLS adapter tests must accept Response Message tag `0x42007B` and reject Request Message tag `0x420078` on the response path.
- The live run uses `.\scripts\integration\cosmian-kms.ps1 -Action test`, which invokes `cargo test -p kmipkit-client --test cosmian_kms -- --ignored --test-threads=1`; retain per-operation results and the Cosmian image digest as evidence.
- Run `cargo fmt --all --check`, focused client and transport tests, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and `cargo test --workspace --all-features --locked --quiet` after synchronization and test changes.
- Do not claim broader KMIP 2.1 interoperability or profile conformance from the selected operations.

## Risks

- Cosmian may reject an operation because its version, configuration, authorization, or object-specific constraints differ from the supported-operation matrix; retain each observed response precisely and report the limitation.
- The initial Create request builder used the Attribute tag where KMIP requires the Attributes structure tag, and Create Split Key used Register's operation enumeration. Candidate fixes are in the separate KMIPKIT-0014 branch; do not merge those production changes into this integration-test specification.
- A typed response parser may expose a defect only when a real server returns a value shape not represented by unit fixtures. Any production behavior fix remains outside this integration-test specification and requires its own approved scope.
- The local OpenSSL generator and Compose bind mount must be validated on supported developer platforms.
- Docker registry availability is an external prerequisite; this test is not suitable as an always-on offline workspace test.
