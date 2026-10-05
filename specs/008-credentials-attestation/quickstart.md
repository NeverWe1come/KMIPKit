# KMIPKIT-0008 Test Scenarios

These are acceptance scenarios for the future implementation. They do not authorize production secret transmission. Conversion tests in this feature remain in-memory and use a test-only generic TTLV representation regardless of OD-004; this feature never adds a production credential send path.

## Authentication

1. Build no Authentication and verify the containing request keeps it absent.
2. Build present Authentication with zero Credentials and require validation failure with no payload in the error.
3. Build one and multiple Credential values, including repeated variants; preserve count and order.
4. Decode an unknown Credential Type and Extensions value; preserve raw enum and opaque TTLV tree exactly.

## Credential variants

1. For Tables 411–416, build the minimum-valid and optional-field variants and compare field order/type to the source table.
2. Reject missing Username, OTP, Timestamp, Hashed Password, Attestation Type, or Nonce where required.
3. Keep Device empty/minimum-field cases gated by OD-002; test all individual Device fields and their types meanwhile.
4. Test omitted, explicit SHA-256, other known, and unknown Hashing Algorithm values. Omitted algorithm has effective SHA-256 semantics while retaining absence.
5. Preserve caller-provided Timestamp and hashed bytes without performing the hash formula. Leave monotonic-sequence verification gated by OD-003.
6. Preserve Nonce ID/Value bytes, including embedded zero bytes; never generate or modify them.
7. Attestation requires Measurement or Assertion; test neither, each individually, and both.

## Safety and capability

1. Place unique sentinel secrets in every sensitive field. Assert the exact sentinel is absent from Debug, Display, validation errors, and captured logs.
2. Verify KMIPKit-owned secret allocations use the approved zeroization boundary; do not claim zeroization of external language-runtime copies.
3. Verify indicator false/omitted when the public API cannot construct attestation, and true only when it can construct the structural model from caller data.
4. Assert this feature exposes no production writer/transport entry point accepting Authentication. Any later send path requires a separate approved feature and candidate-callsite/owner-through-transport lifecycle evidence.

## Evidence

Every test name and case should identify its requirement ID and exact Specification section/table. Mark tests derived. Do not label them OASIS official vectors unless a pinned official test ID and available fixture are later linked by the catalog.
