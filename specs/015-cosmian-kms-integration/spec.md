# Feature Specification: Local Cosmian KMS Integration Tests

**Feature Branch**: `feature/KMIPKIT-0015-cosmian-operations`
**Created**: 2026-10-08
**Status**: Approved for implementation on 2026-10-09; exhaustive matrix amendment authorized on 2026-10-16; targeted multipart interoperability amendment authorized by the maintainer on 2026-10-17.
**Input**: [Approved exhaustive matrix design](../../docs/superpowers/specs/2026-10-16-cosmian-exhaustive-matrix-design.md).

## Scope and normative sources

This feature provides opt-in Rust integration tests through KMIPKit's production typed `Client` over raw TTLV/TLS 1.3/mTLS against local Cosmian KMS `5.28.0`, image `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`. The tag object is `cec59a003b9cfc4f253ca239fde42925988700fa`; its dereferenced source commit is `4ab3ee65829a903abb71f1298f400657fcee7ec9`. The versioned matrix covers every valid operation/object/format/encoding/algorithm/parameter intersection documented or implemented by this release within the operation scope below. It is not a blind Cartesian product.

The maintainer's 2026-10-17 scope amendment authorizes targeted Rust protocol changes for KMIP 2.1 cryptographic multipart input/output handling, and corrections to the Cosmian Hash multipart integration flow. Multipart response output may be absent or present where the operation tables do not require it; single-part response outputs remain required. Test requests include data blocks and reuse the first response's Correlation Value. This amendment does not authorize a new public API or changes to other protocol areas.

The published support table has **34 supported rows representing 35 operation paths** because `Set Attribute (Modify)` groups two distinct KMIP operations. The complete path set is: Activate, Add Attribute, Certify, Check, Create, Create Key Pair, Create Split Key, Decrypt, Delete Attribute, Derive Key, Destroy, Discover Versions, Encrypt, Export, Get, Get Attribute List, Get Attributes, Hash, Import, Join Split Key, Locate, MAC, MAC Verify, Modify Attribute, Query, RNG Retrieve, RNG Seed, Re-key, Re-key Key Pair, Register, Revoke, Set Attribute, Sign, Signature Verify, and Validate. Set Attribute and Modify Attribute already have separate successful typed exchanges. Query remains in the supported inventory with its known response-shape finding. Ping and Adjust Attribute are separately classified unsupported probes. Re-certify is a separate source/documentation discrepancy probe, not a 36th advertised supported path.

The historical 2026-10-15 baseline has twenty passing test cases: eighteen successful typed operation exchanges and the Query/Ping expected-limitation probes. Query omits Protection Storage Masks required by Table 283; Ping is unsupported and its error Batch Item omits Operation required by §8.6, Table 399. Adjust Attribute was separately rejected as unsupported. No new matrix combination has been run by this documentation amendment. The baseline is preserved in `traceability.md` and does not prove any additional object, format, algorithm, parameter combination, or profile conformance.

Normative operation payloads and responses are OASIS KMIP Specification v2.1 §6.1 (multipart cryptographic request/response sequence); §7.8 (Correlation Value reuse); §6.1.16, Tables 211–213 (Discover Versions); §6.1.8, Tables 186–188 (Create); §6.1.9, Tables 189–192 (Create Key Pair); §6.1.10, Tables 193–195 (Create Split Key); §6.1.2, Tables 167–169 (Add Attribute); §6.1.13, Tables 202–204 (Delete Attribute); §6.1.20, Tables 223–225 (Get Attributes); §6.1.21, Tables 226–228 (Get Attribute List); §6.1.34, Tables 265–267 (Modify Attribute); §6.1.51, Tables 322–324 (Set Attribute); §6.1.1, Tables 163–166 (Activate); §6.1.40, Tables 282–284 (Query); §6.1.36, Tables 271–272 (Ping); §6.1.17, Tables 214–216 (Encrypt); §6.1.11, Tables 196–198 (Decrypt); §6.1.24, Tables 235–237 (Hash, with Data requiredness in Table 235); §6.1.32, Tables 259–261 (MAC); §6.1.33, Tables 262–264 (MAC Verify); §6.1.55, Tables 334–336 (Sign); and §6.1.56, Tables 337–339 (Signature Verify). The cryptographic output and verification tags follow §11.56, Table 487. Adjust Attribute (§6.1.3, Tables 170–172) was probed but is excluded because Cosmian 5.28.0 rejects it as unsupported. Protocol Version encoding is §9.16, Table 421. The response-frame root is the Response Message Structure in §8.4, Table 397, and its TTLV tag assignment is §11.56. Tests use the existing KMIPKit typed production client and TLS 1.3/mTLS policy. The test target exposed and fixed KMIPKit tag assignments covered by the approved KMIPKIT-0021 operation specification. They do not use the public demo service.


