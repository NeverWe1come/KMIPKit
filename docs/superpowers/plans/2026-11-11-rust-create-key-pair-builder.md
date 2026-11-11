# Rust Create Key Pair Builder Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Rust fluent builder that constructs, batches, and executes a typed KMIP Create Key Pair request.

**Architecture:** The builder lives in `kmipkit-client::fluent`, creates the existing protocol `CreateKeyPairRequest`, and executes via the existing typed client writer. Attribute groups remain separate `AttributeSet` values and use generated typed attribute markers; no new codec or transport path is introduced.

**Tech Stack:** Rust 2024, `kmipkit-protocol` typed requests and attributes, `kmipkit-ttlv` values, existing client transport fixtures, public API manifest generator.

**Spec:** `specs/097-fluent-create-key-pair/spec.md`.

## Global Constraints

- The default effective protocol version remains KMIP 2.1.
- Cryptographic choices MUST remain explicit to callers.
- No public `Client::execute(Item)`, raw-message send, or arbitrary transport constructor is introduced.
- One explicit send performs at most one exchange; there is no automatic retry or Poll.
- Public APIs MUST be documented; `rustfmt` and Clippy warnings MUST pass.
- Preserve existing APIs, secret redaction, request-delivery state, response ownership, and generated API declarations.

---

### Task 1: Specify the Red tests

**Files:**
- Create: `crates/kmipkit-client/tests/unit/fluent_create_key_pair_tests.rs`
- Modify: `crates/kmipkit-client/src/lib.rs` to include the test module under `#[cfg(test)]` using the existing test-path convention.

**Interfaces:**
- Spec: `specs/097-fluent-create-key-pair/spec.md`.
- The expected API is `Client::request().create_key_pair().algorithm(CryptographicAlgorithm).length(i32)` plus independent `private_usage_mask`, `public_usage_mask`, `private_name`, and `public_name` setters.
- Construction returns the existing `CreateKeyPairRequest`; batch conversion returns `ClientBatchItem`; attached execution returns `ClientBatchItemResponse`.

- [ ] Write tests that assert missing algorithm and missing length produce distinct payload-free `FluentBuildError` values.
- [ ] Write tests that assert common algorithm/length, private usage/name, and public usage/name serialize in their exact Common, Private Key, and Public Key groups.
- [ ] Write a Table 191 test where conflicting private/public algorithm values fail before transport and report `RequestDeliveryState::NotSent`.
- [ ] Run `cargo test -p kmipkit-client --lib fluent_create_key_pair_tests --locked`; expected result is compile failure because the builder and method are not yet defined.
- [ ] Commit only the failing tests with `git commit -s -m "test: specify fluent create key pair"`.

### Task 2: Implement the typed builder

**Files:**
- Modify: `crates/kmipkit-client/src/fluent.rs`.
- Modify: `crates/kmipkit-client/src/lib.rs` to export `CreateKeyPairBuilder` and include the test module.
- Test: `crates/kmipkit-client/tests/unit/fluent_create_key_pair_tests.rs`.

**Interfaces:**
- Consumes: existing `CreateKeyPairRequest`, `AttributeSet`, `Attribute<D>`, `ClientRequest::CreateKeyPair`, and `Client::execute_one_typed_request_with_async_indicator`.
- Produces: `RequestBuilder::create_key_pair()` and public `CreateKeyPairBuilder` with `algorithm`, `length`, separate key group setters, `with_options`, `with_asynchronous_indicator`, `build`, `build_request`, `to_batch_item`, and `execute`.

- [ ] Add a builder that stores the optional client, explicit algorithm/length, common/private/public attributes, per-exchange `RequestOptions`, and optional asynchronous indicator.
- [ ] Require algorithm and length; construct their typed attributes in Common Attributes.
- [ ] Add each usage mask and name only to the explicitly selected private or public group; leave them absent unless the caller supplies them.
- [ ] Build the existing `CreateKeyPairRequest`, call its existing validator, and map local failure to payload-free `FluentBuildError`/`ClientError` with `RequestDeliveryState::NotSent`.
- [ ] Implement batch conversion and one-exchange execution through the existing client method.
- [ ] Run the focused Red test command and the Create Key Pair protocol/client unit tests; expected result is pass.
- [ ] Commit production code separately with `git commit -s -m "feat: add fluent create key pair builder"`.

### Task 3: Complete the API contract and verify

**Files:**
- Modify: `docs/user-guide/en/client-execution.md`.
- Modify: `docs/user-guide/es/ejecucion-cliente.md`.
- Modify: `specification/api/public-api.json` using the repository generator source and generated outputs.
- Modify: `specs/097-fluent-create-key-pair/tasks.md` and `traceability.md`.

**Interfaces:**
- Consumes: public builder signatures from Task 2.
- Produces: a documented Rust example for explicit key-pair algorithm, length, and independent private/public usages.

- [ ] Add equivalent English and Spanish snippets showing explicit values and a call to `execute`.
- [ ] Add all public builder declarations to the API manifest source; run `python3 -B tools/api_manifest/generate.py --write` and then `--check`.
- [ ] Run `cargo fmt --all -- --check`, `cargo clippy -p kmipkit-client --all-targets --all-features --locked -- -D warnings`, `cargo test -p kmipkit-client --all-targets --locked --quiet`, the relevant traceability tests, and API export checks.
- [ ] Run `git diff --check`; commit documentation and manifest changes with `git commit -s -m "docs: publish fluent create key pair API"`.
- [ ] Open a `[WIP]` draft PR based on the KMIPKIT-0096 head, keep it WIP until all required checks are green, and fix every red check before marking it ready.

## Spec coverage self-review

- FR-001 and FR-002: Tasks 1 and 2.
- FR-003: Tasks 1 and 2.
- FR-004: Tasks 1 and 2.
- FR-005: Task 3.
- The plan has no raw request encoder, automatic cryptographic defaults, or automatic follow-up operation.
