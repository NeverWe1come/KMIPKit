# Quickstart Validation: Fluent Object Creation

## Prerequisites

- A build of KMIPKit with the approved KMIPKIT-0091 Fluent DX foundation.
- Rust, C, Java, or Python development tools for the language being checked.
- The repository's controlled TLS 1.3/mTLS fixture and its test certificates.
- An in-memory synthetic KMIP server response for each scenario. A response
  returned only by a local fake transport is not end-to-end fixture evidence.

## Scenarios

Run these independently for Rust, C, Java, and Python:

1. **Create Key Pair**: Select one explicit preset, set distinct private and
   public usage masks and a human-readable name, execute once, then inspect
   both returned identifiers. Confirm the fixture received the exact
   algorithm/parameters and did not receive a usage mask that the caller did
   not select.
2. **Create Split Key**: Select an object type, method, part count, and
   threshold. For Polynomial Sharing Prime Field, supply Prime Field Size.
   Execute once and verify every response identifier is returned in wire
   order.
3. **Register**: Build one approved typed managed object with secret-bearing
   test bytes, attach attributes, and execute once. Confirm object type,
   Key Format Type, Key Material, and attributes arrive unchanged. Release the
   object and verify the documented ownership and redaction behavior.
4. **Negative requests**: Omit or mismatch required key-pair values, omit the
   method-required split Prime Field Size, use a mismatched Register object
   type, and select an unsupported effective protocol version. Each case
   must fail before fixture acceptance and record NotSent delivery state.
5. **Outcome handling**: Return valid Success, Pending, and Failure results
   from the fixture. Confirm the typed result and correlation remain
   inspectable and that no Poll or second request is sent automatically.

## Expected Results

- Every language sends typed operation payloads through one TLS fixture
  exchange for each explicit execute call.
- Semantic selections match the decoded request exactly.
- Invalid local inputs produce no network write.
- The three operation results retain required IDs, status, correlation, and
  delivery evidence.
- Secret bytes do not appear in formatted requests or errors. C releases
  library-owned handles; Java and Python report their runtime-copy limits.

## Planned Test Locations

Use the paths and tests named in [traceability.md](traceability.md). The final
implementation quickstart must add the exact runnable command for each suite
and record actual results in the draft PR.