### Expanded normative operation references

All sources in this section are OASIS KMIP Specification v2.1 (`[kmip-spec-v2.1]`), checked against immutable `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`. The [document hierarchy](../../docs/compliance/document-hierarchy.md) governs interpretation. Vendor sources select test combinations and cannot create KMIP requirements.

| Newly covered path | Exact OASIS clause and tables | Existing `ClientRequest` variant |
|---|---|---|
| Certify | §6.1.6, Tables 179–181 | `Certify` |
| Check | §6.1.7, Tables 183–185 | `Check` |
| Derive Key | §6.1.14, Tables 205–207 | `DeriveKey` |
| Destroy | §6.1.15, Tables 208–210 | `Destroy` |
| Export | §6.1.18, Tables 217–219 | `Export` |
| Get | §6.1.19, Tables 220–222 | `Get` |
| Import | §6.1.25, Tables 238–240 | `Import` |
| Join Split Key | §6.1.27, Tables 244–246 | `JoinSplitKey` |
| Locate | §6.1.28, Tables 247–249 | `Locate` |
| Re-key | §6.1.46, Tables 303–307 | `Rekey` |
| Re-key Key Pair | §6.1.47, Tables 310–312 | `RekeyKeyPair` |
| Register | §6.1.43, Tables 291–294 | `Register` |
| Revoke | §6.1.44, Tables 295–297 | `Revoke` |
| RNG Retrieve | §6.1.49, Tables 316–318 | `RngRetrieve` |
| RNG Seed | §6.1.50, Tables 319–321 | `RngSeed` |
| Validate | §6.1.57, Tables 340–342 | `Validate` |

The additional Re-certify discrepancy probe cites §6.1.45, Tables 298–302 and uses the existing `Recertify` variant. All 35 supported paths and the three separate probes already have typed `ClientRequest` variants in `crates/kmipkit-client/src/execute.rs`; this feature does not add operation APIs.

### Normative data-dimension registry

Each concrete matrix row must cite its operation and each applicable data dimension below. Documented valid combinations with implementation gaps remain discrepancy rows. Invalid combinations and unsupported capabilities receive source-backed exclusion reasons.

