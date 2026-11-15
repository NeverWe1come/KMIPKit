# KMIPKIT-0091 Fluent DX Foundation Implementation Plan

> **For agentic workers:** Use this plan task-by-task after maintainer approval of `specs/091-fluent-dx-foundation/spec.md`. Keep all implementation on the `feature/KMIPKIT-0091-fluent-dx` branch.

**Goal:** Let Rust, C, Java, and Python callers create a symmetric key, retrieve it by ID, and destroy it through task-oriented typed APIs without assembling KMIP payloads or message batches themselves.

**Architecture:** Keep semantic construction and validation in Rust protocol/client crates. Expose it through the existing C ABI; Java and Python map the same calls into idiomatic facades. Existing typed operations remain the only network execution boundary.

**Tech Stack:** Rust 2024, KMIPKit C ABI, Java 17/JNI, Python 3.12/CFFI, public API manifest v2, existing loopback TLS 1.3/mTLS consumer fixture.

**Spec:** `specs/091-fluent-dx-foundation/spec.md`

## Global Constraints

- Preserve all existing public APIs and stable C ABI symbols.
- Default to KMIP 2.1 and use the existing protocol-version gate.
- Use typed requests only at client execution boundaries.
- Keep security choices explicit; do not retry or poll automatically.
- Redact secrets and use existing zeroizing owners and handle lifetimes.
- Update the API manifest and generated outputs with every new public export.

---

### Task 1: Define semantic attributes and symmetric-key request in Rust

**Files:** `crates/kmipkit-protocol/src/`; matching protocol integration tests.

**Interfaces:** Add a fluent attribute accumulator that produces existing
`AttributeSet` and a `CreateSymmetricKeyBuilder` that produces existing
`CreateRequest`. Add typed setters for algorithm, length, usage, Name, and
catalogued values. Unknown fields require an explicit `Item`; Vendor Attribute
has a constructor for its required structure.

- [ ] Write failing unit tests for required algorithm/length/usage, typed attribute mismatch, unknown explicit Item, Vendor Attribute validity/order, and repeated Name.
- [ ] Run only those tests and record expected missing behavior.
- [ ] Implement minimal Rust protocol builders using existing model validators and named values.
- [ ] Run focused protocol tests; retain payload-free errors and no secret formatting.
- [ ] Commit the Red and Green evidence separately with DCO sign-off.

### Task 2: Add bound Rust client request builders

**Files:** `crates/kmipkit-client/src/`; execution and boundary tests.

**Interfaces:** Add `Client::request(&mut self)` and builders for Create
Symmetric Key, Get by ID, and Destroy by ID. Each builder supports `build`,
`to_batch_item`, `with_options`, and `execute`; execution delegates to existing
typed methods and returns the existing typed response outcome.

- [ ] Add failing tests for Create/Get/Destroy execution, conversion into an existing batch, Success/Pending/Failure propagation, and per-exchange options.
- [ ] Run each focused test and confirm it fails for the missing route.
- [ ] Implement the bound dispatcher and builders without introducing generic TTLV or raw-byte inputs to execution.
- [ ] Run execution-boundary, version-gate, request-validation, outcome, and redaction tests.
- [ ] Commit Red, Green, and Refactor evidence with DCO sign-off.

### Task 3: Expose versioned C helpers and opaque builder

**Files:** `crates/kmipkit-ffi/src/`, `bindings/c/include/kmipkit.h`, C consumer tests, API manifest inputs.

**Interfaces:** Add `kmipkit_symmetric_key_request_v1_t`, direct Create/Get/Destroy
helpers, and the opaque symmetric-key builder operations defined in the spec.
Use existing status codes, last-error snapshots, release functions, and
borrowed-input copying rules.

- [ ] Add failing compiled C consumer tests for simple direct requests, builder construction, invalid struct metadata, type mismatch, ownership, and cleanup.
- [ ] Run the focused C ABI consumers and record the expected missing-symbol/behavior failure.
- [ ] Implement FFI calls as adapters over Rust builders and existing typed client routes.
- [ ] Update manifest v2 and generated C/JNI/Python inventories through the pinned generator.
- [ ] Run C header and binary export audits plus focused C tests.
- [ ] Commit Red and Green evidence with DCO sign-off.

### Task 4: Add Java facade

**Files:** `bindings/java/src/main/java/org/kmipkit/client/`, `org/kmipkit/fluent/`, native declarations, and facade tests.

**Interfaces:** Add `org.kmipkit.client.KmipClient`, typed Create/Get/Destroy
builders, `build`, `toBatchItem`, `withOptions`, and `execute`. Retain the
existing `org.kmipkit.extensions` API. Use `AutoCloseable` for native owners
and existing exception classes.

- [ ] Add failing Java/JNI consumer tests for the three operations, outcomes, options, validation, cleanup, and compatibility imports.
- [ ] Run the focused Gradle/Maven tests and record expected missing members.
- [ ] Implement Java mappings to the shared C ABI calls; add no Java KMIP serializer or independent validation policy.
- [ ] Run Java facade tests and JNI public export audit.
- [ ] Commit Red, Green, and Refactor evidence with DCO sign-off.

### Task 5: Add Python fluent and named-argument facade

**Files:** `bindings/python/src/kmipkit/fluent.py`, public client facade, typed declarations, tests, API manifest inputs.

**Interfaces:** Add `client.request().create_symmetric_key()`, `get(id)`, and
`destroy(id)` builders plus direct named-argument convenience methods. Keep
existing `extensions` functions. Native-backed objects support `close()` and
context management.

- [ ] Add failing pytest cases for the operations, named arguments, outcomes, options, type mismatch, and closed-handle behavior.
- [ ] Run focused tests and record expected missing API failures.
- [ ] Implement Python wrappers over the shared C ABI calls.
- [ ] Run Python facade tests, type checks, and export audit.
- [ ] Commit Red, Green, and Refactor evidence with DCO sign-off.

### Task 6: Prove four-language parity and update docs

**Files:** `tests/consumers/0053-parity/`, English and Spanish task guides, spec evidence, API manifest and generated outputs.

- [ ] Extend each external package consumer to perform Create/Get/Destroy against the loopback TLS 1.3/mTLS fixture and assert request fields and typed outcomes.
- [ ] Add shared semantic parity cases for default Name Type, catalogued attribute types, explicit unknown Items, Vendor Attribute order, and no automatic Poll/retry.
- [ ] Run legacy API compatibility consumers and the public export audit across Rust, C, Java, and Python.
- [ ] Update English and Spanish examples, operation matrix, traceability, and implementation evidence.
- [ ] Run workspace formatting, Clippy, required Rust/C/Java/Python suites, coverage gates, generator checks, and applicable security checks.
- [ ] Commit final Refactor and evidence with DCO sign-off; open a draft PR against `release/1.0.0`.
