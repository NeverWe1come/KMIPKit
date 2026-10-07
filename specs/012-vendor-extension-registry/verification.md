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
