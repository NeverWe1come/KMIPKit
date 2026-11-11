# KMIPKIT-0105 Implementation Plan

## Design decisions

1. Keep protocol assembly and validation in the Rust-owned opaque C builder
   added by KMIPKIT-0103. Python is an idiomatic facade over that ABI.
2. Add a shared private Python builder implementation and seven public
   operation-specific classes. Each class exposes only the fields its typed
   Rust builder accepts.
3. Declare the existing ABI symbols in the Python CFFI declaration module;
   do not add C header symbols or duplicate protocol validation in Python.
4. Copy byte-like input to bounded mutable staging storage, call the native
   ABI synchronously under the binding lock, and clear the staging storage in
   `finally`. Python immutable values and external runtime copies remain the
   caller's responsibility.
5. Build a typed `ClientBatchItem` and return the existing `BatchResult` from
   one exchange. Keep `RequestOptions` separate from the explicit Request
   Header asynchronous indicator.
6. Extend the Python `py.typed` package surface and manifest source; regenerate
   generated outputs only through the repository API-manifest generator.

## Operation field matrix

| Builder | Semantic fields accepted by the typed request |
|---|---|---|
| Encrypt | optional Unique Identifier, Cryptographic Parameters, Data, IV/Counter/Nonce, AAD, correlation, Init/Final |
| Decrypt | optional Unique Identifier, Cryptographic Parameters, Data, IV/Counter/Nonce, AAD, authentication tag, correlation, Init/Final |
| Sign | optional Unique Identifier, Cryptographic Parameters, Data, digested data, correlation, Init/Final |
| Signature Verify | optional Unique Identifier, Cryptographic Parameters, Data, digested data, signature data, correlation, Init/Final |
| MAC | optional Unique Identifier, Cryptographic Parameters, Data, correlation, Init/Final |
| MAC Verify | optional Unique Identifier, Cryptographic Parameters, Data, MAC data, correlation, Init/Final |
| Hash | Cryptographic Parameters, Data, correlation, Init/Final |

This table records supported setters, not universal requiredness. Required and
conditional fields are governed by the existing KMIPKIT-0102 request validator
and cited KMIP operation tables. The matrix was checked against the seven
Rust builders in `crates/kmipkit-client/src/fluent.rs`; tests must assert the
same operation-specific surfaces.

## Test approach

Start with consumer-facing tests that fail because the seven request methods
and builders are absent. Add operation identity and invalid-input tests, then
TLS fixture tests over the production Python client and C ABI. Cover native
ownership, staged-buffer clearing, options/header separation, and no
automatic retry or Poll. Run focused Python tests before broader binding,
manifest, Rust/C/JNI, coverage, and CI gates.

## Release and review

Keep the branch based on KMIPKIT-0104 while it is under review. Create separate
Red, Green, and Refactor commits. Push a `[WIP]` draft PR stacked on the Java
branch, and keep it WIP until every required CI job is green on its exact head.
Only request human review; never approve or merge.
