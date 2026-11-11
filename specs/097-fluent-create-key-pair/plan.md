# KMIPKIT-0097 Implementation Plan

Implement a public fluent Create Key Pair builder over the already validated
`CreateKeyPairRequest`. Keep the change in `kmipkit-client`; do not create a
parallel request encoder or response model. Build each attribute group with the
existing `AttributeSet` and typed `Attribute<D>` APIs, and send through the
existing `Client::execute_one_typed_request_with_async_indicator` boundary.

## Verification

1. Add the failing Rust unit tests first and record Red in a signed-off commit.
2. Implement `CreateKeyPairBuilder` and `RequestBuilder::create_key_pair()`;
   run focused tests and record Green in a separate signed-off commit.
3. Add Rust docs, English and Spanish usage examples, traceability, and generated
   API manifest entries; regenerate checked-in outputs.
4. Run client tests, Clippy, formatting, traceability, manifest/export checks,
   and the PR CI. Keep `[WIP]` until all required checks are green.

## Review follow-up: complete KMIPKIT-0093 FR-004/FR-005

Before completing step 4, add the two contracts identified by the branch review:

1. Add failing tests for public local `.validate()` and prebuilt Common,
   Private Key, and Public Key `AttributeSet` inputs. Verify group placement,
   supplied order, append order, duplicate rejection, matching build errors,
   and that validation does not exchange with the transport.
2. Implement `validate(&self)` by reusing `build_request()` and add
   `common_attributes(AttributeSet)`, `private_key_attributes(AttributeSet)`,
   and `public_key_attributes(AttributeSet)`. These methods append in call
   order; they do not replace prior semantic or typed attributes. Keep final
   duplicate/conflict decisions in the existing typed request validator.
3. Update this operation spec, Rust API docs, English and Spanish examples,
   requirement traceability, and the public API manifest. Regenerate outputs;
   do not hand edit generated files.
4. Run the focused tests, client suite, Clippy, formatting, traceability,
   generator, binding export audit, and the exact-head hosted PR CI. Keep the
   draft WIP until every required check is green.

### Task 6: Complete local validation and prebuilt attribute groups

**Files:**
- Modify: `crates/kmipkit-client/tests/unit/fluent_create_key_pair_tests.rs`
- Modify: `crates/kmipkit-client/src/fluent.rs`
- Modify: `specs/097-fluent-create-key-pair/spec.md`
- Modify: English and Spanish fluent usage guide sections
- Modify: `specification/api/public-api.json` input records and generated outputs
- Modify: requirement traceability evidence, tests, and task ledger

**Interfaces:**
- Add `CreateKeyPairBuilder::validate(&self) -> Result<(), FluentBuildError>`.
- Add consuming append methods `common_attributes(AttributeSet) -> Self`,
  `private_key_attributes(AttributeSet) -> Self`, and
  `public_key_attributes(AttributeSet) -> Self`.
- All set methods append their items after existing semantic and typed
  attributes, in method-call order; each supplied set's internal order is
  preserved. Final typed request validation rejects duplicates/conflicts.

- [ ] Write focused tests proving validation is local and reports the same
  errors as construction, and proving group placement/order and duplicate
  rejection for each prebuilt group.
- [ ] Run the focused tests and record the expected missing-method compile
  failure before production changes.
- [ ] Implement the methods by reusing `build_request()` and existing
  `AttributeSet`/`CreateKeyPairRequest` validation.
- [ ] Run focused tests, client tests, Clippy, formatting, docs examples,
  traceability, manifest generation, and foreign export audit.
- [ ] Update bilingual examples, public API declarations, and generated files;
  preserve existing declarations and refresh affected hashes through the
  repository generator.
- [ ] Commit the Red, Green, and documentation/refactor evidence separately.

### Historical Task 6 fix round 1: reject Message Extension tags as object attributes

This earlier policy was superseded by KMIPKIT-0093. Current fluent builders
preserve allocated extension tags only when the caller supplies an explicit
typed `Item`; the raw tag is not converted to a Vendor Attribute.

- [ ] Add focused failing tests proving each prebuilt group rejects tags in
  `0x540000..=0x54FFFF` with `MessageExtensionTagNotObjectAttribute` and
  `execute()` returns `NotSent` without a transport call. Replace the prior
  order-test fixtures with valid catalogued Boolean attributes.
- [ ] Verify Red against the current implementation.
- [ ] Apply the existing symmetric-builder object-attribute guard consistently
  to Common, Private Key, and Public Key set methods; do not add a new send or
  validation path.
- [ ] Run focused tests, the client suite, Clippy, formatting, manifest,
  traceability, documentation, and binding checks. Commit Red and Green
  separately and append evidence to the task report.
- [ ] Obtain a scoped re-review of this finding and the fix diff before opening
  the WIP PR.
