# KMIPKIT-0092 Implementation Plan

1. Define a checked-in route-capability manifest for Rust, C, Java, and Python,
   tied to the installed public API manifest and exact artifact commit.
2. Generate the required case/version/construction-route/encoding/transport
   matrix from the OASIS-backed catalog and route capabilities.
3. Extend shared evidence and aggregation checks to retain each matrix cell,
   compare normalized request/response trees, and reject missing, duplicate,
   mislabeled, or incomparable results.
4. Add shared CI gates and controlled-fixture contracts that fail when a
   declared public construction path is not actually exercised.
5. Update common documentation and requirement traceability. Keep the four
   language validators on separate follow-up branches until each satisfies
   the matrix; a green common-contract workflow is not evidence that a
   language validator is complete.

## Global constraints

- KMIP 2.0 and 2.1 OASIS standard content only; no vendor extensions.
- Use the exact installed package artifact for each recorded release commit.
- Preserve deterministic batch order and compare canonical numeric result
  codes.
- Normalize only documented nondeterministic identifiers and secret fields.
- Compare exact bytes only when request inputs and encoding are deterministic
  and identical.
- Do not serialize secrets, raw KMIP bodies, remote text, or private material
  into reports.
- Do not claim live vendor interoperability without a configured vendor KMS.
- Every generator source and generated artifact change belongs in one PR.
