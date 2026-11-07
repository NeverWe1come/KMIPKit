# Implementation Plan: KMIPKIT-0052 Response Navigation

**Branch**: `feature/KMIPKIT-0052-typed-responses` | **Base**: `feature/KMIPKIT-0051-pre-send-validation` | **Spec**: `specs/052-response-navigation/spec.md`

## Summary

Close the typed-response and generic-inspection gap using one owned source tree per decoded response message. Add four missing typed readers for asynchronous control/results, direct Rust access to a response payload and header as borrowed `StructureView`s, safe nested-Structure navigation, and equivalent owned raw-tree handles in C, Java, and Python. Keep operation parsing and wire semantics unchanged.

## Normative and Scope Decisions

- The pinned KMIP v2.1 Standard is the normative source. `gap-matrix.md` maps the 56 supported operations to their exact response table or explains why a dedicated table is absent.
- §8.6 Table 399 determines Response Payload presence by result shape. An absent payload returns `None`; it is never synthesized.
- §10.1.2 determines source field order. The raw tree preserves the decoder's ordered structure; it does not reproduce padding bytes.
- §9.13 and §11.56 govern extension semantics and the `0x54xxxx` extension tag range. Generic inspection does not assign semantics to unknown extension fields.
- Table 286's caption/content discrepancy remains generic and is explicitly covered by the approved source note; this feature does not resolve `KMIPKIT-DISC-039`.
- Set Endpoint Role (§6.1.54) and §6.2 server-initiated operations remain out of scope.

## Current-State Gap and Acceptance Contract

- The `ClientResponseView` in `crates/kmipkit-client/src/execute.rs` has typed accessors for 52 of the 56 in-scope operation identities.
- Cancel, Poll, Process, and Query Asynchronous Requests have protocol response models, but `ClientOperationOutcome` has no matching typed readers.
- The standard `ClientBatchItemResponse` contains the typed outcome but not the source `ResponseMessage`; the message tree is dropped after conversion. Its typed models retain different subsets of TTLV, so no common raw payload view exists.
- `ClientOperationOutcome` retains a `ResponseMessage` and has a callback payload accessor, but not a direct borrowed view or the four typed response readers.
- `ResponseHeaderView::with_ttlv` is callback-based. Generic nested Structure traversal is possible through value callbacks, but lacks a direct borrow tied to the owning Item.
- C, Java, and Python have no common owned Response Payload tree handle on the high-level client response result.

The acceptance contract is 56/56 operation result identities, 0 synthesized payloads, 4/4 foreign-language raw-tree consumers, nested unknown `0x54xxxx` fields retained at arbitrary depth, and no payload disclosure through automatic formatting.

## Architecture

1. **Shared message ownership**: convert the decoded `ResponseMessage` into `Arc<ResponseMessage>` before producing batch item results. Each private `ClientBatchItemResponse` stores an `Arc` clone and its response item index. This retains one response tree for the lifetime of the returned response items without duplicating every payload. Dropping the last owner releases the `ResponseMessage` and its zeroizing TTLV tree.
2. **Safe borrowed raw views**: add direct, read-only accessors on `ResponseBatchItemView` and `ResponseHeaderView` that return `StructureView<'_>` tied to the message borrow. Add `ClientBatchItemResponse::raw()` for the Response Payload and `ClientBatchResponse::response_header()` for the header. Add `ClientOperationOutcome::raw()` as the direct-view counterpart to its existing callback API.
3. **Typed asynchronous results**: add `ClientOperationOutcome` readers returning `CancelResponse`, `PollResponse`, `ProcessResponse`, and `QueryAsyncRequestsResponse` borrowed from the owned response message. Require that the `ClientOperation` matches the requested typed reader; preserve existing generic payload methods.
4. **Nested navigation**: add a safe `Item::as_structure()` borrowed accessor and a focused nested traversal helper so callers can descend from any raw payload to an arbitrary Structure without copying or flattening. Preserve order, duplicates, unknown allocated tags, item type, and values.
5. **FFI ownership**: C returns an explicitly owned `kmipkit_ttlv_structure_t` copy for a raw response payload/header and uses existing release APIs. Java wrappers close the owned structure handle with `AutoCloseable`; Python wrappers expose `close()` and context-manager support. All copies use the zeroizing TTLV owner.
6. **Manifest and docs**: reconcile every new public route through the reviewed API manifest and pinned generator. Document lifetime, no-padding-replay, redaction, and foreign-runtime copy limitations in English and Spanish.

This supersedes the previous prototype choice of cloning a separate payload into every typed operation model: the outer response result owns an `Arc` to the one decoded message tree, avoiding N payload copies across a batch while giving each item a stable lifetime.

## TDD and Commit Sequence

For each behavior slice, commit in this order:

1. **Red**: failing tests for response raw access, absent payload, nested Structure traversal, async response readers, header access, FFI lifetime, or language consumer behavior.
2. **Green**: minimal implementation for those tests.
3. **Refactor**: consolidate repeated dispatch/handle conversion only after focused and neighboring tests pass.

No production code precedes its failing behavior test. Source-level schema matrices and generated artifacts are updated only after tests establish their output contract.

## Verification Strategy

- Exhaustive compile-time/runtime coverage for all 56 operation identities and exact response table map.
- Status matrix: Success with payload, Success with empty Structure payload, Pending/no payload, Failure/no payload, and operation-specific optional payload cases.
- Unknown/repeated standard and vendor fields, future Enumeration values, deep nesting, Key Block, Server Information, and response header source order.
- Real Rust, compiled C, JNI Java, and CFFI Python consumers; close/release idempotence and owner drop.
- Redaction canaries in Rust `Debug`, C error formatting, Java `toString`, and Python `repr`.
- `cargo fmt`, Clippy, workspace tests, binding tests, normative traceability, manifest `generate.py --check`, Rust export audit, compatibility checks, and local coverage when coverage gates are re-enabled. Remote CI remains subject to the maintainer's billing instruction.

## Risks and Mitigations

- **Arc Send/Sync boundary**: keep `Arc<ResponseMessage>` private and make no cross-thread promise unless the type's actual auto traits support it.
- **Source lifetime exposure**: all Rust views borrow the owning response; compile-fail docs/tests prevent views escaping.
- **Sensitive response values**: preserve the existing zeroizing `Structure` owner; ensure all new FFI clones use that type and format only metadata.
- **Schema discrepancy**: Query Asynchronous Requests remains generic under the exact Table 286 source note; do not parse it based on the mislabeled caption.
- **Generated API volume**: use the official manifest workflow and bounded batches; never add a broad Rust parity exception.

## Dependencies

- Stacked code dependency: KMIPKIT-0051 / PR #122.
- Existing typed model baseline: KMIPKIT-0050 / PR #118.
- Foreign-language raw TTLV primitives: KMIPKIT-0047–0049 ancestors and current C/JNI/CFFI wrapper generators.
- Existing TTLV `Structure`, `StructureView`, redaction, and zeroization contracts.
