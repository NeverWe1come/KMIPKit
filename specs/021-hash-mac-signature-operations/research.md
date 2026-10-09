# Research: KMIP 2.1 Hash, MAC, and Signature Operations

## Decision 1: Reuse the existing typed client execution boundary

**Decision**: Implement the five operations as typed protocol requests/responses added to the existing closed `ClientRequest` / `ClientOperation` dispatch and the common one-exchange execution path. Do not introduce caller-supplied raw KMIP wire bytes.

**Rationale**: `crates/kmipkit-client/src/execute.rs` already owns batch construction, shared message encoding, TLS/HTTPS exchange, response correlation, result extraction, and delivery-state handling. Adding operation variants preserves those security and reliability guarantees.

**Alternatives considered**: A separate operation transport or raw-TTLV public escape path would duplicate transport behavior and bypass existing request validation and response ownership.

## Decision 2: Reuse the KMIPKIT-0019 cryptographic input contracts

**Decision**: The implementation depends on KMIPKIT-0019's `OperationData` model for §7.9 Data and its lossless ordered `Structure` representation for Cryptographic Parameters. It also reuses the shared Data, Correlation Value, Init Indicator, and Final Indicator fields assigned to KMIPKIT-0019. Implement KMIPKIT-0019 before coding KMIPKIT-0021; do not fork duplicate equivalents.

**Rationale**: Encrypt/Decrypt and Hash/MAC/Sign share these wire fields. The current release branch contains the KMIPKIT-0019 specification but not its operation model, so the dependency is explicit. It prevents API divergence and keeps unknown Cryptographic Parameters members intact.

**Alternatives considered**: Adding local duplicate CP/Data models would make the five operations inconsistent with the earlier cryptographic family and increase compatibility work.

## Decision 3: Preserve algorithm values without executing algorithms

**Decision**: Hashing Algorithm, Cryptographic Algorithm, Digital Signature Algorithm, and Validity Indicator values use open/lossless raw enumeration representations, including extension and future values. KMIPKit serializes caller-supplied values but performs no local digest, MAC, signature, or verification algorithm and makes no algorithm choice.

**Rationale**: This matches the product boundary and the `kmipkit-ttlv` value-preservation model. All standardized enumeration values are inventoried and assigned in the catalog, while forward compatibility is maintained for unknown values.

**Alternatives considered**: A closed Rust enum alone would reject unknown/vendor values and would not satisfy the lossless protocol contract. A local cryptographic provider would violate product scope.

## Decision 4: Treat operation inputs and outputs as sensitive opaque material

**Decision**: Reuse the existing zeroizing owner for KMIPKit-owned byte strings. Manually redact Debug/Display and sanitized errors for request and response data, MAC values, signatures, digested data, and cryptographic parameter contents. Response views borrow from the source response where practical.

**Rationale**: These fields can contain plaintext, signatures, MACs, or other sensitive values. The shared security invariants prohibit leaking payload material in diagnostics.

**Alternatives considered**: Formatting values for diagnostics or copying response bytes into unnecessary owned buffers would increase disclosure and lifetime risk.

## Decision 5: Preserve the disputed final multipart verification shape

**Decision**: Keep `KMIPKIT-DISC-048` open. The typed client accepts a present or absent Validity Indicator in the final multi-part MAC Verify or Signature Verify response and does not use that disputed field to claim server conformance. A non-final multipart response with a Validity Indicator is a typed response-shape error; the generic TTLV response remains available for lossless inspection.

**Rationale**: Tables 263/338 conflict with operation prose only for the final part. Both source forms prohibit the indicator on non-final parts. Tolerating both final forms lets the client preserve either normative reading without silently choosing a server requirement.

**Alternatives considered**: Rejecting all final multipart indicators selects the table; requiring one selects the prose. Neither is justified while the pinned normative source remains internally inconsistent.

## Decision 6: Use table-derived tests while official fixtures remain absent

**Decision**: Add exact TTLV model vectors, negative response-shape tests, property/round-trip tests, and fake-transport tests for every operation table. Record official test mappings as unavailable fixture evidence; do not mark those linked cases passed.

**Rationale**: The catalog links official Test Cases for Hash, MAC, Sign, and Signature Verify, but their XML fixtures are unavailable in the pinned repository; MAC Verify has no explicitly linked case. Table-derived tests are necessary implementation evidence without overstating official evidence.

**Alternatives considered**: Inferring payloads or passing results from case titles would not reproduce a source vector and would create unsupported conformance claims.

## Local evidence reviewed

- Pinned OASIS Specification v2.1 at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`; no network fetch or modification.
- Reviewed KMIPKIT-0019 shared input decisions in `specs/019-cryptographic-operations/data-model.md` and `contracts/operation-payloads.md`.
- Reviewed the existing typed request pipeline in `crates/kmipkit-client/src/execute.rs`, protocol operation models, workspace version/lint settings, and the active constitution.
- Catalog and deterministic report validation are recorded in the specification PR verification summary.