| Dimension | Exact OASIS KMIP Specification v2.1 clauses/tables | Application |
|---|---|---|
| Managed objects | §2.1, Table 2; §2.3, Table 4; §§2.5–2.9, Tables 6–10 | Certificate, Opaque Object, Private Key, Public Key, Secret Data, Split Key, Symmetric Key, intersected with operation-specific validity. Object-free operations mark the dimension inapplicable. |
| Certificate request input | §2.2, Table 3; §6.1.6, Table 179; §11.8, Table 439 | Certify CSR input where supported does not imply storage support for Certificate Request objects. |
| Key block/value and transparent material | §§3.1–3.2, Tables 11–13; §3.4, Table 18; §3.5, Table 19; §§3.7–3.8, Tables 21–22; §§3.11–3.12, Tables 25–26 | Applicable symmetric, DSA private, RSA and EC material; documented/source disagreements remain explicit. |
| Key format | §4.26, Tables 79–80; §11.25, Tables 460–461 | Separate default from explicit format selection; PKCS#12 is standard Key Format Type `0x00000016`. |
| PKCS#12 metadata/links | §4.39, Tables 107–108; §4.31, Tables 90–92; §11.28, Tables 464–465 | PrivateKey/PFX import, linked key/certificate objects and aliases/password links; secret values are never reported. |
| ASN.1 representation | §2.1, Table 2; §11.25, Table 460; §6.1.6, Table 179 | Preserve DER requirements for X.509 and DER-defined key formats. PEM/DER fixture and vendor encoding paths are recorded separately from KMIP Key Format Type and TTLV message encoding; PEM is not a Table 461 enumeration. |
| Algorithms, sizes and curves | §4.13, Tables 53–54; §4.14, Tables 55–56; §4.15, Tables 57–58; §11.12, Table 443; §11.45, Table 477 | Explicit algorithms, lengths, domain parameters and curves; vendor extensions require pinned vendor evidence without invented standard values. |
| Cryptographic parameters | §4.16, Tables 59–60; §11.6, Table 436; §11.16, Table 447; §11.21, Table 454; §11.30; §11.37, Table 471 | Applicable modes, hashes, signatures, padding, IV/nonce, AAD, tag/salt length and mask generation, only where valid. IV Length is in bits and Tag Length in bytes (§4.16); preserve these normative units. |
| Derivation | §6.1.14, Tables 205–207; §7.12, Table 364; §11.14, Tables 445–446; §4.17, Table 62 | Explicit derivation methods/parameters and applicable Derive Key usage authorization. |
| Wrapping | §3.3, Tables 14–17; §7.18, Table 370; §11.18, Tables 449–450; §11.29; §11.62, Tables 493–494 | Wrapping method/parameters, key-wrap type and No Encoding/TTLV Encoding where supported; Encoding Option does not mean PEM/DER. |


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

As a KMIPKit contributor, I can run every source-derived valid case for the 35 supported operation paths through the existing typed Rust client, inspect each outcome, and distinguish successful exchanges from incompatibilities and setup/client blockers.

**Independent Test**: Start the pinned local server and explicitly run the ignored `cosmian_kms` integration target. Each case provisions its own state and reports its stable matrix ID. Supported paths assert successful typed results with required fields; Query and separate probes retain their actual finding classifications. All object, format, encoding, algorithm and parameter cases have individual outcomes; passing a finding assertion does not establish interoperability.

**Acceptance Scenarios**:

1. **Given** Cosmian is available over the configured mTLS socket, **When** the Create, Create Key Pair, or Create Split Key test runs, **Then** KMIPKit encodes that typed request and validates the successful operation-specific response.
2. **Given** a fresh test key exists, **When** an attribute operation test runs, **Then** it sends the selected typed request and validates the response identifier and requested attribute result where applicable.
3. **Given** Query or Ping reaches a known Cosmian limitation, **When** its test runs, **Then** it checks the sanitized result and retained response shape and records the limitation without presenting it as successful interoperability or printing credentials, key material, or raw TTLV bodies.
4. **Given** ordinary workspace test commands run, **When** the live integration target is discovered, **Then** every live Cosmian test remains ignored; offline inventory validators run without network access.
5. **Given** an Import, Destroy or linked-object case runs, **When** it provisions and cleans up state, **Then** it uses only its own test-generated fixtures and unique objects, without relying on another case.
6. **Given** any complete or selected live run finishes, including a failure, **When** teardown executes, **Then** Compose services and disposable database state are removed and status shows no running service.

## Requirements

### Functional Requirements

