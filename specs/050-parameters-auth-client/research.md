# Research: KMIPKIT-0050

## Existing API audit

- `ClientOperation` in `crates/kmipkit-client/src/execute.rs` has 56 values. It includes four asynchronous control/query operations: Poll, Cancel, Process, and Query Async Requests.
- `ClientRequest` has 52 typed request variants. The asynchronous four already have direct methods (`execute_poll`, `execute_cancel`, `execute_process`, and `execute_query_async_requests`).
- The current `Client` surface lacks direct methods for Discover Versions, Delegated Login, Login, Logout, Get, Locate, Add Attribute, Adjust Attribute, Delete Attribute, Modify Attribute, Set Attribute, Get Attributes, Encrypt, and Decrypt.
- Ten existing direct methods have no `_with_options` form: Get Constraints, Get Attribute List, Set Constraints, Set Defaults, Get Usage Allocation, Validate, RNG Retrieve, Log, Interop, and RNG Seed.
- `RequestOptions`, batch response types, delivery-state errors, and fake-transport execution are already available and remain the shared implementation path.

## Cryptographic Parameters

- The protocol crate has a private shared validator in `src/cryptographic_parameters.rs`, but no public typed `CryptographicParameters` type.
- The current validator recognizes all 18 Table 59 member tags and validates member types/singleton cardinality. Existing operation setters accept generic `Structure` in Encrypt, Decrypt, Derive Key, Sign, Signature Verify, MAC, MAC Verify, and key-wrapping key information.
- `DerivationParameters` is separately typed but its optional Cryptographic Parameters value remains generic `Structure`.
- The new public model should reuse the existing validation rules, add known-field order validation from §10.1.2, expose exact raw enum values, and preserve unknown items. It must not make algorithms or server decisions implicitly.
- Pinned-source verification: `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, SHA-256 `8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF`. Table 59's exact order is Block Cipher Mode, Padding Method, Hashing Algorithm, Key Role Type, Digital Signature Algorithm, Cryptographic Algorithm, Random IV, IV Length, Tag Length, Fixed Field Length, Invocation Field Length, Counter Length, Initial Counter Value, Salt Length, Mask Generator, Mask Generator Hashing Algorithm, P Source, and Trailer Field. KMIP §10.1.2 requires fields of defined Structures to be encoded in the order in which they appear in their structure descriptions. The earlier draft list was incorrect and is superseded by this source-checked sequence.
- Existing generic request setters do not currently accept the new wrapper type. Typed convenience setters will be additive and consume it into the existing zeroizing TTLV `Structure` representation.

## Authentication secret lifecycle (ADR-0012)

- This is the separate approved client feature that introduces the first caller-requested secret-bearing request path. The direct maintainer approval is recorded in `approval-record.md` and supersedes the earlier 0008/0038 planning hold for this scope.
- `Authentication` owns an ordered `Structure`; `ClientBatch` will own it by value and is not cloned. The common Request Header builder moves the structure into the typed `RequestMessage`. `kmipkit-ttlv::Value` owns payloads in boxed `Secret<ValueRepr>` and recursively zeroizes nested Structure payloads on drop; `StructureView::try_clone` produces the same zeroizing owner. The private wire encoder owns initialized bytes in `Zeroizing<Vec<u8>>` through `Transport::exchange` and clears them on drop.
- With profile validation enabled, `execute_inner_with_context` creates one additional `RequestMessage` copy via `clone_request_message` for profile validation. That path clones through `StructureView::try_clone`, so nested values retain the zeroizing owner and are cleared when the profile copy drops. The production request tree is consumed into the execute-only wire path; `exchange_operation` drops the initialized encoded owner immediately after `Transport::exchange` returns, on either success or error. Existing `kmipkit-ttlv` tests cover the owner implementation; the candidate client test will add the ADR-0012 owner-through-transport assertions for Authentication.
- KMIP v2.1 §8.2 Table 395 places Authentication after Attestation Type and before Batch Error Continuation Option. Since the current client does not emit Attestation Type, the new field will be appended immediately after Attestation Capable Indicator and before Batch Error Continuation Option.
- The implementation test must exercise this candidate callsite, assert the encoded owner remains live through transport success and partial-write failure, then assert zeroization after transport returns. It must verify delivery state and one exchange/no retry on both paths. CI must pass that test against the candidate callsite before merge, enablement, or release, as ADR-0012 requires. This branch will remain a draft while hosted CI is unavailable.

## Authentication and security

- The protocol crate already has typed `Authentication`, `Credential`, and a read-only `RequestHeaderView::with_authentication` accessor. Credential types redact formatting and use owned secret wrappers.
- The client batch owns all common Request Header options and `build_request_header` emits those fields, but it currently has no Authentication field.
- OASIS places Authentication in the Request Header and one Request Message contains the ordered Batch Items. The one header is therefore the correct shared scope; no per-item API is needed.
- The approved plan authorizes completing the send path despite the earlier KMIPKIT-0008/0038 planning hold. Implement omission as the default and per-call explicit supply as the only selection policy.
- The caller-owned Authentication remains absent by default and is never stored in `Client` or configuration. Tests must prevent credentials and request bytes from entering logs, debug output, or errors.

## Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Unknown parameter fields | Preserve and expose through generic TTLV | KMIPKit must remain forward/vendor compatible. |
| Unknown Enumeration values | Preserve raw values | Future values must not be rejected for lack of a named constant. |
| Request Authentication scope | One optional field on `ClientBatch`/Request Message | Matches OASIS header shape and avoids global or per-item precedence. |
| Authentication absent | Omit it | OASIS makes Authentication optional; the client must not infer server policy. |
| Direct wrappers | Delegate through shared batch execution | Prevents divergence in preflight, transport, delivery evidence, and response decoding. |
| Language bindings | Defer parity to 0053 | The accepted Phase 5 plan assigns binding reconciliation to that specification. |

## Clarification result

The only apparent ambiguity—whether Authentication belongs to an operation item or the containing request—has a direct answer in KMIP v2.1 §§8.1–8.3: it is in the Request Header. The maintainer-approved plan resolves the earlier project planning hold. The Table 59 field order and the ADR-0012 first-secret-send test are incorporated before implementation. No unresolved clarification blocks implementation.

## Spec Kit consistency analysis

Reviewed `spec.md`, `plan.md`, `tasks.md`, traceability, contracts, the
requirements checklist, the normative catalog, and the pinned OASIS sources on
2026-11-05. The 13 functional requirements and five success criteria map to
the three story task groups and final verification tasks; the 56 `client_1_0`
operations map to direct-method decisions in `gap-matrix.md`. No critical or
high-severity issue remains. The analysis found and resolved the draft's wrong
Table 59 order and section reference, clarified that empty Authentication is
rejected during typed construction (rather than at Client execution), and
explicitly recorded three operation entries with no `feature_spec` assignment
in the catalog (Discover Versions, Poll, Cancel) as shared/core API rather than
inventing an owner. The draft quickstart constructor is intentionally pending
T012, which replaces it with compile-tested public examples before PR closure.
