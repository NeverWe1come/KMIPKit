# KMIPKIT-0116 Implementation Plan

## Architecture

Add thin operation-specific builder objects over the existing typed request
models and `ClientRequest` execution routes. The builders must share the
existing one-exchange execution, options, version gate, outcomes, and opaque
ownership mechanisms. Language-specific surfaces must use the current C ABI,
JNI, and CFFI rather than adding another protocol implementation.

## Delivery order

1. Add consumer-shape and runtime tests for all seven operations in Rust, C,
   Java, and Python; record expected Red failures.
2. Implement Rust typed builders and version-gated execution.
3. Add C opaque builders/direct helpers and typed results on the Rust ABI
   owner; verify fixed-width errors and release behavior.
4. Add Java builders and JNI mapping, then Python builders and PEP 561 stubs.
5. Reconcile the public manifest and capability matrix, regenerate outputs,
   update English and Spanish guides, and add controlled request/response
   examples.
6. Run focused suites, full language suites, coverage, generation, export,
   traceability, sanitizers, and exact-head CI. Keep the PR `[WIP]` until every
   required check passes.

The feature stacks on KMIPKIT-0115 because its review/merge may be pending;
human review and merge remain separate acceptance steps.
