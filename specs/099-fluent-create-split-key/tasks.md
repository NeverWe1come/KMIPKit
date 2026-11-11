# KMIPKIT-0099 Tasks

**Prerequisite**: KMIPKIT-0093 authorizes this operation-object slice. Work is
stacked on PR #179 head `45749b62e355ceea1ac6c936ec4a65b085050d60`; retain the
closed typed request and execution boundaries.

## Task 1: Setup and source mapping

- [x] Confirm the branch base is the published #179 head and the worktree is
  clean.
- [x] Map KMIP v2.1 §6.1.10 Tables 193–195 and KMIPKIT-0093-FR-004–011 to the
  existing request, response, validator, batch item, and execution route.
- [x] Record the preflight scan and TDD/check commands in the SDD ledger.

## Task 2: Rust request builder (Red)

- [x] Add focused tests for required Object Type, Split Key Method, Parts,
  and Threshold, including payload-free matching `validate()` and
  `build_request()` errors.
- [x] Add tests for optional Unique Identifier, exact Prime Field Size bytes,
  optional Protection Storage Masks, and an empty but present Attributes
  Structure.
- [x] Add tests for typed attributes, explicit unknown TTLV Items, valid
  Vendor Attribute construction, duplicate/final-position errors, and
  Message Extension tag rejection before transport.
- [x] Add tests for typed batch conversion and controlled Success, Failure,
  Pending, repeated response IDs in wire order, exact correlation, and one
  exchange without retry/Poll.
- [x] Run the focused test and record its expected Red failure before adding
  production code.

## Task 3: Rust request builder (Green)

- [x] Add `RequestBuilder::create_split_key()` and public
  `CreateSplitKeyBuilder` with explicit required/optional fields, default
  empty `AttributeSet`, appended typed/prebuilt attributes, explicit vendor
  constructor, and checked final positioning.
- [x] Add payload-free errors for the four missing required fields and stable
  display text; preserve existing fluent errors for invalid TTLV, invalid
  attributes, duplicate positions, bounds, and message-extension tags.
- [x] Implement `validate`, `build`, `build_request`, `to_batch_item`, and
  `execute` through `CreateSplitKeyRequest` and the current typed client.
- [x] Run focused tests, then `cargo test -p kmipkit-client --all-features`.

## Task 4: Public contract and docs (Refactor)

- [x] Add bilingual Create Split Key examples with explicit method, parts,
  threshold, and optional fields; show all returned IDs from the typed
  response, not inferred attributes.
- [x] Add every new public Rust declaration to the source side of
  `specification/api/public-api.json`, including stable requirement IDs and
  the pending cross-language parity state.
- [x] Regenerate all manifest outputs with `tools/api_manifest/generate.py`;
  never edit generated output by hand.
- [x] Complete `traceability.md` and `implementation-evidence.md` with exact
  source clauses, tests, Red/Green/Refactor SHAs, commands, and results.
- [ ] Commit docs and generated-artifact Refactor evidence separately.

## Task 5: Review, verify, and publish

- [x] Run formatting, Clippy, all client tests, manifest and traceability
  tests, generated-output check, Rust/C/Java/Python export audit, user-guide
  example checks, and changed-code coverage.
- [x] Obtain independent task review and resolve all findings with scoped
  re-review.
- [ ] Open a `[WIP]` draft PR based on PR #179, update the SDD ledger, and
  fix every red CI check on the exact head.
- [ ] Remove `[WIP]` and mark ready only after every required check is green;
  leave approval and merge to the human maintainer.
