# Incremental verification record

This file records evidence as KMIPKIT-0012 is implemented. It is not the final
feature-wide verification record; binding parity, coverage gates, interoperability,
fuzzing, and the remaining acceptance criteria are still open.

## 2026-10-07 — bounded schema order validation

### Change

Fixed repeated schema-width work in nested Structure validation. Empty Structures
now perform no ordering work. Sparse non-empty Structures use indexed directed
edge lookups; denser Structures make one pass over the compiled edge list. The
sorted edge vector remains the deterministic source for iteration and error
selection.

Red/Green/Refactor commits:

- Empty repeated Structure amplification: `aba425c` / `385b402` / `1f3ca70`.
- Sparse order lookup work: `55764d0` / `9698697` / `e232887`.
- Sparse full-edge fallback: `8eb2b9d` / `c641674` / `c886882`.
- QA regression assertion for zero work on empty Structures: `442937d`.

### Verification

- Mutation sensitivity check: temporarily forced the full-edge fallback for an
  empty nested Structure. The regression failed with 2,550,000 order-edge work
  units against the expected zero. The temporary mutation was reverted.
- `cargo test -p kmipkit-protocol --lib many_empty_nested_structures_do_not_repeat_schema_width_work -- --nocapture` — passed (1 test).
- `cargo test -p kmipkit-protocol -p kmipkit-client --all-features --quiet` — passed.
- `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.

## 2026-10-07 — vendor extension registry user story 1 test Red stage

### Change

Added focused boundary coverage for definition identity and limits, Extension
Information, aggregate registry accounting, schema work bounds, compile-fail
request boundaries, and outbound Message Extension ordering and secret-buffer
lifecycle. Most T017–T019 behavior was already present in the committed
foundation; those expanded acceptance tests pass against that baseline. The
new T020 outbound fake-transport tests intentionally expose the unimplemented
request-use wrapper and `ClientBatchItem::with_extension` API.

### Red evidence

- `cargo test -p kmipkit-protocol --test extension_definition --test extension_information` — passed (9 tests); this confirms the existing foundation satisfies the newly expanded T017/T019 cases.
- `cargo test -p kmipkit-client --test extension_registry` — passed (16 tests), including every configured text field independently and the pre-index-allocation resource checks.
- `cargo test -p kmipkit-protocol --lib t018_work_bound_tests` — passed (3 tests), including one order-edge pass for repeated ordered fields.
- `cargo test -p kmipkit-client --test extension_api_boundary` — passed (1 harness, 4 compile-fail fixtures); arbitrary Items, raw bodies, caller conversions, and executable callbacks remain outside the typed boundary.
- `cargo test -p kmipkit-client --lib extension_outbound_tests -- --nocapture` — failed to compile as intended: `ClientRequestMessageExtension`, `client_request_message_extension`, and `ClientBatchItem::with_extension` do not exist yet. This is the focused Red for T026.

The per-field text-limit test was corrected after independent QA found an
all-within-limit entry incorrectly expected to fail. Its targeted rerun passed.
A second QA pass found the exact-limit Description value had four bytes instead
of five; commit `09a9f6f` corrects it and the same targeted test passed again.
The outbound tests live under `tests/unit/` so Cargo does not also treat their
private crate-internal test module as an integration-test crate. No production
implementation changes are part of the Red commits.

### Independent reviews and limits

- Independent security review: PASS for the repeated-work finding. The compiled
  edge index reserves fallibly; resource limits bound definitions, rules, and
  edges; deterministic behavior uses the sorted vector rather than map iteration.
- Independent QA review: PASS after adding the zero-work assertion for empty
  nested Structures. The sparse-work metric counts map lookup calls; it does not
  expose internal hash-table probes. These calls use Rust's randomized
  `HashMap`, whose lookup complexity is expected constant time. The edge/rule
  counts are bounded by the configured and hard schema limits.
- These reviews cover only this remediation. They do not replace the qualified
  independent human security review required before the 1.0 release or the
  feature-wide QA/security gates in T061–T063.

## 2026-10-07 — client configuration owns an extension registry

### Change

Added an immutable `ClientConfiguration` that owns one registry snapshot and
exposes a read-only registry view. The configuration type and its constructor
and accessor are now part of the public API manifest. The production transport
constructor remains in KMIPKIT-0013 and can consume this configuration.

Red/Green evidence:

- Red: `ee76e84`; the new cross-client configuration test failed to compile
  because `ClientConfiguration` did not exist.
- Green: `89cc053` implements the configuration type and updates the public
  manifest and all generated outputs.
- Refactor: `366ec90` isolates metadata ordering and registry-index compilation
  in `RegistryIndexes::compile` without changing behavior.
- Independent QA review: PASS for T013 attachment/isolation and T016 behavior;
  the separate Red/Green/Refactor commits are present.

### Verification

- `cargo test -p kmipkit-client -p kmipkit-protocol --all-features --quiet` — passed.
- `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all --check` — passed after formatting the new test.
- `py -3 tools/api_manifest/generate.py --check` — passed; all six generated outputs match.
- `.venv\\Scripts\\python.exe -m unittest discover -s tools/api_manifest/tests -v` — 31 passed, 2 skipped because this Windows account cannot create symlinks.
- `git diff --check` — passed.

## 2026-10-07 — User Story 1 registered outbound extension path (Green)

### Change

The T022–T025 definition, schema, registry-limit, and sealed protocol-value
capabilities were already present in the committed foundation; the expanded
Red boundary tests now verify those limit and validation requirements. Added
the client-scoped `RegisteredExtensionValue` path: only validation by exact
identity against the immutable registry can produce a value accepted by the
outbound Message Extension wrapper. The wrapper requires explicit criticality,
preserves repeated caller order, and sends through the existing private
zeroizing request encoder. The public API manifest and six generated surfaces
include the registry validation entry point and registered-value handle.

The execute-boundary source audit now allows the generic TTLV `Structure`
input only for the exact `extension_registry.rs::validate_extension_value`
signature and exact registered-value result. New negative fixtures prove that
the same signature in another module, a renamed payload argument, or a generic
TTLV output remains rejected. The design plan, data model, and Rust protocol
contract distinguish standalone `ValidatedExtensionValue` from the
request-admissible `RegisteredExtensionValue`.

Red/Green evidence:

- Registry outbound Red: `63780a9` and exact Description limit correction
  `09a9f6f`.
- Public API boundary Red: `13a2bc7`; the focused audit failed because the
  exact registry validation seam was not yet allowlisted.
- Green: the exact-boundary fixture and production-source inventory pass; the
  rest of the T022–T027 changes are in this Green commit.

### Verification

- `cargo test -p kmipkit-client -p kmipkit-protocol --all-features --quiet` — passed (all client/protocol unit, integration, UI compile-fail, and doctest targets).
- `cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all --check` — passed.
- `cargo test -p kmipkit-client --lib execute_boundary_tests::registry_validation_generic_input_is_the_only_exact_exception -- --exact --nocapture` — passed, including the negative signature fixtures.
- `py -3 tools/api_manifest/generate.py --check` — passed; all six generated outputs match the manifest.
- `.venv\\Scripts\\python.exe -m unittest discover -s tools/api_manifest/tests -v` — 31 passed, 2 skipped because Windows denied symlink creation.
- `git diff --check` — passed.

This is feature-level incremental evidence, not the T061 release-readiness gate;
cross-language parity, coverage, fuzzing, sanitizer jobs, and interoperability
remain open.

## 2026-10-07 — registry provenance seal (partial cross-spec correction)

Independent QA found that a registry-validated outbound value did not retain
which immutable client registry produced it. Added a private per-registry
identity token, cloned into each `RegisteredExtensionValue` and retained by
the request-use wrapper. The regression test proves that configuration A
recognizes a value validated by A and configuration B does not.

This closes the provenance-loss portion of the finding. The current KMIPKIT-0007
`Client::execute` has no client-configuration field, so it cannot yet reject a
cross-configuration request before encoding or exchange. KMIPKIT-0013 owns the
production client constructor; its integration must retain the registry
configuration and perform this token comparison at the start of execution,
returning sanitized `InvalidInput` with `NotSent` before message construction.
That cross-spec execution check remains open and must be covered by a public
client test before the finding is closed.

Independent review also confirmed that the current accepted KMIPKIT-0013
specification does not yet require that composition or rejection test. The
KMIPKIT-0013 contract and task plan must be amended before implementation can
claim to close this cross-client isolation finding.

## 2026-10-07 — User Story 2 test-only Red stage

Added recognition contract tests for exact discriminator lookup, missing,
repeated, and wrong-type nested paths, wide structures and bounded index work,
schema rejection after a unique match, duplicate registration keys, ambiguous
matches, and absence of partial typed output. Added fake-transport assertions
that keep unknown-critical rejection and unknown-noncritical preservation in
KMIPKIT-0007, plus generic TTLV preservation/property and redaction regressions.

Red/baseline evidence:

- `cargo test -p kmipkit-client --test extension_recognition` — expected Red
  (exit 101): `ExtensionRecognition` and the `inspect`, `is_recognized`,
  `validated_value`, and `generic_value` APIs are not implemented.
- `cargo test -p kmipkit-client --lib extension_execution_tests -- --nocapture`
  — expected Red (exit 101) for the same missing recognition APIs; response
  integration assertions did not run yet.
- `cargo test -p kmipkit-protocol --test extension_preservation` — baseline
  passed (6 tests); these generic preservation and codec-boundary behaviors
  already exist below the new client recognition layer.
- `cargo test -p kmipkit-protocol --test extension_redaction` — baseline
  passed (2 tests); current schema values and diagnostics already redact the
  sentinel data.
- Runtime payload-index accounting is asserted at the client layer because
  `kmipkit-protocol` deliberately has no dependency on `kmipkit-client`.

Manifest ownership correction Red evidence:

- `.venv\Scripts\python.exe -m unittest tools.api_manifest.tests.test_generate.ManifestSchemaTests.test_schema_accepts_borrowed_value_inside_optional_rust_result -v`
  — expected Red: the Rust type schema rejects
  `Option<&ValidatedExtensionValue>`, so the manifest cannot yet describe the
  borrowed inspection accessor without pretending to return an owned payload.

Exact inbound inspection boundary Red evidence:

- `cargo test -p kmipkit-client --lib registry_inspection_and_generic_accessors_are_the_only_inbound_ttlv_surface -- --nocapture`
  — expected Red (exit 101): the source inventory rejects the exact
  `extension_registry::inspect` generic-Structure input because it has no
  narrow signature allowlist yet. Negative fixtures also cover a different
  module, renamed payload argument, wrong return type, and non-borrowed or
  arbitrary generic-value accessor signatures.

Registry-depth scope Red evidence:

- `cargo test -p kmipkit-client --test extension_recognition registry_depth_limit_does_not_restrict_preserved_unknown_payload_subtrees -- --nocapture`
  — expected Red (exit 101): a payload within `CodecLimits` but deeper than
  the registry's schema/path depth is rejected as `ResourceLimit`. The
  registry depth limit must bound registered schemas and discriminator paths;
  payload nesting remains governed by `CodecLimits`.

Root-inclusive payload-cap Red evidence:

- `cargo test -p kmipkit-client --test extension_recognition raised_codec_limit_cannot_raise_the_hard_payload_item_cap -- --nocapture`
  — expected Red (exit 101): the current index accepts 100,000 children in
  addition to the root when `CodecLimits` is raised. The hard recognition cap
  is 100,000 total TTLV items, including the root Structure.

Red/Green evidence:

- Red: `de93ace`; the provenance regression test failed to compile because the
  request wrapper had no registry-ownership check.
- Green: recorded in the following implementation commit; the private token is
  checked by the wrapper against the configuration's immutable registry.

### Verification

- `cargo fmt --all --check` — passed.
- `cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features -- -D warnings` — passed.
- `cargo test -p kmipkit-client -p kmipkit-protocol --all-features --quiet` — passed, including the new provenance regression test.
- `git diff --check` — passed.

## 2026-10-07 — registry total accounting refactor

Extracted checked, bounded accumulation of registry text bytes, schema nodes,
discriminator bytes, and constraint members into one helper. Each category
still uses checked `usize`-to-`u64` conversion, checked addition, and its own
configured maximum; error categories and fail-fast order are unchanged.

- Red/Green evidence: the full KMIPKIT-0012 US1 boundary suite passed before
  this behavior-preserving refactor; no acceptance behavior changed.
- `cargo test -p kmipkit-client -p kmipkit-protocol --all-features --quiet` — passed after refactor (all unit, integration, UI compile-fail, and doctest targets).
- `cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all --check` — passed.
- `git diff --check` — passed.
