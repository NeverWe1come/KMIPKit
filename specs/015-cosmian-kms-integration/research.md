# Research: Local Cosmian KMS Integration

## Decisions

### Use the binary KMIP socket

**Decision**: Connect to Cosmian's raw TCP socket using KMIP binary TTLV and TLS 1.3/mTLS through the production `kmipkit_client::Client` API.

**Rationale**: Cosmian documents a native KMIP socket that carries binary TTLV, while its HTTP KMIP API is a JSON TTLV API. KMIPKit 1.0's existing production raw-TLS transport is the matching path. The KMS 5.28.0 configuration supports a listener on `5696`, server PEM certificate/key, and a client CA.

**Sources**:

- Cosmian KMS 5.28.0 server configuration reference: [server_configuration_file.md](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/configuration/server_configuration_file.md), database defaults at lines 171–196, TLS and KMIP socket settings at lines 197–233.
- Cosmian's [KMIP socket and JSON TTLV documentation](https://docs.cosmian.com/key_management_system/kmip_support/json_ttlv_api.html).
- KMIPKit production transport contract and TLS policy: `specs/013-production-transport/spec.md`, `specs/013-production-transport/plan.md`, and OASIS KMIP Profiles v2.1 Operating System Profile §5.3.1 items 3–4 as the existing profile context.

### Pin the local server and isolate it

**Decision**: Use `ghcr.io/cosmian/kms:5.28.0@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e` and `alpine:3.22.2@sha256:4b7ce07002c69e8f3d704a9c5d6fd3053be500b7f1c69fc0d80990c2ad8dd412`; run them only for local tests, publish KMIP port `5696` to `127.0.0.1`, use SQLite in the disposable container filesystem with database clearing enabled, and do not mount persistent KMS data.

**Rationale**: This makes the requested KMS version reproducible and keeps all mutations inside the disposable local service. The original bounded suite created server-side test objects and changed their metadata. The 2026-10-16 amendment additionally authorizes importing test-generated fixtures and destroying only unique objects created by the same case. Stopping and removing the container discards its database state.

### Generate temporary mutual-TLS credentials in Compose

**Decision**: A short-lived Alpine/OpenSSL Compose service creates a local CA and `localhost` server/client certificates in an ignored `.local/cosmian-kms/certs` directory, then exits successfully before KMS starts.

**Rationale**: Contributors do not need host OpenSSL, checked-in credentials, a secret manager, or an externally issued certificate. The root CA is supplied explicitly to KMIPKit and to Cosmian for client-certificate validation.

### Keep the first live test opt-in and narrow

**Decision**: Add one ignored test for the existing typed Discover Versions request. The user later authorized executing it while the KMIPKIT-0013 coverage blocker was still open.

**Rationale**: The branch already has production raw-TLS, mTLS, TTLV framing, typed request and typed response support for this one operation. OASIS KMIP Specification v2.1 §6.1.16, Tables 211–213, defines the operation and response; §9.16, Table 421, defines the Protocol Version structure. No broader operation support is required to test this vertical slice.

## Risks and limitations

- The GitHub container registry image must be reachable and the `5.28.0` image tag must be published for the contributor's platform.
- The initial smoke test covered only Discover Versions. The approved 2026-10-16 matrix expansion below replaces that original limitation; new live outcomes remain pending.
- This smoke test exercises one request/response on one implementation. It is not a conformance suite, full KMIP 2.1 compatibility result, or certification.
- TLS certificate files are generated as local ignored artifacts. Removing `.local/cosmian-kms` deletes those files; Compose can regenerate them on the next start.

## Initial live-test failure and diagnosis

The user later authorized the live test before KMIPKIT-0013's coverage gate was cleared. The first run reached the KMS over verified mTLS; Cosmian logged one `DiscoverVersions` request for `kmipkit-integration-client`. KMIPKit then returned a sanitized transport error with `ResponseStarted` and cause `Other`. Temporarily logging only the eight-byte TTLV response-header metadata showed `tag=0x42007b`, `type=0x01`, and `value_len=168`; the diagnostic code was removed immediately afterward. OASIS KMIP v2.1 §8.4, Table 397, defines Response Message as the root structure, and §11.56 assigns its tag `0x42007B`. The transport had been expecting `0x420078`, the Request Message tag. This identifies a KMIPKit response-frame validation defect rather than a missing Cosmian setting.

## Deployment-only evidence