- **KMIPKIT-0015-FR-001**: The local deployment MUST pin the Cosmian KMS image to version `5.28.0`, enable its binary TTLV KMIP socket on port `5696`, and publish the socket only on host loopback. It MUST use the existing KMIPKit raw-TLS transport path, not the Cosmian JSON TTLV HTTP endpoint.
- **KMIPKIT-0015-FR-002**: The deployment MUST generate a test-only CA, a `localhost` server certificate and a client certificate at runtime. Generated private keys and certificate output MUST remain ignored local artifacts and MUST NOT be committed.
- **KMIPKIT-0015-FR-003**: The Discover Versions test MUST use the existing typed production `Client`, assert `Result Status = Success`, and require Protocol Version `(2, 1)` in the typed result.
- **KMIPKIT-0015-FR-004**: Every live-server test MUST be opt-in and ignored by default. Documentation MUST give explicit commands to start and stop the local server and to run the integration suite or one selected operation test.
- **KMIPKIT-0015-FR-005**: Integration mutations, imports and deletion MUST be confined to test-owned objects in disposable container-local SQLite with no persistent database mount. Import MUST use solely test-generated material; Destroy MUST target only a unique object created by that same case. Teardown MUST remove the container and its database.
- **KMIPKIT-0015-FR-006**: The implementation MUST record the actual live-test result and MUST NOT claim a successful interoperability result before the test passes.
- **KMIPKIT-0015-FR-007**: The raw-TLS response reader MUST require the KMIP Response Message TTLV tag `0x42007B` defined by OASIS KMIP Specification v2.1 §8.4, Table 397, and §11.56. It MUST reject the Request Message tag `0x420078` on the response path.
- **KMIPKIT-0015-FR-008**: Each live case MUST use existing `ClientRequest` or typed convenience methods, validate operation-specific required response fields and avoid hand-encoded TTLV frames or sensitive output. Generic object material inside an existing typed operation payload is allowed by that API. An unrepresentable case MUST be recorded as a client blocker, never bypassed through an alternate wire client. Query and separate probes MUST NOT count as successful interoperability without a successful typed response meeting its contract.
- **KMIPKIT-0015-FR-009**: Each attribute case MUST provision a fresh, separate test-owned object; cases MUST NOT rely on execution order or shared identifiers.
- **KMIPKIT-0015-FR-010**: Documentation and traceability MUST identify exact operation clauses/tables and preserve historical selected-case evidence separately from new exhaustive-matrix observations.
- **KMIPKIT-0015-FR-011**: The capability inventory MUST cover all 35 advertised operation paths in Scope, with Set and Modify separate and Query retained as a known response defect. It MUST seek successful typed exchanges for supported paths while preserving every failed interoperability outcome. Ping and Adjust Attribute MUST remain separate unadvertised probes; their measured malformed-response outcomes MUST NOT be described as valid unsupported responses or advertised successes. Source-implemented but unadvertised Re-certify MUST remain a separate discrepancy probe when safely expressible, with a reason recorded if blocked; it MUST NOT expand the advertised-success set.
- **KMIPKIT-0015-FR-012**: Cases MUST cover every valid intersection of each scoped operation and the seven advertised managed-object types, using the object and operation clauses above. The inventory MUST distinguish managed-object support from primitive support, mark inapplicable dimensions and record source-backed invalid/unsupported exclusions. A documented valid intersection with an implementation gap MUST remain a discrepancy row. Activate/Opaque must not be invented where the operation/source excludes it.
- **KMIPKIT-0015-FR-013**: Import/Export MUST cover every applicable default and nonempty documented format, plus source-implemented formats, crossed only with valid object/algorithm combinations. Applicable ASN.1 PEM/DER vendor or fixture paths MUST be tested while preserving normative DER wire requirements and recording normalization separately. Blank format cells MUST NOT imply all formats. Include PrivateKey/PKCS#12 PFX import with the persistent note that `objects.md` and `import.rs::process_private_key/process_pkcs12` describe/implement it while `formats.md` omits it. Other format/encoding disagreements MUST receive the same explicit treatment; no invented standard PEM, PKCS7 or vendor enumeration is permitted.
- **KMIPKIT-0015-FR-014**: Cases MUST enumerate every documented or source-confirmed applicable algorithm and parameter variant for generation, Certify, Encrypt/Decrypt, Hash, MAC/MAC Verify, Sign/Signature Verify, Derive Key and wrapping, with exact normative dimension citations above and pinned vendor evidence. Source inspection MUST supplement incomplete documentation, including HMAC and derivation. Documented-but-source-gap variants, including Hash/SHA-224, MUST remain individually tested discrepancy rows. Certify variants must follow actual issuer/input parameters rather than assume caller-selectable signature algorithms. Cryptographic pairs MUST verify round trips or validity with matching explicit keys/parameters; standard units and algorithm semantics MUST NOT be weakened to fit server behavior.
- **KMIPKIT-0015-FR-015**: Each case MUST independently provision unique state, track all created/linked objects including PFX chains, and attempt applicable cleanup without hiding the original failure or retrying a KMIP request. No case may consume another case's identifiers or mutate user data. After every live run, including failure, the runner/procedure MUST stop and remove Compose services and disposable state and verify empty status; only ignored local test certificates/fixtures may remain.
- **KMIPKIT-0015-FR-016**: Each matrix row MUST have a stable unique case ID linked to FR(s), exact OASIS operation/data clauses, pinned Cosmian document and code location, operation, object, algorithm, parameters, format, encoding, setup dependencies, expected classification, executable test/family and actual observed result. Offline schema/completeness and case-to-test parity checks MUST detect missing paths/dimensions or unmapped rows. Reports MUST record run date, client commit, server image/configuration/transport and sanitized per-case status/reason or response-shape/delivery evidence. Not-run, setup/client-blocked, rejected, malformed, unsupported and successfully typed outcomes MUST remain distinct. A finding probe counts as observed interoperability only when its target request was sent, KMIPKit accepted a typed success response and its operation-specific postconditions passed; merely matching an expected finding, including an expected rejection, MUST NOT count. Observed interoperability for source-only values MUST remain distinct from KMIP 2.1 conformance claims.
- **KMIPKIT-0015-FR-017**: For KMIP 2.1 cryptographic multipart requests, typed builders MUST allow
  caller-provided per-part input when the operation supports it; they MUST preserve single-part
  required-input checks and invalid framing checks. For successful multipart Hash, MAC, and Sign
  responses, typed parsing MUST preserve optional operation output when present and MUST accept its
  absence; successful single-part responses MUST continue requiring operation output. Hash request
  sequencing follows OASIS KMIP Specification v2.1 §6.1; Correlation Value reuse follows §7.8; Hash
  fields follow §6.1.24, Table 235. Table 235 marks Data as not required for multipart Hash, while
  §6.1 states that Data is always required except when Init or Final Indicator is true. The
  project records this source interpretation as unresolved and makes no conformance claim for an
  intermediate Hash request that omits Data. This test sends Data on every part.
  The integration test MUST send explicit a, b, and c Data blocks in its three parts, reuse the
  Correlation Value from the first response, and compare a returned final digest with an independent
  known-answer vector before counting interoperability. This is the selected test input; the live
  test does not exercise the disputed omission case.

