# Feature Specification: Local Cosmian KMS Integration Tests

**Feature Branch**: `feature/KMIPKIT-0015-cosmian-kms-integration`
**Created**: 2026-10-08
**Status**: Approved for implementation by the maintainer on 2026-10-09
**Input**: Expand the local Cosmian KMS integration coverage to the KMIP 2.1 operations with typed Rust client support in `release/1.0.0`.

## Scope and normative sources

This feature provides opt-in Rust integration tests for KMIPKit's production client against a locally run, version-pinned Cosmian KMS. The KMS exposes its binary TTLV KMIP socket with mutually authenticated TLS. The ignored target contains typed test cases for ten operations: Discover Versions, Create, Create Key Pair, Create Split Key, Add Attribute, Delete Attribute, Get Attribute List, Get Attributes, Modify Attribute, and Set Attribute. The pinned Cosmian 5.28.0 parser rejects Adjust Attribute as an unsupported operation, so it is excluded from the supported-operation test target and recorded as an observed server limitation. Object creation and attribute changes use only server-generated test objects in the container-local database, which is removed with the container. Tests assert successful typed results where the operation reaches a supported server path; observed blockers and server rejections are recorded without being reported as successes. The raw-TLS transport validates the response frame as a Response Message. Passing these selected tests is interoperability evidence for those operation inputs and this transport path; it is not a claim of full KMIP 2.1 or profile conformance.

Normative operation payloads and responses are OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213 (Discover Versions); §6.1.8, Tables 186–188 (Create); §6.1.9, Tables 189–192 (Create Key Pair); §6.1.10, Tables 193–195 (Create Split Key); §6.1.2, Tables 167–169 (Add Attribute); §6.1.13, Tables 202–204 (Delete Attribute); §6.1.20, Tables 223–225 (Get Attributes); §6.1.21, Tables 226–228 (Get Attribute List); §6.1.34, Tables 265–267 (Modify Attribute); and §6.1.51, Tables 322–324 (Set Attribute). Adjust Attribute (§6.1.3, Tables 170–172) was probed but is excluded because Cosmian 5.28.0 rejects it as unsupported. Protocol Version encoding is §9.16, Table 421. The response-frame root is the Response Message Structure in §8.4, Table 397, and its TTLV tag assignment is §11.56. Tests use the existing KMIPKit typed production client and TLS 1.3/mTLS policy. They add no public API or server implementation changes and do not use the public demo service.

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

### User Story 3 - Exercise supported typed operations (Priority: P1)

As a KMIPKit contributor, I can run opt-in tests for each operation with a typed Rust client path in `release/1.0.0`, so I can see which requests Cosmian accepts and whether KMIPKit parses the typed results correctly.

**Independent Test**: Start the pinned local server and explicitly run the ignored `cosmian_kms` integration target. Each operation test creates a fresh server-side object when needed, sends one typed operation at a time, and checks a successful result and required response fields.

**Acceptance Scenarios**:

1. **Given** Cosmian is available over the configured mTLS socket, **When** the Create, Create Key Pair, or Create Split Key test runs, **Then** KMIPKit encodes that typed request and validates the successful operation-specific response.
2. **Given** a fresh test key exists, **When** an attribute operation test runs, **Then** it sends the selected typed request and validates the response identifier and requested attribute result where applicable.
3. **Given** a request is rejected by Cosmian or its response is malformed, **When** the integration test runs, **Then** the test fails with the operation and KMIP result status/reason visible, without printing credentials, key material, or raw TTLV bodies.
4. **Given** ordinary workspace test commands run, **When** the live integration target is discovered, **Then** every Cosmian test remains ignored unless explicitly selected.

## Requirements

### Functional Requirements

