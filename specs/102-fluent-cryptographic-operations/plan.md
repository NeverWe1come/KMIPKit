# Implementation Plan: KMIPKIT-0102

## Design

Add explicit builder types in `crates/kmipkit-client/src/fluent.rs` for the
seven KMIP 2.1 cryptographic operations listed in the feature specification.
Each builder owns its existing protocol request, sets fields through the
request's typed methods, and delegates local validation, batching, and sending
to the existing typed client route. Use semantic byte setters that move
caller-owned `Vec<u8>` values into `SecretBytes`; do not introduce formatted
debug output or implicit cryptographic parameters. Keep public methods
explicit so API-boundary auditing can inspect each signature.

## Red / Green / Refactor

1. Add constructor and request-shape tests for all seven builders; run the
   focused command and preserve the missing-method failure as Red evidence.
2. Implement the smallest explicit builder set using existing request models.
   Add caller-selected key identifier, data, cryptographic parameters,
   operation-specific MAC/signature/AEAD fields, and multipart indicators only
   where the corresponding typed model supports them.
3. Add local validation, exact TTLV payload, typed batch conversion,
   one-exchange fake-transport execution, result propagation, redaction, and
   version-gate tests. Update manifest, traceability, and Rust documentation.
4. Refactor only after the tests pass: remove duplication without hiding public
   methods behind macros or weakening the source-boundary checks.
5. Run format, focused and full client tests, Clippy, workspace coverage,
   manifest generation and audit, guide-example checks, and CI. Keep Red,
   Green, and Refactor in separate signed commits. Open a stacked `[WIP]` PR
   against #184 and leave it WIP until every exact-head CI check is green.

## Risks and mitigations

- The seven typed request models have different data and multipart rules; do
  not flatten them into one public generic request.
- Encrypt and Decrypt have AEAD-only members whose presence is constrained to
  the initial part; reuse their existing validators and add negative tests.
- Sign and verification operations distinguish original data, digested data,
  signature data, and MAC data; give each a distinct semantic setter.
- Cryptographic fields may contain sensitive material; keep owned bytes in
  `SecretBytes` and test all relevant formatting and errors for redaction.
- The branch depends on unmerged PR #184; use its current head as the base and
  rebase after parent changes without force-pushing shared branches.
