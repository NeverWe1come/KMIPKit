# Implementation Plan: KMIPKit Fluent Object Creation

**Branch**: `feature/KMIPKIT-0092-fluent-objects`  
**Base**: `release/1.0.0` at `2b01ab516e50035eee42fc206396c313dae8d9fb`  
**Spec**: [spec.md](spec.md)  
**Status**: Draft; implementation is gated on human approval of the spec.

## Summary

Extend the KMIPKIT-0091 fluent foundation with Create Key Pair, Create Split
Key, and Register across Rust, C, Java, and Python. Reuse existing typed
operation, managed-object, validation, secret ownership, and execution paths.
Rust remains the single place for fluent construction and validation; the C
ABI carries the capability to Java and Python.

## Technical Context

- **Languages**: Rust 2024/MSRV 1.94, stable C ABI, Java 17/JNI, Python 3.12/CFFI.
- **Dependencies**: Existing `kmipkit-protocol`, `kmipkit-client`, `kmipkit-ffi`,
  public C header, and Java/Python binding packages. No new runtime dependency
  is planned.
- **Storage**: None.
- **Testing**: Focused request-construction and rejection tests, compiled C
  consumers, Java/JNI and Python/CFFI parity tests, and the controlled TLS
  1.3/mTLS KMIP fixture that decodes requests and returns valid typed responses.
- **Target**: Existing supported Linux, macOS, and Windows platforms.
- **Constraints**: Preserve public API/ABI compatibility; KMIP 2.1 default;
  reuse only approved operations and managed-object formats; no raw Request
  Message send path, automatic Poll/retry, local crypto, or implicit usage
  selection; preserve KMIPKIT-0091's RequestOptions and result boundaries.
- **Scope**: Create Key Pair, Create Split Key, and Register fluent access only.
  Cryptographic operations, search, lifecycle expansion, and progressive
  end-to-end documentation remain later deliveries.

## Constitution Check

- **Spec gate**: No production change before a human approves this spec.
- **Traceability**: Each requirement maps to exact KMIP 2.1 clauses/tables,
  implementation paths, and executable verification in `traceability.md`.
- **TDD**: Record distinct failing-test, implementation, and refactor commits;
  use DCO sign-off as required by repository practice.
- **Architecture**: Implement semantic construction in Rust. Keep unsafe code
  in `kmipkit-ffi`, use the existing C ABI for Java/Python, and do not add a
  new transport or serialization implementation.
- **Named values**: Expose semantic enum/value names for the assigned common
  choices so the fluent path does not make callers supply numeric KMIP codes
  or tags. Explicit generic expert inputs remain a separate path.
- **Security**: Reuse secret owners, redaction, and zeroization. Document
  Java/Python runtime-copy limits. Validate all local errors before sending.
- **Review**: Open a `[WIP]` draft PR, wait for every CI check to finish green,
  then remove `[WIP]` and mark the PR ready. Only a human may approve or merge.

## Project Structure

```text
crates/kmipkit-protocol/src/                 # fluent semantic request/object inputs
crates/kmipkit-protocol/tests/               # construction and local rejection tests
crates/kmipkit-client/src/                   # typed builders and execution
crates/kmipkit-client/tests/                 # batch conversion and TLS fixture journeys
crates/kmipkit-ffi/src/                      # additive stable C ABI functions/handles
bindings/c/include/                          # generated public C declarations
bindings/java/src/main/java/org/kmipkit/     # Java facade and fluent builders
bindings/python/src/kmipkit/                  # Python facade and fluent builders
specification/api/public-api.json             # generated public API manifest
docs/user-guide/en/                          # English task examples and ownership notes
docs/user-guide/es/                          # Spanish task examples and ownership notes
```

**Structure Decision**: Add only facade layers on the current typed protocol,
client, and binding paths. Do not add another request dispatcher, wire codec,
transport, or local cryptographic provider.

## Execution Sequence

1. **Red — Rust contract**: Add focused failing tests for preset boundaries,
   Create Key Pair attribute group preservation, Split Key required and
   conditional fields, typed Register object association, invalid inputs, and
   conversion to existing typed requests/batch items.
2. **Green — Rust core**: Implement builders and semantic validation by
   adapting existing request and managed-object models; preserve explicit
   caller choices and current secret ownership.
3. **Refactor — Rust boundary**: Simplify shared fluent conversion, document
   invariants, and verify existing typed APIs remain unchanged.
4. **Red/Green/Refactor — C ABI**: Add focused C consumer failures, opaque
   builder/direct function adapters, ownership/release paths, and stable status
   propagation. Do not add a second error-handle model.
5. **Red/Green/Refactor — Java and Python**: Add idiomatic wrappers and tests
   that demonstrate parity and accurately describe foreign-runtime copies.
6. **End-to-end acceptance**: Exercise all three operations in all four
   languages through the controlled TLS fixture; verify request semantics,
   response correlation, failure behavior, and no automatic follow-up.
7. **Compatibility and publication**: Regenerate/audit exports and the public
   API manifest, update both language guides and traceability, run the
   repository's required checks, then open a WIP PR for CI.

Red, Green, and Refactor evidence will be recorded in separate development
commits as required by `AGENTS.md` and the constitution. Production work does
not begin until this specification is approved.

## Integration Gate

KMIPKIT-0091's implementation PR #172 is now ready for human review but has
not yet merged into `release/1.0.0`. It supplies the fluent namespace and
typed outcome contract this feature extends. Do not start production changes
until PR #172 is merged; then refresh this branch from the new release head.
