# Fluent Create Split Key Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> `superpowers:subagent-driven-development` to execute the tasks with a fresh
> implementer and independent review. Steps use checkbox syntax.

**Goal:** Add a Rust fluent Create Split Key builder using the current typed
request, validation, batch, and client execution paths.

**Architecture:** The builder collects explicit semantic fields and ordered
attributes, then constructs and validates `CreateSplitKeyRequest`. It sends
only `ClientRequest::CreateSplitKey` through the existing client and returns
the existing typed batch response.

**Tech Stack:** Rust 2024/MSRV 1.94, `kmipkit-protocol`, `kmipkit-client`,
controlled TLS 1.3/mTLS fixtures, `tools/api_manifest/generate.py`.

**Spec:** `specs/099-fluent-create-split-key/spec.md`.

## Global Constraints

- Preserve the KMIP v2.1 default and use only the existing typed Create Split
  Key route.
- Add no runtime dependency, generic raw-message send, second codec, or
  second transport.
- Make no cryptographic choice implicitly; do not retry or Poll automatically.
- Keep secrets and raw KMIP bodies out of diagnostics; reuse current
  zeroizing ownership and redaction.
- Run Red, Green, and Refactor in distinct DCO-signed commits.
- Keep the PR `[WIP]` until all required exact-head CI checks are green; only a
  human may approve or merge.

---

## File map

- Modify `crates/kmipkit-client/src/fluent.rs`: public builder, error
  variants, attribute insertion, request construction and execution.
- Modify `crates/kmipkit-client/src/lib.rs`: export the builder and register
  its unit test module.
- Create `crates/kmipkit-client/tests/unit/fluent_create_split_key_tests.rs`:
  behavior tests.
- Modify `specification/api/public-api.json` source records via the repository
  workflow; generated outputs are written only by the pinned generator.
- Modify `docs/user-guide/en/client-execution.md`,
  `docs/user-guide/es/ejecucion-cliente.md`, and feature traceability/evidence.

### Task 1: Confirm source and create the focused test file

**Files:** Create `crates/kmipkit-client/tests/unit/fluent_create_split_key_tests.rs`.

**Interfaces:** Consume existing `CreateSplitKeyRequest`, `SplitKeyMethod`,
`ObjectType`, `UniqueIdentifier`, `AttributeSet`,
`Client::execute_one_typed_request_with_async_indicator`, and test fixture
helpers.

- [ ] **Step 1: Add missing-field tests** for each builder requirement and
  assert that `validate()` and `build_request()` return the same exact
  `FluentBuildError`.
- [ ] **Step 2: Add explicit-field tests** for Object Type, Method, Parts,
  Threshold, optional ID, exact Prime Field Size bytes, optional masks, and
  the always-present Attributes Structure.
- [ ] **Step 3: Add attribute tests** for typed attributes, an explicit
  unknown `Item`, a correctly structured Vendor Attribute, positioned
  insertion, duplicate/out-of-range positions, and `0x540000`/`0x548000`/
  `0x54FFFF` rejection.
- [ ] **Step 4: Add batch and exchange tests** for one request, typed result
  access, repeated IDs and their order, Success, Failure, Pending, exact
  correlation, options, delivery state, and no retry/Poll.
- [ ] **Step 5: Run the Red command**:
  `cargo test -p kmipkit-client --lib fluent_create_split_key_tests`.
  Expected: compile errors naming the missing `create_split_key` builder and
  its fluent types/methods.
- [ ] **Step 6: Commit the specification and tests with DCO**:
  `git add specs/099-fluent-create-split-key crates/kmipkit-client/tests/unit/fluent_create_split_key_tests.rs`
  then `git commit -s -m 'test: specify fluent create split key'`.

### Task 2: Implement the typed builder

**Files:** Modify `crates/kmipkit-client/src/fluent.rs` and `src/lib.rs`.

**Interfaces:** Consume the existing typed protocol model; produce
`RequestBuilder::create_split_key() -> CreateSplitKeyBuilder<'a>`, the public
`CreateSplitKeyBuilder`, and four `FluentBuildError` variants for missing
Object Type, Method, Parts, and Threshold.

- [ ] **Step 1: Add public error variants and stable messages**, then add
  `CreateSplitKeyBuilder` fields for the client, four required values,
  optional Unique Identifier, Prime Field Size bytes, Protection Storage
  Masks, AttributeSet, positioned Items, saved attribute error, RequestOptions,
  and asynchronous indicator.