On 2026-10-08, `docker compose config --quiet` and the PowerShell parser check succeeded. The test-PKI Compose service generated the certificates and exited successfully; the Cosmian KMS 5.28.0 container started and its log reported `Socket server listening on 0.0.0.0:5696`. Docker showed only `127.0.0.1:5696->5696/tcp` published on the host. The resolved image digest was `ghcr.io/cosmian/kms@sha256:7b60fd4484930969906caa5722b727054ff96d49339ce81e3c64ca9e4278540e`. Container inspection showed mounts for the read-only KMS config and the server-certificate directory; the client certificate/key and CA private key are not mounted into the KMS container. `git check-ignore` confirmed the CA and client private-key paths are ignored.

After the response-tag fix, `cargo test -p kmipkit-client --test cosmian_kms -- --ignored` passed (1 passed, 0 failed). The server therefore requires no additional configuration for this Discover Versions exchange. `cargo test -p kmipkit-transport --test raw_tls` passed (90 passed, 0 failed). The production-client suite passed (14 passed, 0 failed) using `cargo test -p kmipkit-client --test production_client --features kmipkit-test-support/fixtures`. The Cosmian container remained running after the test.


## Exhaustive matrix amendment — 2026-10-16

### Source order and exact release identity

**Decision**: Use OASIS KMIP Specification v2.1 for normative validity and operation/data contracts, then the exact Cosmian 5.28.0 documentation and implementation to select vendor combinations. The source references below all use tag `5.28.0`; `git ls-remote` confirms annotated tag object `cec59a003b9cfc4f253ca239fde42925988700fa` dereferences to commit `4ab3ee65829a903abb71f1298f400657fcee7ec9`. The runtime remains pinned to the image digest in `spec.md`. Neither tag metadata nor vendor declarations alone prove an observed runtime result.

**Normative evidence**: Exact object, format, parameter and encoding clauses were read from immutable local `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` (Windows-1252 HTML). `spec.md` contains the reviewed operation and data-dimension registry. Each concrete manifest row must refine those citations and vendor source symbols; a generic §6 citation alone is insufficient.

### Versioned vendor source ledger

