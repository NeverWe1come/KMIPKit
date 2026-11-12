# KMIPKIT-0107 Implementation Plan: Fluent End-to-End Journeys

**Owning requirement**: KMIPKIT-0093-FR-013 and F001.
**Parent**: KMIPKIT-0093 typed-result branch at `dc9819f61` (PR #193).
**Status**: implementation plan for the five approved end-to-end journeys.

## Scope

Implement the five progressive journeys in Rust, C, Java, and Python: symmetric
key lifecycle, AES-GCM, key-pair signing and verification, expert attributes,
and batch ID Placeholder. Every journey must use TLS 1.3 mutual authentication,
send encoded KMIP requests to a controlled listener, assert the fixture's
decoded request, and receive and inspect a real encoded KMIP response. The
examples must exercise the new fluent APIs from their respective bindings.

Do not use a legacy facade as a substitute for fluent calls, synthetic in-memory
responses as a substitute for the listener exchange, or tests that only assert
builder construction. Preserve explicit crypto choices, protocol-version
validation, secret redaction, delivery-state semantics, and no-retry behavior.
If an approved fluent operation cannot complete a journey, record the missing
contract and exact evidence for the owning feature; do not invent a protocol
behavior.

## Tasks

### Task 1: Shared fixture contract and symmetric-key journey

Add failing consumer tests for Create → Get → Destroy in all four languages.
The fixture must use a real TLS 1.3/mTLS socket, decode and assert each request,
and return encoded, typed KMIP responses with identifiers that the next request
actually uses. Keep fixture certificates scoped to tests and test-only.

### Task 2: AES-GCM journey

Add failing tests that create an AES key through the fluent API, encrypt with
explicit AES-GCM parameters, and decrypt the returned ciphertext. The fixture
must validate algorithm, key ID, IV, AAD, tag and payload fields and return
encoded operation results; the application must assert plaintext equality.

### Task 3: Key-pair and signature journey

Add failing tests that create a supported key pair, sign known bytes, and
verify the signature using explicit approved algorithm/hash choices. Assert
the decoded request fields, encoded responses, key identifiers and verify
result in each language.

### Task 4: Expert-attributes journey

Add failing tests that insert a catalogued expert attribute using the fluent
API, send it over the controlled fixture, and assert the decoded value and
typed response in all four languages. Include a negative wrong-TTLV-type case
that fails locally before transport.

### Task 5: Batch ID Placeholder journey

Add failing tests for a batch containing Create followed by an operation whose
identifier is a same-batch ID Placeholder. Assert item order, placeholder
resolution semantics, every decoded request payload, and each encoded response
in all four languages. Do not substitute a locally guessed identifier.

### Task 6: Documentation, traceability, and closeout

Publish runnable English and Spanish journey documentation, link the fixture
contract, update F001 traceability with exact test/example paths, and run the
required contract, compatibility, redaction, format, lint, platform, and
coverage checks. Record exact commands and results for the PR.

## Global constraints

- All examples use the typed fluent APIs and the approved KMIP 2.1 default
  unless a journey explicitly selects another approved version.
- TLS is TLS 1.3 with mutual authentication, rustls/aws-lc-rs; never trust any
  certificate in production configurations.
- No request retries, hidden cryptographic defaults, raw KMIP-body logging, or
  secret material in diagnostics.
- C ownership follows existing ABI rules; Java/Python document runtime copies.
- Keep the existing consumer fixture and compatibility suites intact. Add a
  dedicated 0093 journey fixture or extend the existing one only if its
  supported wire protocol and isolation remain clear.
- Red, Green, and Refactor evidence must be separate commits.
