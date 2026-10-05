# KMIPKIT-0008 Test Scenarios

These are acceptance scenarios for the future implementation. They do not authorize production secret transmission. KMIPKIT-0008 is permanently in-memory for its scope and uses a test-only generic TTLV representation; OD-004 is resolved by that scope and is not an implementation gate.

## Authentication

1. Build no Authentication and verify the containing request keeps it absent.
2. Build present Authentication with zero Credentials and require validation failure with no payload in the error.
3. Build one and multiple Credential values, including repeated variants; preserve count and order.
4. Decode an unknown Credential Type and Extensions value; preserve raw enum and opaque TTLV tree exactly.

## Credential variants

1. For Tables 411–416, build the minimum-valid and optional-field variants and compare field order/type to the source table.
2. Reject missing Username, OTP, Timestamp, Hashed Password, Attestation Type, or Nonce where required.
3. Represent and preserve every Table 412 Device field and validate individual field types. Keep only empty/minimum-field validation gated by OD-002 until the covered set is reviewed; do not claim local/global uniqueness enforcement.
4. Test omitted, explicit SHA-256, other known, and unknown Hashing Algorithm values. Omitted algorithm has effective SHA-256 semantics while retaining absence.
5. Require and preserve caller-provided Timestamp and hashed bytes, and expose effective SHA-256 when the optional algorithm is omitted. Do not calculate hashes or implement/claim monotonicity checking or tests until OD-003 review settles owner, comparison scope, and clock behavior.
6. Preserve Nonce ID/Value bytes, including embedded zero bytes; never generate or modify them.
7. Attestation requires Measurement or Assertion; test neither, each individually, and both.

## Safety and capability

1. Place unique sentinel secrets in every sensitive field. Assert the exact sentinel is absent from Debug, Display, validation errors, and captured logs.
2. Verify KMIPKit-owned secret allocations use the approved zeroization boundary; do not claim zeroization of external language-runtime copies.
3. Verify indicator false/omitted when the public API cannot construct attestation, and true only when it can construct the structural model from caller data.
4. Assert this feature exposes no production credential writer or send path. Any later path requires a separate approved feature and candidate-callsite/owner-through-transport lifecycle evidence.
5. Keep Credential and Authentication models usable as standalone in-memory values. Gate only execution integration on an explicit KMIPKIT-0007 handoff that settles inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the whole batch.
6. Keep OTP request-scoped per architecture without adding library-wide replay/single-use state or claiming normative client enforcement; the informative source clause is `KMIPKIT-CLAUSE-SPEC-9.11-008`. Resolve execution selection under OD-005.

## Evidence

Every test name and case should identify its requirement ID and exact Specification section/table. Mark tests derived. Do not label them OASIS official vectors unless a pinned official test ID and available fixture are later linked by the catalog.
