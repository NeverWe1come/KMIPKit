# Feature Specification: Local Cosmian KMS Integration Smoke Test

**Feature Branch**: `feature/KMIPKIT-0015-cosmian-kms-integration`
**Created**: 2026-10-08
**Status**: Approved for preparatory implementation by the maintainer
**Input**: Prepare a local, reproducible Cosmian KMS deployment and an opt-in KMIPKit integration test while KMIPKIT-0013 completes its coverage gate.

## Scope and normative sources

This feature adds test infrastructure for the production client and fixes the response-frame tag check exposed by the first live run. A locally run, version-pinned Cosmian KMS exposes its binary TTLV KMIP socket with mutually authenticated TLS. An ignored Rust integration test sends the existing typed Discover Versions request and requires the server to report KMIP 2.1. The raw-TLS transport validates the response frame as a Response Message. This is an interoperability smoke test for that operation and transport path; it is not a claim of full KMIP 2.1 or profile conformance.

The normative operation and payload are OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213; Protocol Version encoding is §9.16, Table 421. The response-frame root is the Response Message Structure in §8.4, Table 397, and its TTLV tag assignment is §11.56. The test requires the server's successful response to include the version `(2, 1)` offered by KMIPKit. The test transport uses KMIPKit's existing TLS 1.3/mTLS policy. It does not introduce a public API change, test additional operations, or use the public demo service.

## User Scenarios & Testing

### User Story 1 - Start an isolated local KMIP server (Priority: P1)

As a KMIPKit contributor, I can start the pinned Cosmian KMS locally with disposable TLS credentials and isolated storage, so I can later run an integration test without external accounts or services.

**Independent Test**: Start the documented compose deployment and observe the Cosmian KMS container listening on the loopback-published KMIP socket. No KMIP integration test is required to start it.

**Acceptance Scenarios**:

1. **Given** Docker is available, **When** I start the integration deployment, **Then** it generates a local CA, server certificate and client certificate and starts Cosmian KMS 5.28.0 with its binary KMIP listener on host loopback port 5696.
2. **Given** the deployment is stopped and removed, **When** I inspect the repository, **Then** generated private keys and local server data are absent from tracked files.

### User Story 2 - Check the KMIPKit-to-Cosmian 2.1 path (Priority: P1)

As a KMIPKit contributor, I can explicitly run an ignored integration test against that deployment, so I can verify the production client, mTLS transport, TTLV exchange, typed result parsing and KMIP 2.1 interoperability together.

**Independent Test**: Run the documented ignored integration test after starting the local server. It sends one Discover Versions request and asserts a successful operation result containing Protocol Version `(2, 1)`.

**Acceptance Scenarios**:

1. **Given** the local Cosmian KMIP socket is available and generated credentials are valid, **When** the opt-in test executes, **Then** the typed KMIPKit client completes Discover Versions successfully and the result lists `(2, 1)`.
2. **Given** ordinary workspace test commands run, **When** this integration target is discovered, **Then** the live-server test remains ignored and sends no request unless explicitly selected.
3. **Given** the local server is unavailable or TLS credentials are invalid, **When** a contributor explicitly runs the test, **Then** the test fails with a useful local setup/configuration error and does not retry or fall back to another server.
4. **Given** a raw-TLS response frame, **When** KMIPKit reads its TTLV root, **Then** it accepts the OASIS Response Message tag `0x42007B` and rejects a Request Message tag `0x420078`.

## Requirements

### Functional Requirements

- **KMIPKIT-0015-FR-001**: The local deployment MUST pin the Cosmian KMS image to version `5.28.0`, enable its binary TTLV KMIP socket on port `5696`, and publish the socket only on host loopback. It MUST use the existing KMIPKit raw-TLS transport path, not the Cosmian JSON TTLV HTTP endpoint.
- **KMIPKIT-0015-FR-002**: The deployment MUST generate a test-only CA, a `localhost` server certificate and a client certificate at runtime. Generated private keys and certificate output MUST remain ignored local artifacts and MUST NOT be committed.
- **KMIPKIT-0015-FR-003**: The integration test MUST use the existing typed production `Client` and an explicit `Discover Versions` request. It MUST assert `Result Status = Success` and the presence of Protocol Version `(2, 1)` in the typed result.
- **KMIPKIT-0015-FR-004**: The live-server test MUST be opt-in and ignored by default. Documentation MUST give explicit commands to start and stop the local server and to run only this integration test.
- **KMIPKIT-0015-FR-005**: The test MUST NOT create, import, modify or destroy cryptographic objects. The container MUST have no persistent database mount, and deployment commands MUST support removing the test container and its state.
- **KMIPKIT-0015-FR-006**: The implementation MUST record the actual live-test result and MUST NOT claim a successful interoperability result before the test passes.
- **KMIPKIT-0015-FR-007**: The raw-TLS response reader MUST require the KMIP Response Message TTLV tag `0x42007B` defined by OASIS KMIP Specification v2.1 §8.4, Table 397, and §11.56. It MUST reject the Request Message tag `0x420078` on the response path.

### Out of Scope

- Any operation beyond Discover Versions.
- Using or sending requests to `demo-kms.cosmian.dev`.
- HTTPS/JSON TTLV integration, formal OASIS profile testing, certification, multi-server orchestration, or CI-hosted KMS deployment.
- Changes to production protocol, transport, public APIs, release boundaries or the immutable OASIS source copies.

## Normative Traceability

| Requirement | Source | Verification |
|---|---|---|
| `KMIPKIT-0015-FR-003` | OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213; §9.16, Table 421 | `crates/kmipkit-client/tests/cosmian_kms.rs` sends the typed request offering 2.1 and checks success plus returned `(2, 1)`. |
| `KMIPKIT-0015-FR-001`, `KMIPKIT-0015-FR-002`, `KMIPKIT-0015-FR-004`, `KMIPKIT-0015-FR-005`, `KMIPKIT-0015-FR-006` | KMIPKit project constraints in `AGENTS.md` §§2, 6, 8; local integration policy | Compose configuration, generated-PKI script, opt-in test gate, test docs, and branch verification record. These are project requirements, not OASIS clauses. |
| `KMIPKIT-0015-FR-007` | OASIS KMIP Specification v2.1 §8.4, Table 397; §11.56 | `crates/kmipkit-transport/src/raw_tls.rs`; valid and wrong-root response frames in `crates/kmipkit-transport/tests/raw_tls.rs`; the live Cosmian response. |

## Success Criteria

- **KMIPKIT-0015-SC-001**: One documented command starts a loopback-only Cosmian KMS 5.28.0 KMIP socket using newly generated local mTLS credentials.
- **KMIPKIT-0015-SC-002**: Ordinary Rust test runs skip the integration test; an explicit ignored-test command executes only the single Discover Versions exchange.
- **KMIPKIT-0015-SC-003**: The test's successful assertion is limited to an operation success and returned KMIP 2.1; it does not imply support for other operations or formal conformance.
- **KMIPKIT-0015-SC-004**: The live integration test runs only when explicitly selected, and its actual result is recorded without presenting one successful operation as full server conformance.

## Assumptions

- The integration is based on the user's authorized KMIPKIT-0013 continuation branch, which contains the production raw-TLS client API.
- Contributors have Docker Compose and Rust 1.94 or later. The PKI is generated inside a short-lived container, so host OpenSSL is not required.
- Cosmian KMS 5.28.0 supports the configuration documented at its matching source tag, including PEM TLS files, client-CA verification and the binary KMIP socket.
