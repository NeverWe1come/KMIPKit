# Cosmian 5.28.0 Exhaustive KMIP Integration Matrix

**Status:** Design proposal approved in chat on 2026-10-16; awaiting review of this written design.

## Goal

Extend the opt-in Rust integration suite so KMIPKit exercises every KMIP 2.1 operation Cosmian KMS 5.28.0 advertises as supported and every applicable object, key-format, encoding, algorithm, and parameter combination documented or implemented by that exact release.

## Evidence and current baseline

The test server is already pinned to `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`. The upstream tag resolves to commit `cec59a003b9cfc4f253ca239fde42925988700fa`. The local Compose configuration uses SQLite without a persistent data mount, loopback-only port 5696, and runtime-generated test mTLS certificates.

The existing ignored integration target has cases for eighteen operations with successful typed results, plus Query and Ping probes. Query is advertised as supported but its response omits the required Protection Storage Masks field and KMIPKit rejects the typed response. Cosmian rejects Ping and the separate Adjust Attribute probe as unsupported. These outcomes must remain visible as interoperability findings, not be relabeled as successful operation exchanges.

The 5.28.0 support table advertises 34 operation rows; its `Set Attribute (Modify)` row groups the related Set and Modify paths already exercised separately. Sixteen advertised operation paths have no current integration case:

| Operation | OASIS KMIP 2.1 source |
|---|---|
| Certify | §6.1.6, Tables 179–181 |
| Check | §6.1.7, Tables 183–185 |
| Derive Key | §6.1.14, Tables 205–207 |
| Destroy | §6.1.15, Tables 208–210 |
| Export | §6.1.18, Tables 217–219 |
| Get | §6.1.19, Tables 220–222 |
| Import | §6.1.25, Tables 238–240 |
| Join Split Key | §6.1.27, Tables 244–246 |
| Locate | §6.1.28, Tables 247–249 |
| Re-key | §6.1.46, Tables 307–309 |
| Re-key Key Pair | §6.1.47, Tables 310–312 |
| Register | §6.1.43, Tables 291–294 |
| Revoke | §6.1.44, Tables 295–297 |
| RNG Retrieve | §6.1.49, Tables 316–318 |
| RNG Seed | §6.1.50, Tables 319–321 |
| Validate | §6.1.57, Tables 340–342 |

Inspection of `crates/kmipkit-client/src/execute.rs::ClientRequest` confirms the typed Rust API already represents all these paths. No additional operation API specification is needed unless an integration case exposes a production defect; such a defect must be fixed under its own approved specification.

## Source hierarchy for the matrix

1. OASIS KMIP Specification v2.1 is normative for request/response shape and valid operation/object combinations. Each test and each matrix row will cite the relevant `§6.1.x` clause and tables; shared structures will cite their exact clauses too.
2. Cosmian KMS tag `5.28.0` source and documentation define the vendor-specific supported combinations for this test target. The versioned inputs are `documentation/docs/kmip_support/support.md`, `objects.md`, `formats.md`, `documentation/docs/certifications_and_compliance/cryptographic_algorithms/algorithms.md`, and the corresponding `crate/server/src/core/operations/` implementation.
3. The image digest above fixes the runtime under test. Mutable documentation for later releases will not silently expand this matrix.

The matrix will distinguish managed-object support from cryptographic primitive support. In the 5.28.0 support table, Certificate, Symmetric Key, Public Key, Private Key, Split Key, Secret Data, and Opaque Data are managed objects; PGP Key is unsupported and Template is deprecated. The separate objects page lists cryptographic primitives for Certificate, symmetric keys, public keys, and private keys. Those are different capability axes, not a reason to exclude lifecycle tests for other managed objects.

## Proposed test design

Create a checked-in, versioned capability matrix under `tests/integration/cosmian/`. It will contain one row for each applicable, valid intersection, with at least: stable case ID, KMIP operation, object type, algorithm and parameters where applicable, key format and encoding where applicable, expected server outcome, exact OASIS citation, test name, and observed live result.

The matrix will cover:

- One successful typed request/response path for each advertised supported operation, except that known Query nonconformance remains reported as a failed interoperability path with its exact response-shape finding.
- Every object type supported by the server for each operation where OASIS semantics and Cosmian 5.28.0 source permit that object. It will not send invalid object/operation pairs.
- For Import and Export, each supported default format and every non-empty Import/Export format listed for the applicable object/key algorithm. Every applicable ASN.1 encoding will be tested as both PEM and DER. Algorithm-dependent formats will be crossed only with the algorithms for which the release supports them. Include the source-confirmed PKCS#12/PFX import through a `PrivateKey` object; record that `formats.md` omits this case while `objects.md` and `import.rs` describe and implement it.
- Every 5.28.0 algorithm and parameter variant applicable to key generation, Certify, Encrypt/Decrypt, Hash, MAC/MAC Verify, Sign/Signature Verify, Derive Key, and key wrapping. The 5.28.0 source is required where the algorithm documentation is incomplete; for example, it does not enumerate the HMAC variants that `mac.rs` implements.
- Cryptographic round trips or verification pairs where the operation contracts require matching keys, parameters, or output data.

The current test file is already large. New cases will be organized by operation family under the existing `cosmian_kms` integration target, sharing only typed-client and test-fixture helpers. Each case will provision its own state so it can run independently and report its own matrix ID.

## Isolation and secret handling

All requests will use KMIPKit's production typed `Client` over the existing raw TTLV TLS 1.3/mTLS path. Tests remain ignored in ordinary Cargo runs and use only the local pinned container. Imported material will be generated solely for tests; no production credentials or user data will be used. Destroy will target only a unique object created by that test in the disposable SQLite database. Tests and diagnostics will never print private key bytes, secret material, credentials, or raw TTLV bodies.

Run the suite serially through `scripts/integration/cosmian-kms.ps1`. After the run, including a failing run, stop and remove the Compose services and disposable database, then confirm `status` reports no running service. Preserve only ignored local certificates for reuse.

## Reporting and acceptance

A case marked supported by the pinned server is interoperable only after KMIPKit sends a typed request and parses a successful typed response with its required fields. A server rejection, malformed response, unsupported operation, or unimplemented server combination must be recorded separately with the KMIP status/reason or response-shape evidence. The suite must not turn a known incompatibility into a green interoperability claim.

The follow-up feature specification will update KMIPKIT-0015 scope, stable requirement IDs, task plan, OASIS traceability, English/Spanish integration instructions, and the matrix. It will keep all live tests opt-in and will not claim KMIP profile conformance or certification.

## Source questions for review

- `objects.md` says PKCS#12/PFX can be imported, while the `formats.md` table omits it. `crate/server/src/core/operations/import.rs` implements a `PrivateKey`/PKCS#12 import path that extracts the private key, leaf certificate, public key, and optional chain. **Resolution:** include this source-confirmed PFX case and record the documentation omission in the capability matrix.
- `formats.md` has blank available-format cells for several defaults. The matrix must distinguish “default format to test” from “explicitly advertised non-default Import/Export format”; blank cells must not be silently interpreted as support for every format.

## Out of scope

- Access to the public Cosmian demo or any external account/service.
- HTTPS/JSON TTLV transport, multiple KMS vendors, HSM certification, formal KMIP profile conformance, or certification claims.
- Production protocol/API changes in this integration feature. If an integration test uncovers a KMIPKit defect, record it and create a separate approved implementation scope before changing production behavior.
