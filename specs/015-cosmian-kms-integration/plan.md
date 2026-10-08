# Implementation Plan: Local Cosmian KMS Integration Smoke Test

**Branch**: `feature/KMIPKIT-0015-cosmian-kms-integration` | **Date**: 2026-10-08 | **Spec**: [spec.md](spec.md)

**Input**: [Feature specification](spec.md)

## Summary

Provide an opt-in interoperability smoke test for the existing typed KMIP 2.1 Discover Versions path against a local, pinned Cosmian KMS 5.28.0 instance. Docker Compose generates disposable mTLS credentials, configures the binary KMIP TTLV socket on loopback, and stores server data only in the disposable container. The first live run exposed that the raw-TLS adapter expected the Request Message root tag on responses; correct the adapter to require the Response Message tag defined by OASIS KMIP v2.1 §8.4, Table 397, and §11.56.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94; PowerShell 7-compatible runner; POSIX shell in the cert-generation container.

**Primary Dependencies**: Existing `kmipkit-client`, `kmipkit-protocol`, `kmipkit-transport`; Docker Compose; `alpine:3.22` with OpenSSL for test PKI; `ghcr.io/cosmian/kms:5.28.0`.

**Storage**: Cosmian SQLite database under the container filesystem, cleared at server startup and removed with the container; generated PKI in ignored `.local/cosmian-kms/certs`.

**Testing**: One opt-in integration target `crates/kmipkit-client/tests/cosmian_kms.rs`; executed successfully against the local pinned Cosmian KMS 5.28.0 deployment after correcting a response-root tag defect in the raw-TLS transport.

**Target Platform**: Local Docker Desktop or Docker Engine, with Rust 1.94+ host.

**Project Type**: Existing Rust workspace plus local integration-test infrastructure.

**Constraints**: No production API changes; no public demo access; no key lifecycle operations; port 5696 bound only to 127.0.0.1; no automatic retries; no committed credentials; live test remains ignored.

**Scale/Scope**: One server, one client, one typed request, one response, one returned version.

## Constitution Check

- **Specification and traceability**: Pass for this bounded scope. `spec.md` identifies KMIP v2.1 §6.1.16 Tables 211–213 and §9.16 Table 421, and `traceability.md` links the assertions to the test.
- **Test first and evidence**: The user subsequently authorized the live test. Its first run reached Cosmian over verified mTLS and failed because the response tag check rejected Cosmian's valid `0x42007B` Response Message frame. A focused negative regression test reproduced the defect on the old implementation. After correcting the response tag, all 90 raw-TLS tests passed, all 14 production-client tests passed (with test-support fixtures enabled), and the live test passed (1/1). Keep Red, Green and Refactor evidence in distinct development commits before feature PR review.
- **Core and protocol boundaries**: Pass. The change only calls the existing typed client; no new protocol/API behavior or operation is introduced.
- **Security**: Pass by design, subject to verification. Test-only certificates are generated locally and ignored; loopback-only bind; no persistent data mount; no request retries; the operation is read-only.
- **Human-governed workflow**: The user explicitly authorized this feature work to start from the KMIPKIT-0013 continuation branch. It does not push, merge, approve, or release.

## Design

1. Put a version-pinned KMS configuration and Compose manifest in `tests/integration/cosmian/`.
2. Generate CA/server/client credentials into the workspace's ignored `.local/cosmian-kms/certs/` bind mount before starting KMS. The server certificate carries `DNS:localhost` SAN and `serverAuth`; the client certificate carries `clientAuth`.
3. Publish `5696` only on `127.0.0.1`. Do not publish the unused HTTP UI or HTTP JSON TTLV port.
4. Add an ignored Cargo integration test which reads the generated PEM files, creates the existing raw-TLS client configuration, sends typed Discover Versions, and checks successful status plus version `(2, 1)`.
5. Validate the response frame root as the OASIS Response Message Structure; reject a Request Message root on the response path.
6. Provide a PowerShell helper and English quickstart with start, status, logs, stop, cleanup, and explicit test commands.
7. Record exact live test outcomes and resolve observed protocol incompatibilities without weakening the assertions.

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
- The live run used `.\scripts\integration\cosmian-kms.ps1 -Action test`, which invokes `cargo test -p kmipkit-client --test cosmian_kms -- --ignored`; retain the observed result and Cosmian image digest as evidence.
- Do not claim broader KMIP 2.1 interoperability or profile conformance from this single operation.

## Risks

- Later KMIP operations could return response shapes that the typed result parser does not yet support; expand coverage only through separately scoped tests without widening this one-operation smoke test silently.
- The local OpenSSL generator and Compose bind mount must be validated on supported developer platforms.
- Docker registry availability is an external prerequisite; this test is not suitable as an always-on offline workspace test.