- **KMIPKIT-0015-FR-001**: The local deployment MUST pin the Cosmian KMS image to version `5.28.0`, enable its binary TTLV KMIP socket on port `5696`, and publish the socket only on host loopback. It MUST use the existing KMIPKit raw-TLS transport path, not the Cosmian JSON TTLV HTTP endpoint.
- **KMIPKIT-0015-FR-002**: The deployment MUST generate a test-only CA, a `localhost` server certificate and a client certificate at runtime. Generated private keys and certificate output MUST remain ignored local artifacts and MUST NOT be committed.
- **KMIPKIT-0015-FR-003**: The Discover Versions test MUST use the existing typed production `Client`, assert `Result Status = Success`, and require Protocol Version `(2, 1)` in the typed result.
- **KMIPKIT-0015-FR-004**: Every live-server test MUST be opt-in and ignored by default. Documentation MUST give explicit commands to start and stop the local server and to run the integration suite or one selected operation test.
- **KMIPKIT-0015-FR-005**: Integration tests MAY create server-generated test objects and change their metadata only in the isolated Cosmian test database. They MUST NOT import caller-supplied key material or destroy objects. The container MUST have no persistent database mount, and deployment commands MUST remove the container and its state.
- **KMIPKIT-0015-FR-006**: The implementation MUST record the actual live-test result and MUST NOT claim a successful interoperability result before the test passes.
- **KMIPKIT-0015-FR-007**: The raw-TLS response reader MUST require the KMIP Response Message TTLV tag `0x42007B` defined by OASIS KMIP Specification v2.1 §8.4, Table 397, and §11.56. It MUST reject the Request Message tag `0x420078` on the response path.
- **KMIPKIT-0015-FR-008**: The ignored Rust integration target MUST define an operation-specific test case for each of the ten operations listed in Scope using `ClientRequest` or the existing typed client convenience methods. It MUST check successful typed responses and required operation-specific fields when the operation reaches a supported server path, without hand-encoding TTLV or logging sensitive values. Any blocked setup, request, or server rejection MUST be documented as such and MUST NOT be reported as an operation pass.
- **KMIPKIT-0015-FR-009**: Each attribute integration case MUST operate on a fresh object created by that test or on a separate test-owned object, so the tests do not rely on test order or shared identifiers.
- **KMIPKIT-0015-FR-010**: Documentation and traceability MUST identify the exact KMIP v2.1 section and tables for every exercised operation and record the observed live result for each operation individually.

### Out of Scope

- Any operation beyond the ten listed in Scope, including Adjust Attribute (rejected as unsupported by the pinned parser), Get, Locate, lifecycle operations, and cryptographic operations.
- Using or sending requests to `demo-kms.cosmian.dev`.
- HTTPS/JSON TTLV integration, formal OASIS profile testing, certification, multi-server orchestration, or CI-hosted KMS deployment.
- Changes to production protocol, transport, public APIs, release boundaries or the immutable OASIS source copies.

## Normative Traceability

| Requirement | Source | Verification |
|---|---|---|
| `KMIPKIT-0015-FR-003` | OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213; §9.16, Table 421 | `crates/kmipkit-client/tests/cosmian_kms.rs` sends typed Discover Versions and checks success plus returned `(2, 1)`. |
| `KMIPKIT-0015-FR-008`, `KMIPKIT-0015-FR-009`, `KMIPKIT-0015-FR-010` | OASIS KMIP Specification v2.1 sections/tables listed under Scope | One ignored typed integration test per operation in `crates/kmipkit-client/tests/cosmian_kms.rs`; per-operation status and required response fields recorded in `traceability.md`. |
| `KMIPKIT-0015-FR-001`, `KMIPKIT-0015-FR-002`, `KMIPKIT-0015-FR-004`, `KMIPKIT-0015-FR-005`, `KMIPKIT-0015-FR-006` | KMIPKit project constraints in `AGENTS.md` §§2, 6, 8; local integration policy | Compose configuration, generated-PKI script, ignored test gate, test docs, and live-test records. These are project requirements, not OASIS clauses. |
| `KMIPKIT-0015-FR-007` | OASIS KMIP Specification v2.1 §8.4, Table 397; §11.56 | `crates/kmipkit-transport/src/raw_tls.rs`; valid and wrong-root response frames in `crates/kmipkit-transport/tests/raw_tls.rs`; the live Cosmian response. |

## Success Criteria

- **KMIPKIT-0015-SC-001**: One documented command starts a loopback-only Cosmian KMS 5.28.0 KMIP socket using newly generated local mTLS credentials.
- **KMIPKIT-0015-SC-002**: Ordinary Rust test runs skip every live integration test; an explicit ignored-test command executes the Cosmian operation suite.
- **KMIPKIT-0015-SC-003**: Successful assertions are limited to the ten selected typed operation inputs and responses; they do not imply support for other operations or formal conformance.
- **KMIPKIT-0015-SC-004**: Each live operation result is recorded separately, and the documentation does not present selected interoperability tests as full server conformance.

## Assumptions

- The integration is based on `release/1.0.0`, which contains the production raw-TLS transport and the typed Rust client paths for the ten operations in Scope.
- Contributors have Docker Compose and Rust 1.94 or later. The PKI is generated inside a short-lived container, so host OpenSSL is not required.
- Cosmian KMS 5.28.0 supports the configuration documented at its matching source tag, including PEM TLS files, client-CA verification and the binary KMIP socket.