### Out of Scope

- Unadvertised operations beyond the separately classified Ping, Adjust Attribute and Re-certify probes; unsupported PGP Key and deprecated Template. CSR/format evidence must not be promoted into a claim of Certificate Request managed-object storage support.
- Using or sending requests to `demo-kms.cosmian.dev`.
- HTTPS/JSON TTLV integration, formal OASIS profile testing, certification, multi-server orchestration, or CI-hosted KMS deployment.
- Production protocol changes remain outside KMIPKIT-0015 except for the bounded multipart input/output correction in FR-017, authorized on 2026-10-17. The independently approved KMIPKIT-0021 scope covers the separate cryptographic TTLV tag correction. Transport/public API/release-boundary changes and edits to immutable OASIS source copies remain out of scope.

## Normative Traceability

| Requirement | Source | Verification |
|---|---|---|
| `KMIPKIT-0015-FR-003` | OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213; §9.16, Table 421 | `crates/kmipkit-client/tests/cosmian_kms.rs` sends typed Discover Versions and checks success plus returned `(2, 1)`. |
| `KMIPKIT-0015-FR-008`–`KMIPKIT-0015-FR-014` | Operation and normative data-dimension registries above; §11.56, Table 487 for shared cryptographic tags | Existing baseline plus planned manifest, offline validators and ignored typed operation-family cases; each row records its sources and actual outcome in `traceability.md`/the matrix. |
| `KMIPKIT-0015-FR-015`, `KMIPKIT-0015-FR-016` | Project isolation/reporting policy and approved exhaustive matrix design; operation-specific cleanup semantics above | Planned per-case setup/cleanup, case-to-test parity, live result records and Compose teardown/status evidence. These are project requirements, not additional OASIS clauses. |
| `KMIPKIT-0015-FR-001`, `KMIPKIT-0015-FR-002`, `KMIPKIT-0015-FR-004`, `KMIPKIT-0015-FR-005`, `KMIPKIT-0015-FR-006` | KMIPKit project constraints in `AGENTS.md` §§2, 6, 8; local integration policy | Compose configuration, generated-PKI script, ignored test gate, test docs, and live-test records. These are project requirements, not OASIS clauses. |
| `KMIPKIT-0015-FR-007` | OASIS KMIP Specification v2.1 §8.4, Table 397; §11.56 | `crates/kmipkit-transport/src/raw_tls.rs`; valid and wrong-root response frames in `crates/kmipkit-transport/tests/raw_tls.rs`; the live Cosmian response. |
| `KMIPKIT-0015-FR-017` | OASIS KMIP Specification v2.1 §6.1 (multipart sequence); §6.1.24, Tables
235–237 (Hash); §6.1.32, Tables 259–261 (MAC); §6.1.55, Tables 334–336 (Sign); §7.8 (Correlation
Value) | Protocol request/response contract tests; ignored Cosmian multipart Hash cases with
independent final digest verification and preserved initial Correlation Value. |

