# Validation Quickstart: KMIPKIT-0040

These are implementation checks, not evidence of formal conformance, interoperability, or PKCS#11 Profile support. Run at repository root with Rust 1.94.

## Focused checks

    cargo test -p kmipkit-protocol pkcs11_operation_tests
    cargo test -p kmipkit-client pkcs11_execution_tests

The first suite covers Tables 273/274, optional absence, Function Enumeration/Integer preservation, generic structures, malformed shapes, and redaction. The fake-transport suite proves operation selection, one exchange, result/Pending/correlation/delivery preservation, and no retry.

## Review gates

    cargo fmt --all --check
    cargo clippy --workspace --all-targets --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo llvm-cov --workspace --all-features
    python tools/normative_catalog/validate.py
    python tools/normative_catalog/report.py --check
    git diff --check

Apply the exact gates in docs/development/testing.md: 95% line coverage for the protocol crate and changed production code, 90% Rust workspace coverage, and 100% normative traceability. Apply the 85% per-scope adapter/transport thresholds when those scopes change.

## 1.0 interoperability release gate

Before 1.0, exercise this operation against at least two independent KMIP server implementations and record product/version, transport, profile, operation, result, and workaround in docs/development/kmip-interoperability.md and the test report, following docs/development/testing.md. A server that does not support the operation is recorded as SKIP/N/A with its reason; that result is not a passing interoperability test and leaves the release gate open.


## Implemented Rust surface

`Pkcs11Request::new(function_structure)` requires an explicit Function
Structure. `with_interface`, `with_correlation_value`, and
`with_input_parameters` preserve optional presence; `with_extension` preserves
generic unknown fields. `into_ttlv_payload` validates and consumes the request.
The client offers `ClientRequest::pkcs11`, `Client::pkcs11`, and
`Client::pkcs11_with_options` through the shared one-exchange path.

`Pkcs11Response::result` exposes common status/reason/message metadata.
Callback accessors expose the optional Interface, Function, Correlation Value,
Output Parameters and Return Code without formatting their contents.
`with_ttlv` retains the entire generic response batch item and
`with_ttlv_payload` retains the original payload, including unknown fields.
Failure/Pending results do not fabricate required Success fields.

Owned opaque TTLV storage and `SecretBytes` use the existing recursive
zeroization contract. Callbacks permit deliberate caller-owned copies, whose
lifetime is controlled by the caller. No PKCS#11 profile codec is added.

The source-derived protocol suite contains 15 tests and the fake-client suite
contains 9 tests. Exact names, sources and release gates are in
`traceability.md`; command results and TDD commits are in
`implementation-evidence.md`. The four-language T019 mapping is implemented
on the KMIPKIT-0043 feature branch. No official PKCS#11 fixture or real
two-server pass is claimed.

## Four-language public mapping

The public request/response mapping follows the approved-for-implementation
KMIPKIT-0040 contract. The maintainer's direct instruction dated 2026-10-20
authorized its implementation; this does not change the specification's
approval status or constitute formal acceptance (`formalAcceptance: false`).
The APIs below forward the KMIP operation and preserve opaque values; they do
not parse the PKCS#11 Profile.

- **Rust**: `Pkcs11Request`, `ClientRequest::pkcs11`, `Client::pkcs11`, and
  `Pkcs11Response::with_ttlv` / `with_ttlv_payload`.
- **C**: `kmipkit_client_batch_item_pkcs11` constructs the request and
  `kmipkit_client_batch_item_response_pkcs11_ttlv` returns an owned copy of
  successful response TTLV for caller release.
- **Java**: `ClientBatchItem.pkcs11` constructs the request;
  `ClientBatchItemResult.pkcs11Response()` returns an owned
  `Pkcs11Response` with `responseTtlv()` and `close()`.
- **Python**: `client_batch_item_pkcs11` constructs the request;
  `BatchItemResult.pkcs11_response` returns an owned `Pkcs11Response` with
  `response_ttlv` and `close()`.

The shared successful-response fixture checks the complete Batch Item Response
wrapper, then the five ordered payload fields, unknown Return Code and vendor
extension preservation, optional absence, redaction, ownership after source
closure, and no automatic retry or follow-up. Rust, compiled C, Java/JNI, and
Python/CFFI test names and command results are in `traceability.md` and
`implementation-evidence.md`. These are local adapter tests, not
interoperability evidence.


The focused suites also exercise §§7.27–7.29 known nested types/cardinality,
local NotSent rejection and received malformed nested Output Parameters.
Function's operation-specific Enumeration/vendor Integer context is explicitly
traced in `research.md` as `KMIPKIT-026-SOURCE-001`.
