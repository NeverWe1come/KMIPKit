# KMIPKIT-0105 Delivery Tasks

## Phase 1 — Specification and contracts

- [x] T001 Verify the KMIPKIT-0102 operation field matrix against all seven builders in `crates/kmipkit-client/src/fluent.rs`; record supported setters without implying universal requiredness in `specs/105-fluent-python-crypto-api/plan.md`.
- [x] T002 Record each Python API and test path in `specs/105-fluent-python-crypto-api/traceability.md`.

## Phase 2 — Shared FFI tests and ownership

- [x] T003 [US1] Add failing Python API-shape and operation identity tests in `bindings/python/tests/test_fluent_crypto.py`.
- [x] T004 [US1] Add failing tests for each operation's supported and prohibited setter surfaces in `bindings/python/tests/test_fluent_crypto.py`.
- [ ] T005 [US2] Add failing native handle, byte staging, and closed-builder tests in `bindings/python/tests/test_fluent_crypto.py`.

## Phase 3 — Operation-specific request construction

- [ ] T006 [US1] Declare the existing crypto builder C ABI in `bindings/python/src/kmipkit/_client_ffi.py`.
- [ ] T007 [US1] Add Python operation-specific crypto builders and `Request` factories in `bindings/python/src/kmipkit/fluent.py`.
- [ ] T008 [US1] Verify typed Cryptographic Parameters, explicit TTLV structures, Data values, and operation-only fields in `bindings/python/tests/test_fluent_crypto.py`.

## Phase 4 — TLS execution and Python typing

- [ ] T009 [US2] Add TLS 1.3 mutual-TLS fixture exchanges for all seven operations in `bindings/python/tests/test_fluent_crypto.py`.
- [ ] T010 [US2] Verify `RequestOptions`, asynchronous indicator, typed batch result, correlation, and no retry/Poll behavior in `bindings/python/tests/test_fluent_crypto.py`.
- [ ] T011 [US3] Add generated API declaration source rows and the contract test in `specification/api/public-api.json` and `tools/api_manifest/tests/test_0105_fluent_python_crypto_api_manifest.py`.
- [ ] T012 [US3] Update the Python type information in `bindings/python/src/kmipkit/py.typed` and both user guides at `docs/user-guide/en/client-execution.md` and `docs/user-guide/es/ejecucion-cliente.md`.

## Phase 5 — Verification and review

- [ ] T013 [US3] Regenerate manifest outputs and verify `tools/api_manifest/generate.py --check` and `tools/api_manifest/audit_binding_exports.py`.
- [ ] T014 Run Python, Rust FFI, C ABI, Java/JNI, formatting, coverage, traceability, and security checks; record exact results in `specs/105-fluent-python-crypto-api/implementation-evidence.md`.
- [ ] T015 Commit Red, Green, and Refactor separately and open/update `[WIP]` draft PR stacked on KMIPKIT-0104; keep WIP until exact-head CI passes.