- [ ] **Step 2: Add semantic setters** with signatures
  `object_type(ObjectType)`, `method(SplitKeyMethod)`, `parts(i32)`,
  `threshold(i32)`, `unique_identifier(UniqueIdentifier)`,
  `prime_field_size(Vec<u8>)`, and
  `protection_storage_masks(Structure)`.
- [ ] **Step 3: Add ordered attribute setters**:
  `attribute(Attribute<D>)`, `attribute_item(Item)`,
  `attributes(AttributeSet)`, `vendor_attribute(&str, &str, &Item)`, and
  `attribute_at(FluentAttributePosition, Item)`. Reject message-extension
  tags and map invalid values to existing payload-free errors.
- [ ] **Step 4: Implement local construction** by requiring the four semantic
  fields, cloning valid stored Items into `AttributeSet`, applying positions
  against the final list, constructing `CreateSplitKeyRequest`, and running
  `ValidateKmipRequest::validate()` before returning it.
- [ ] **Step 5: Implement public methods**:
  `validate(&self)`, `build(self)`, `build_request(&self)`,
  `to_batch_item(self)`, `with_options(RequestOptions)`,
  `with_asynchronous_indicator(u32)`, and
  `execute(&CodecLimits) -> Result<ClientBatchItemResponse, ClientError>`.
  Execution must construct `ClientRequest::CreateSplitKey` and call the
  existing typed one-exchange method.
- [ ] **Step 6: Register and export the builder** in `src/lib.rs`; run
  `cargo test -p kmipkit-client --lib fluent_create_split_key_tests` and
  require all focused tests to pass.
- [ ] **Step 7: Run** `cargo test -p kmipkit-client --all-features` and commit
  production changes separately with
  `git commit -s -m 'feat: add fluent create split key builder'`.

### Task 3: Document and generate the public surface

**Files:** Modify the English/Spanish guides, feature traceability/evidence,
and public API manifest inputs; regenerate checked-in output using repository
tools.

**Interfaces:** Document the exact Task 2 signatures. Use
`ObjectType::from_raw(kmipkit_protocol::named_values_generated::values::enumeration::object_type::SYMMETRIC_KEY)`
and `SplitKeyMethod::XOR` in the example so no KMIP numeric values or tags
are required.

- [ ] **Step 1: Add guide examples** showing a Split Key request, explicit
  choices, optional Unique Identifier and the returned IDs from the typed
  response. State that no split cryptography or follow-up Poll occurs.
- [ ] **Step 2: Add traceability** from each FR to OASIS §6.1.10 Tables
  193–195, KMIPKIT-0093 FRs, exact builder paths, and test names.
- [ ] **Step 3: Add public API records** for the builder, all public methods,
  and new errors; preserve existing records and identify foreign-language
  mappings as pending until their own slice.
- [ ] **Step 4: Regenerate** with `python3 tools/api_manifest/generate.py
  --write`; run its `--check` mode and the Rust export audit. Do not edit
  generated outputs directly.
- [ ] **Step 5: Commit Refactor/docs/generated evidence** separately using
  `git commit -s -m 'docs: publish fluent create split key API'`.

### Task 4: Verify, review, and publish

**Files:** No new files; use the focused feature files and PR #179 stack.

**Interfaces:** The PR base is `feature/KMIPKIT-0097-fluent-object-api` at
`45749b62e355ceea1ac6c936ec4a65b085050d60` unless the parent CI fix changes
that head; if it does, rebase this branch onto its new published head first.

- [ ] **Step 1: Run format and lint** with
  `cargo fmt --all --check` and
  `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`.
- [ ] **Step 2: Run feature suites** with
  `cargo test -p kmipkit-client --all-features` and
  `python3 scripts/test_user_guide_examples.py`.
- [ ] **Step 3: Run manifest and traceability** with the feature's focused
  API-manifest test, `python3 tools/api_manifest/generate.py --check`,
  and `python3 tools/api_manifest/audit_rust_exports.py`.
- [ ] **Step 4: Run** `git diff --check` and the repository changed-code
  coverage gate; resolve failures without lowering thresholds.
- [ ] **Step 5: Request an independent task review** and resolve any findings
  with a scoped re-review.
- [ ] **Step 6: Open a `[WIP]` draft PR**, run exact-head hosted CI, fix every
  failed required job, and keep the PR WIP until all required checks are
  green. Never approve or merge it.
