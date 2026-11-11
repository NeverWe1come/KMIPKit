# KMIPKIT-0100 Tasks

**Prerequisite**: complete KMIPKIT-0093 scope was authorized by the maintainer
on 2026-11-11. This branch is stacked on PR #181 at its published head and
must stay based on that dependency while review is pending.

## Specification and Red

- [x] Map Register model, validator, execution route, OASIS references, and
  parent fluent contract.
- [x] Add focused tests for missing object, typed object, explicit generic
  object, attributes and order, masks/context, invalid object, batch
  conversion, one-exchange result, and secret redaction.
- [x] Run `cargo test -p kmipkit-client --all-features fluent_register_tests
  -- --nocapture`; expected Red exit 101 reports missing `register`,
  `RegisterBuilder`, and the missing/invalid Register builder errors.
- [ ] Commit the specification and failing tests with DCO before production
  changes.

## Green implementation

- [ ] Add `RequestBuilder::register()` and public `RegisterBuilder`.
- [ ] Add semantic object and explicit typed-item setters, attributes,
  Protection Storage Masks, and Opaque Object context.
- [ ] Add `validate`, `build`, `build_request`, `to_batch_item`, and one-shot
  `execute` through the existing typed path.
- [ ] Run focused tests, full client tests, and Clippy; commit Green separately.

## Refactor and evidence

- [ ] Add English and Spanish examples with semantic object construction.
- [ ] Add requirement traceability and implementation evidence.
- [ ] Add exact public Rust declarations to the API manifest source and
  regenerate outputs; mark language parity pending.
- [ ] Run formatting, client tests, guide examples, manifest/export checks,
  coverage, and diff checks; commit Refactor separately.

## PR and follow-on work

- [ ] Open `[WIP]` draft PR #182, based on PR #181; attach it to this task.
- [ ] Fix all CI failures and wait for required checks on the exact final SHA.
- [ ] Remove WIP and mark ready only when all required CI is green. Do not
  approve or merge.
- [ ] Continue the KMIPKIT-0093 operation/binding/end-to-end slices without
  waiting for human merge.