| Input | Purpose |
|---|---|
| [support.md](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/kmip_support/support.md), Operations by KMIP Version and Managed Objects | 34 published supported rows, 35 operation paths; seven advertised managed-object types; unsupported/deprecated object exclusions. |
| [objects.md](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/kmip_support/objects.md), object table and Notes | Primitive/object distinction, CSR input and certificate/PFX import descriptions. The four primitive-bearing object categories do not exclude lifecycle tests for other managed objects. |
| [formats.md](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/kmip_support/formats.md), Import–Export and Internal storage | Defaults, nonempty explicit formats, ASN.1 PEM/DER claim, NIST-only SEC1 restriction; blank cells are not blanket format support. |
| [algorithms.md](https://github.com/Cosmian/kms/blob/5.28.0/documentation/docs/certifications_and_compliance/cryptographic_algorithms/algorithms.md), operation-specific algorithm sections | Documented algorithms, sizes, curves, modes, hashes, padding and wrapping parameters; only applicable combinations become cases. |
| [operation source directory](https://github.com/Cosmian/kms/tree/5.28.0/crate/server/src/core/operations) and [dispatch.rs](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/dispatch.rs) | Actual object/parameter restrictions and source-only combinations; follow shared crypto conversion helpers where operation code delegates. |

### Operation scope and disagreements

**Decision**: Enumerate the exact 35-path set in `spec.md` before implementation. Set Attribute and Modify Attribute have distinct typed requests and independent measured successes in the historical twenty-case run; retain the single published `Set Attribute (Modify)` grouping as source provenance. Query stays in the set with the known Table 283 required-field defect. Ping and Adjust Attribute remain unsupported probes. [dispatch.rs](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/dispatch.rs) dispatches `ReCertify` although `support.md` marks Re-certify unimplemented: retain a separate safe typed discrepancy probe (OASIS §6.1.45, Tables 298–302), or explicit setup/client blocker, outside the advertised-success count.

**Disagreement policy**: Record both documentation and implementation citations. Retain standards-valid documented combinations even when source inspection predicts rejection; classify them as documented-but-source-gap and observe the live outcome. Add applicable source-only formats/parameters to the inventory without describing them as published promises. Standards-invalid combinations get source-backed exclusions rather than fabricated positive tests. A source comment claiming KMIP compliance cannot override the pinned normative text. Runtime outcome remains independent of expected classification.

### Import/Export formats, PFX and encoding

**Decision**: Test each applicable default and explicit format, with algorithm restrictions. Include PrivateKey/PKCS#12 PFX import: `objects.md` Notes advertises PFX, while [import.rs](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/import.rs) `process_private_key` selects `process_pkcs12`, which parses a `PrivateKey` Key Block and extracts linked private/public key, leaf certificate and optional chain. `formats.md` omits this path. Preserve that omission note in its matrix row, including links and cleanup ownership for every created object.

Inspect [export.rs](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/export.rs) and its delegated conversion helpers for other source-only formats, including PKCS7/PKCS12 legacy variants, transparent symmetric material and Secret Data/Opaque export. These are inventory candidates, not invented standard enumerations or evidence of a successful exchange. Capture exact source symbols, representation and operation applicability before accepting each row.

OASIS §11.25, Table 461 defines PKCS#12 `0x16` and PKCS#10 `0x17`; it does not define PEM or PKCS7 Key Format Type values. OASIS §2.1, Table 2 and §11.25, Table 460 require DER for certificates and DER-defined key formats. The vendor `formats.md` advertises PEM/DER, whereas `import.rs::process_certificate/process_pkcs12` uses DER parsing and key conversion paths must be checked individually. Keep fixture encoding, any normalization, actual wire format and documented-but-source-gap probes distinct; do not treat PEM text as a standard wire-format obligation. PEM *Certificate Request Type* is separately defined by §11.8, Table 439 and may apply to Certify input. Wrapping Encoding Option (§11.18, Tables 449–450) means No Encoding/TTLV Encoding, not PEM/DER.

### Algorithms, parameters and valid object intersections

- Inspect [mac.rs](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/mac.rs) for HMAC variants absent from `algorithms.md`, and [derive_key.rs](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/derive_key.rs) for actual methods/parameters. Standard enumeration presence does not itself mean Cosmian supports that variant.
- `algorithms.md` Hash functions advertises SHA-224; [hash.rs::compute_hash](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/hash.rs) selects SHA-256/384/512 and SHA3 variants but omits SHA-224. Include the SHA-224 Hash row with both sources and measure its result; do not silently drop it or turn expected rejection into interoperability success (§6.1.24, Tables 235–237; §11.21, Table 454).
- The RSA-OAEP/AES128-GCM example in `algorithms.md` and [encrypt.rs::encrypt_with_rsa](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/encrypt.rs) disagree on the request's algorithm selection. Record the documented path and source-supported path separately where expressible, citing applicable §4.16/§6.1.17 contracts; do not assume a successful cross-product.
- OASIS §4.16 explicitly defines **IV Length in bits and Tag Length in bytes**. Cosmian's byte-based GCM TagLength handling is not evidence of a bits-versus-bytes defect. Use standard units and mandatory applicable parameters; unsupported lengths remain measured outcomes.
- Certify's issuer-key selection and digest behavior must be derived from [certify.rs](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/certify.rs) and delegated helpers. Do not invent caller-selectable Certify signature/digest combinations where the source fixes SHA-256 for RSA/EC issuers or omits a digest for other issuer types.
- [activate.rs](https://github.com/Cosmian/kms/blob/5.28.0/crate/server/src/core/operations/activate.rs) explicitly rejects OpaqueObject; retain that as an operation-specific source restriction, not a universal managed-object exclusion or an invented normative rule. Inspect Revoke/Destroy independently for Opaque support. General operations and cryptographic operations need different object intersections.

### Test architecture, isolation and reporting

**Decision**: Keep `cosmian_kms` as the integration target and split family cases into focused modules. The JSON manifest is the versioned inventory. A stable row records FR links, operation/object, algorithm/parameters, format/encoding, setup, exact normative/vendor sources, expected outcome, test/family and actual result. Offline validators prove completeness and case-to-test parity; every network case remains ignored.

All supported paths and separate probes already have `ClientRequest` variants. Typed Import/Register/Get/Export can carry their existing object payloads; this does not authorize a second transport, hand-encoded frames, parser weakening or public API changes. Any unrepresentable required row is an explicit client blocker requiring separately approved production scope.

Each row generates its own fixtures and identifiers, tracks linked objects, and attempts cleanup without hiding the original failure. Destroy targets only an object that case created. Run serially through the existing local runner, retain sanitized per-case outcomes, and tear down Compose even after failures; verify no running service remains. Historical eighteen-success evidence does not close any new row. The complete matrix and live results remain pending implementation after this documentation task.

**Citation correction from the task brief/design**: Re-key §6.1.46 uses Tables 303–304 for date/attribute rules, Table 305 for the request, Table 306 for the response and Table 307 for errors. The supplied 307–309 range incorrectly crosses into Re-key Key Pair. This amendment uses Tables 303–307. Re-key Key Pair §6.1.47 request/response/errors remain Tables 310–312, with date/attribute rules in 308–309. The pinned OASIS source controls this correction.
