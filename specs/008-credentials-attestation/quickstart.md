# KMIPKIT-0008 Test Scenarios

These are acceptance scenarios for the implementation. Credential and Authentication values remain in-memory and are not sent by this feature. The only execute-path change is the non-secret Attestation Capable Indicator in the existing Request Header builder; OD-004 continues to prohibit a new Credential writer or secret-bearing send path.

## Authentication

1. Build no Authentication and verify the containing request keeps it absent.
2. Build present Authentication with zero Credentials and require validation failure with no payload in the error.
3. Build one and multiple Credential values, including repeated variants; preserve count and order.
4. Decode an unknown Credential Type and Extensions value; preserve raw enum and opaque TTLV tree exactly.

## Credential variants

1. For Tables 411–416, build the minimum-valid and optional-field variants and compare field order/type to the source table.
2. Reject missing Username, OTP, Timestamp, Hashed Password, Attestation Type, or Nonce where required.
3. Represent and preserve every Table 412 Device field and validate individual field types. Reject a Device with no Table 412 member; accept field presence independently of text content. Document that the caller must supply a unique one or combination of the four identifiers named in §9.11. The source does not specify comparison scope, and KMIPKit does not verify uniqueness from client-local data.
4. Test omitted, explicit SHA-256, other known, and unknown Hashing Algorithm values. Omitted algorithm has effective SHA-256 semantics while retaining absence.
5. Require and preserve caller-provided Timestamp and hashed bytes, and expose effective SHA-256 when the optional algorithm is omitted. Do not calculate hashes or implement/claim monotonicity checking or tests until OD-003 review settles owner, comparison scope, and clock behavior.
6. Preserve Nonce ID/Value bytes, including embedded zero bytes; never generate or modify them.
7. Attestation requires Measurement or Assertion; test neither, each individually, and both.

## Safety and capability

1. Place unique sentinel secrets in every sensitive field. Assert the exact sentinel is absent from Debug, Display, validation errors, and captured logs.
2. Verify KMIPKit-owned secret allocations use the approved zeroization boundary; do not claim zeroization of external language-runtime copies.
3. Capture a request through the existing fake-transport execute path and verify Attestation Capable Indicator=True, Authentication absent, and no Credential payload. Separately preserve the omitted/effective-false behavior for an externally supplied header.
4. Assert this feature adds no Credential writer or secret-bearing send path and retains the single existing writer/permit boundary. Any later credential path requires a separate approved feature and candidate-callsite/owner-through-transport lifecycle evidence.
5. Keep Credential and Authentication models usable as standalone in-memory values. Gate only future execution integration on an explicit KMIPKIT-0007 handoff that settles inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the whole batch.
6. Keep OTP request-scoped per architecture without adding library-wide replay/single-use state or claiming normative client enforcement; the informative source clause is `KMIPKIT-CLAUSE-SPEC-9.11-008`. Resolve execution selection under OD-005.

## Evidence

Every test name and case should identify its requirement ID and exact Specification section/table. Mark tests derived. Do not label them OASIS official vectors unless a pinned official test ID and available fixture are later linked by the catalog.
