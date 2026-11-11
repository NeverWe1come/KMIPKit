# KMIPKIT-0100 Tasks

**Prerequisite**: complete KMIPKIT-0093 scope was authorized by the maintainer
on 2026-11-11. This branch is stacked on PR #181 head
`d1eaf235a6aa39fa5db4a53e0562f1bdddf5afe6` while its review is pending.

## Specification and Red

- [x] Map Register model, validator, execution route, OASIS references, and
  parent fluent contract.
- [x] Add focused tests for missing object, typed object, explicit generic
  object, attributes and order, masks/context, invalid object, batch
  conversion, one-exchange result, and secret redaction.
- [x] Run `cargo test -p kmipkit-client --all-features fluent_register_tests
  -- --nocapture`; expected Red exit 101 reports missing `register`,
  `RegisterBuilder`, and the missing/invalid Register builder errors.
- [x] Commit the specification and failing tests with DCO before production
  changes.

## Green implementation

- [x] Add `RequestBuilder::register()` and public `RegisterBuilder`.
- [x] Add semantic object and explicit typed-item setters, attributes,
  Protection Storage Masks, and Opaque Object context.
- [x] Add `validate`, `build`, `build_request`, `to_batch_item`, and one-shot
  `execute` through the existing typed path.
- [x] Run focused tests, the full client suite, and Clippy; commit Green
  separately.

## Refactor and evidence

- [x] Add English and Spanish examples with semantic object construction.
- [x] Add requirement traceability and implementation evidence.
- [x] Add exact public Rust declarations to the API manifest source and
  regenerate outputs; mark language parity pending.
- [x] Run formatting, client tests, guide examples, manifest/export checks,
  coverage, and diff checks; commit Refactor separately. The exact Register
  exports match; the global Rust audit remains incomplete outside this spec
  (see `implementation-evidence.md`).

## PR and follow-on work

- [ ] Keep `[WIP]` draft PR #183 based on PR #181; attach it to this task.
- [ ] Fix all CI failures and wait for required checks on the exact final SHA.
- [ ] Remove WIP and mark ready only when all required CI is green. Do not
  approve or merge.
- [ ] Continue the KMIPKIT-0093 operation/binding/end-to-end slices without
  waiting for human merge.