## Success Criteria

- **KMIPKIT-0015-SC-001**: One documented command starts a loopback-only Cosmian KMS 5.28.0 KMIP socket using newly generated local mTLS credentials.
- **KMIPKIT-0015-SC-002**: Ordinary Rust test runs skip every live integration test; an explicit ignored-test command executes the Cosmian operation suite.
- **KMIPKIT-0015-SC-003**: The inventory covers all 35 advertised paths with separate Set/Modify cases; Query and all separate probes retain their actual classifications. Interoperability success requires a successful typed response with all required fields.
- **KMIPKIT-0015-SC-004**: Every inventoried case has an individual live outcome or explicit blocker; summaries distinguish successful interoperability from passing finding assertions. Historical eighteen-exchange evidence is not reused as proof of new combinations or formal conformance.
- **KMIPKIT-0015-SC-005**: Offline checks validate unique IDs, complete source-derived valid intersections, required fields/citations and case-to-executable-test parity, with no silently omitted documented or source-implemented combination in scope.
- **KMIPKIT-0015-SC-006**: Import/Export cases cover applicable defaults, explicit formats and PEM/DER paths, including PrivateKey/PFX and its table omission; fixture normalization and actual wire representation are recorded separately.
- **KMIPKIT-0015-SC-007**: Every scoped algorithm/parameter variant has a case and evidence, including documentation/source discrepancies; required round trips and verification pairs assert matching results.
- **KMIPKIT-0015-SC-008**: Each case runs independently with its own unique setup and cleanup, without production material or sensitive output.
- **KMIPKIT-0015-SC-009**: After every live run, including failure, Compose teardown completes and status reports no running service; disposable database state is removed and only ignored local test certificates/fixtures remain.
- **KMIPKIT-0015-SC-010**: Multipart cryptographic output presence is optional in typed responses, while single-part output remains required. The Cosmian Hash multipart test sends three data blocks, uses the original Correlation Value on both later requests, and records success only if the final digest matches the independent known-answer vector; SHA-224 rejections remain separately classified against the pinned Cosmian source and documentation.

## Assumptions

- The integration is based on `release/1.0.0`, which contains the production raw-TLS transport and the typed Rust client paths for all 35 scoped operation paths and the separate probes; no new public API is authorized.
- Contributors have Docker Compose and Rust 1.94 or later. The PKI is generated inside a short-lived container, so host OpenSSL is not required.
- Cosmian KMS 5.28.0 supports the configuration documented at its matching source tag, including PEM TLS files, client-CA verification and the binary KMIP socket.

Pinned vendor documentation/source disagreements are recorded in [research.md](research.md). They must be tested or explicitly blocked, without silently changing the target, inventing standard values, or weakening typed parsing.
