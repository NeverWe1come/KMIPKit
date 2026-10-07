# Incremental verification record

This file records evidence as KMIPKIT-0012 is implemented. Cross-language
registry parity, examples, and a bounded fuzz smoke run have local evidence.
The final aggregate coverage gate, current-commit CI/platform matrix, feature
acceptance audit, and independent QA/security reviews remain open.

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

## Implementation authorization record

The feature is approved for implementation under the maintainer's standing
direct authorization to execute the full KMIPKit plan without further approval
requests. This records authorization of the defined feature scope and does not
claim a line-by-line human review of this specification revision. The draft PR
and human-only approval/merge requirements remain in force.

## 2026-10-07 — CI fixture-generator rejection tests

- Red: `aad390b` added a workflow contract requiring CI to run the extension
  fixture generator's negative tests. The focused contract failed because the
  `script-contracts` job only checked generated outputs and did not run those
  tests.
- Green: `0a3dffa` added the fixture test module to the cross-platform
  `script-contracts` job. The workflow suite passed (30 tests), the fixture
  module passed (13 tests), and `python -B
  tools/extension_fixtures/generate.py --check` passed.
- Refactor: clarified the CI coverage in `docs/development/testing.md`; no
  production behavior or generated output changed.

## 2026-10-07 — reproducible fixture-test environment

- Red: `16fca8c` added a workflow contract requiring a pinned Python 3.12
  environment and installation of `tools/api_manifest/requirements-test.txt`.
  It failed because the `script-contracts` job had no such setup or install.
- Green: `4b5e5c0` sets up Python 3.12 with the pinned uv action and installs
  the pinned `jsonschema` requirement before running the fixture tests. The
  workflow contract suite passed (31 tests); the fixture test module passed
  (13 tests).
- Refactor: documented the reproducible CI test environment in
  `docs/development/testing.md`; generated files and production code are
  unchanged.

## 2026-10-07 — select the pinned Python interpreter in CI

- Red: `52ac62c` added a workflow contract rejecting PATH-based Python
  selection. The test failed because the job still preferred the runner's
  `python3` executable.
- CI confirmation: run `37652625959` showed Windows installed pinned
  `jsonschema` into the uv environment, then selected
  `C:\hostedtoolcache\windows\Python\3.12.10\x64\python.exe`; the fixture
  test failed with `ModuleNotFoundError: jsonschema`.
- Green: `a01a430` resolves the platform-specific executable inside
  `$env:VIRTUAL_ENV` and fails early if it is missing. The workflow contract
  suite passed (32 tests). The exact Python-contract command sequence passed
  locally: script tests 199 passed with 26 expected cargo-deny skips, the
  normative catalog suite 170 passed with 7 platform skips, fixture tests
  13/13 passed, and both generator `--check` commands passed.
- Refactor: the testing guide now explains explicit interpreter selection;
  no production code or generated output changed.

## 2026-10-07 — response integration Refactor

Synchronous and asynchronous typed response paths now share
`preserve_response_extensions`, which applies the same recognition lookup,
KMIPKIT-0007 criticality policy, and generic TTLV preservation in both cases.
This removes duplicate response-extension handling without changing behavior.

Verification:

- `cargo test -p kmipkit-client --test extension_recognition` — passed (12
  tests, including exact comparison boundaries and schema-preservation cases).
- `cargo test -p kmipkit-protocol --test extension_preservation` — passed (7
  tests, including property-based round-trip/preservation cases).
- `cargo test -p kmipkit-client -p kmipkit-protocol --all-features` — passed;
  all client/protocol unit, integration, property, UI compile-fail, and doctest
  targets passed (192 client unit tests).
- `cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all --check` — passed.
- `python tools/api_manifest/generate.py --check` — passed; six generated files
  are current.
- `git diff --check` — passed.

## 2026-10-07 — request extension client-ownership Green

The client now checks the sealed registry identity of every outbound typed
Message Extension against its own immutable `ClientConfiguration` before
building or sending the request. A value created by another registry returns a
redacted validation error with delivery state `NotSent`; valid same-registry
values remain eligible for normal request encoding.

Red/Green evidence:

- Red: `d2a0fc6`; the fake-transport regression showed that a request extension
  validated by registry A could be submitted through a client using registry B.
- Green: `cargo test -p kmipkit-client --lib a_request_extension_from_another_client_registry_is_rejected_before_transport -- --nocapture` — passed; the request was rejected before transport and reported `NotSent`.

This closes the ownership boundary described by `KMIPKIT-0012-FR-005` and the
Registered Extension Value definition. The protocol-level schema validation
alone cannot confer a client registry seal.

## 2026-10-07 — cross-configuration request ownership Red

- `cargo test -p kmipkit-client --lib a_request_extension_from_another_client_registry_is_rejected_before_transport -- --nocapture`
  — expected Red: an extension value sealed by one registry was accepted and
  sent by a client configured with a different registry. The required result
  is a sanitized validation error with `NotSent` evidence and zero exchanges.

## 2026-10-07 — adversarial lookup comparison boundary

Added a sorted payload with a repeated discriminator at the start followed by
99,997 greater tags. This exercises 17 lower-bound comparisons, the equality
check, and 17 upper-bound comparisons. The earlier all-equal fixture remains a
separate 34-comparison boundary case.

- Red mutation check: temporarily lowered `MAX_TAG_COMPARISONS_PER_STEP` from
  35 to 34. `cargo test -p kmipkit-client --test extension_recognition
  worst_case_duplicate_tag_search_uses_exactly_thirty_five_comparisons --
  --nocapture` failed at the exact 35-comparison lookup with `ResourceLimit`.
  The temporary implementation mutation was restored.
- The new test passes with the approved bound of 35 and requires a registry
  lookup budget of 34 to reject the same payload.

## 2026-10-07 — registered response recognition Red

- `cargo test -p kmipkit-client --lib unit::extension_execution_tests::a_registered_critical_response_extension_is_accepted_by_typed_execution -- --nocapture`
  — expected Red (exit 101): the test cannot compile because `Client` has no
  configuration-aware construction path. Without retaining the immutable
  client registry, typed response mapping cannot distinguish a recognized
  critical Message Extension from an unknown critical extension while keeping
  KMIPKIT-0007 rejection behavior for the latter.

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

## 2026-10-07 — registry provenance seal and pre-transport check

Independent QA found that a registry-validated outbound value did not retain
which immutable client registry produced it. Added a private per-registry
identity token, cloned into each `RegisteredExtensionValue` and retained by
the request-use wrapper. The regression test proves that configuration A
recognizes a value validated by A and configuration B does not.

The execute path now calls `validate_request_extension_ownership` before
building the request message or exchanging bytes. The regression test
`a_request_extension_from_another_client_registry_is_rejected_before_transport`
asserts sanitized validation failure with `NotSent` delivery state and zero
transport exchanges. This closes the cross-configuration send gap.

An earlier review note attributed the missing check to the later KMIPKIT-0013
transport-construction work. That note is superseded: the current implementation
retains the configuration in `Client` and rejects a foreign registry value
before encoding or transport. No KMIPKIT-0013 specification amendment is
needed for this behavior.

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

Recognition-bridge Red evidence:

- `cargo test -p kmipkit-protocol --test extension_preservation schema_only_validation_does_not_claim_discriminator_recognition -- --nocapture`
  — expected Red (exit 101): the public protocol surface still exposes only
  `validate_for_recognition`/`RecognitionValidation::Validated`, whose names
  can be mistaken for a complete discriminator recognition result. The new
  test requires an explicitly schema-only outcome while confirming that full
  `validate` continues to reject a discriminator mismatch.

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

## 2026-10-07 — recognized response extension Green

`Client` now retains its immutable `ClientConfiguration`. Synchronous and
asynchronous typed response mapping inspects each Message Extension using that
client's registry and the same per-call `CodecLimits` used for response
decoding. Recognized critical extensions continue; unrecognized critical
extensions still pass through KMIPKIT-0007's rejection helper. The response
model retains its original generic Message Extension for caller inspection.

Payload ownership review: `ExtensionRecognition` owns either the original
generic `Structure` or a schema-validated value that owns the same generic
subtree. The response mapping's bounded copies use the existing TTLV `Value`
owner, whose drop recursively zeroizes payloads; decode and response copies
therefore follow the established zeroization behavior. Recognition and
response-extension Debug implementations redact payload/vendor content, with
sentinel assertions in the fake-transport tests. Protocol redaction tests cover
schema errors and validated values.

Red/Green evidence:

- Red: `4961de7`; the registered-critical fake-transport test could not compile
  because the client did not retain a configuration or registry.
- Green: the response mapping stores the configuration and recognizes both
  synchronous and asynchronous registered critical extensions. Unknown
  critical and non-critical fake-transport cases continue to verify the
  KMIPKIT-0007 behavior.
- The adversarial 35-comparison case passes; lowering the internal hard bound
  to 34 makes it fail at that exact comparison. The all-equal repeated-tag
  case separately proves its 34-comparison boundary.

Verification:

- `cargo test -p kmipkit-client -p kmipkit-protocol --all-features` — passed,
  including 192 client unit tests, response-recognition/fake-transport tests,
  protocol preservation/redaction tests, integration tests, and doctests.
- `cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all --check` — passed.
- `python tools/api_manifest/generate.py --check` — passed; six generated files
  are current.
- `git diff --check` — passed.

## 2026-10-07 — client-owned request extensions and async response access

Outbound typed request extensions are now checked against the registry owned
by the executing client before request construction. Cross-registry values
return a redacted `NotSent` validation error. Existing request zeroization and
ordering tests now build their extension values from the same client
configuration they exercise.

Async outcomes now expose accepted generic Message Extensions through
`ClientOperationOutcome::extensions()`, matching synchronous outcome access.
Recognized critical values remain available for caller inspection, while
unrecognized critical values still fail response validation. Documentation for
both outcome accessors now covers recognized critical and non-critical
extensions.

Red/Green evidence:

- Request ownership Red: `d2a0fc6`; the client sent an extension sealed by a
  different configuration instead of rejecting it before transport.
- Async preservation Red: `9885e26`; the focused test failed to compile because
  `ClientOperationOutcome` did not expose an `extensions()` accessor.
- Green: request ownership is validated before encoding; async response mapping
  validates criticality and retains the original generic extension Structure.
  The focused request-ownership and async-recognition tests both pass.
- The first full suite exposed three legacy secret-extension fixtures that
  created unrelated registries; those fixtures now use their client's owned
  registry, preserving the required ownership boundary.

Verification:

- `cargo test -p kmipkit-client -p kmipkit-protocol --all-features` — passed;
  all client/protocol unit, integration, property, UI compile-fail, and doctest
  targets passed (192 client unit tests).
- `cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt --all --check` — passed.
- `python tools/api_manifest/generate.py --check` — passed; six generated files
  are current.
- `git diff --check` — passed.


## 2026-10-07 — User Story 3 test-only Red stage

Added manifest parity checks, expanded the shared adapter-neutral fixture corpus
with all registry limit boundaries and bounded algorithm-work cases, and added
C, Java 17, and Python 3.12 consumer tests for registry ownership, limits,
inspection/preservation, metadata, lifecycle, and redacted errors. The preserved
vendor-range child uses Tag 0x540001 under KMIP 2.1 §11.56 and ADR-0010.

Red and baseline evidence:

- python <temp-venv>/Scripts/python.exe -m unittest discover -s tools/api_manifest/tests -v
  — passed 38 tests; two symlink tests were skipped because Windows denied
  symlink creation without the required privilege.
- python tools/api_manifest/generate.py --check — passed; all six generated
  outputs remain current.
- cc -std=c11 -Wall -Wextra -Werror -Ibindings/c/include -fsyntax-only
  bindings/c/tests/extension_registry.c (WSL Ubuntu) — expected Red: the
  generated declarations use a single opaque handle pointer for the
  ExtensionChildRule and ExtensionDefinition collections, while a C
  consumer needs arrays of opaque handles (handle **). An opaque handle
  cannot represent an array through pointer arithmetic. The manifest and
  generator contract must be corrected before this consumer can compile.
- javac --release 17 -Xmaxerrs 20 ... ExtensionRegistryTest.java — expected
  Red: the Java facade/classes and JUnit dependency are not yet present; javac
  reports the absent org.junit.jupiter.api and org.kmipkit.extensions
  packages.
- python312 -m unittest discover -s bindings/python/tests -p
  test_extension_registry.py -v — expected Red at import:
  ModuleNotFoundError: No module named kmipkit; the CFFI package is not
  implemented yet. Python 3.12 AST parsing passed.
- git diff --check — passed.

No production binding implementation or generated output was changed in this
Red stage. C, Java, and Python runtime assertions await their corresponding
adapter implementations.

## 2026-10-07 — Java 17 facade T050 partial verification

Added the Maven/JUnit 5 build and Java facade classes for the approved registry
and generic TTLV API surface. JNI lifecycle and C ABI bridge implementation
remain in progress under T050. The Java Red test received two compile-only corrections: its missing import for the
manifest's `org.kmipkit.extensions.TtlvPath`, and a final loop-index capture for
the limit-boundary assertion. Neither changes test coverage or intent.

Verification:

- `mvn -q -f bindings/java/pom.xml compile` — passed (exit 0), compiling
  production sources with the Maven compiler configured for Java 17.
- `mvn -f bindings/java/pom.xml -DskipTests test-compile` — passed; Maven
  reports `BUILD SUCCESS` and compiles both Java test sources with `--release
  17`.
- `mvn -f bindings/java/pom.xml '-Dtest=ExtensionRegistryTest#everyRegistryLimitExposesDefaultsAndAcceptsItsLowerAndHardBoundaries' test`
  — passed (1 test, 0 failures/errors); this pure-Java boundary test does not
  require JNI.
- `mvn -f bindings/java/pom.xml -Dtest=NativeHandleTransferTest test` — passed
  (3 tests, 0 failures/errors). Coverage verifies transfer disarms the Cleaner,
  rejects reuse, permits close after transfer without native release, and leaves
  all owners live when an atomic multi-handle transfer sees a closed input.
- Facade transfers now match every `consumed` handle parameter in the manifest.
  The configuration wrapper snapshots definition/limit metadata before it
  transfers its registry; request-extension criticality and batch attachment
  order remain in Java-owned metadata after their native handles transfer.
- `python -m unittest discover -s tools/api_manifest/tests -p test_parity.py -v`
  — passed (5 tests), including legal Java identifier validation.
- `python tools/api_manifest/generate.py --check` — passed earlier with all six
  outputs current. A later rerun in this shared worktree now fails because
  `crates/kmipkit-ffi/src/extension_registry_generated.rs` differs from the
  manifest renderer (expected 27,712 bytes, current 30,646 bytes); that file is
  concurrently modified outside this Java change and was not edited here.
- `mvn -f bindings/java/pom.xml test` — expected runtime block (exit 1):
  `Tests run: 13, Failures: 1, Errors: 8, Skipped: 0`. The first native call
  fails with `java.lang.UnsatisfiedLinkError: no kmipkit_jni in
  java.library.path`; subsequent tests report `NoClassDefFoundError` because
  `NativeExtensionRegistry` could not initialize. The single assertion failure
  expected `ResourceLimitException` but received that same initialization
  error. Build configuration and test sources compile; registry behavior is
  not Green until a JNI library is available.

T050 remains partial. Commit `c8b5052` corrected the manifest mapping to the
Java-legal `TtlvValueView.booleanValue()` name and its JNI symbol. Repeated
outbound attachment order and encoded criticality remain unverified: the
approved slice has no public `ClientBatchItem` constructor or request encoder.
Typed recognition metadata is also blocked because the approved C ABI has no
identity-field getter; do not infer an identity by scanning payload
discriminators. The JNI bridge and its runtime lifecycle remain
unimplemented/in progress under T050 pending an approved identity-getter
contract.

## 2026-10-07 — Java batch inspection API Red stage

The manifest now includes identity field getters and the Discover Versions
`ClientBatchItem` constructor plus extension count, identity-at, and
criticality-at inspection. Added an executable Java test that appends two
extensions, checks caller order and both explicit criticality values, checks
copied native identity fields, and rejects out-of-range inspection.

Red evidence:

- `mvn -f bindings/java/pom.xml -DskipTests test-compile` — expected failure:
  8 javac errors because `ClientBatchItem.discoverVersions`,
  `extensionCount`, `extensionIdentityAt`, and
  `extensionCriticalityIndicatorAt` are not implemented yet.
- Dependency review: `docs/security/dependency-policy.md` governs Cargo and
  defines no Java/Maven dependency allowlist. No Java runtime dependency is
  planned; the bridge uses JDK JNI headers and the installed Visual Studio
  compiler.

## 2026-10-07 — C ABI and real consumer Green stage

Implemented the 122 functions declared by the generated C header, including
the seven identity and `ClientBatchItem` inspection functions added to the
manifest. The C identity getters return owned TTLV TextString handles; batch
inspection copies each stored identity and preserves extension order and
explicit criticality. The CMake consumer links to the built `kmipkit_ffi`
shared library.

Red/Green evidence:

- Red: `d4528d9`; `cargo test -p kmipkit-client --lib repeated_message_extensions_keep_explicit_criticality_and_caller_order` failed to compile because the three batch inspection methods were absent. In WSL Ubuntu, building the C consumer failed at link with unresolved references to the seven newly manifested symbols.
- Green: `crates/kmipkit-client/src/execute.rs` provides Discover Versions construction and ordered inspection. `crates/kmipkit-ffi/src/extension_registry.rs` implements the seven exports. The C consumer verifies all three identity fields, two distinct extension identities, their order, and criticality values `true` then `false`.

Verification:

- `cargo test -p kmipkit-ffi -p kmipkit-client --all-features` — passed; 192 client unit tests and all client integration, UI compile-fail, and doc tests passed. The FFI Rust unit-test target contains no tests; ABI behavior is exercised through the compiled C consumer below.
- `cargo clippy -p kmipkit-ffi -p kmipkit-client --all-targets --all-features -- -D warnings` — passed.
- `cargo fmt -p kmipkit-client -p kmipkit-ffi -- --check` — passed.
- `python tools/api_manifest/generate.py --check` — passed; six generated outputs are current.
- Export audit: 122 C header function declarations and 122 explicit Rust exports; no missing or extra names.
- Unsafe/pointer audit: all 11 unsafe blocks in `extension_registry.rs` have `SAFETY` comments. Variable byte spans and handle arrays pass through bounded helpers before reads; no NUL scans are used. Zero-count arrays accept `NULL` and produce empty collections.
- In WSL Ubuntu, `cargo build -p kmipkit-ffi` — passed and produced the Linux shared library.
- In WSL Ubuntu, `gcc -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include bindings/c/tests/extension_registry.c -L target/debug -lkmipkit_ffi -Wl,-rpath,/mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0012-vendor-extension-registry/KMIPKit/target/debug -o target/debug/kmipkit-c-consumer` — passed; linked against the built cdylib.
- In WSL Ubuntu, `target/debug/kmipkit-c-consumer tests/fixtures/extensions/cases.json` — passed all eight C consumer groups.
- In WSL Ubuntu, CMake 4.2.3 configure, build, and test commands passed:
  - `wsl.exe -e bash -lc "export LD_LIBRARY_PATH=/mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/lib/x86_64-linux-gnu && export CMAKE_ROOT=/mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/share/cmake-4.2 && /mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/bin/cmake -S /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0012-vendor-extension-registry/KMIPKit/bindings/c -B /mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake-build"` — configured successfully.
  - `wsl.exe -e bash -lc "export LD_LIBRARY_PATH=/mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/lib/x86_64-linux-gnu && export CMAKE_ROOT=/mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/share/cmake-4.2 && /mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/bin/cmake --build /mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake-build"` — built the real C consumer.
  - `wsl.exe -e bash -lc "export LD_LIBRARY_PATH=/mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/lib/x86_64-linux-gnu && export CMAKE_ROOT=/mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/share/cmake-4.2 && /mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake/root/usr/bin/ctest --test-dir /mnt/c/Users/ramp1953/.codex/tmp/kmipkit-cmake-build --output-on-failure"` — passed (1/1).
- `git diff --check` — passed.

## 2026-10-07 — C identity and batch inspection Red stage

Added C consumer assertions for all three identity fields and for the order and
explicit criticality of two attached extensions. A Rust client unit test also
asserts `ClientBatchItem` count, identity-at, criticality-at, and out-of-range
behavior.

Red evidence:

- `cargo test -p kmipkit-client --lib repeated_message_extensions_keep_explicit_criticality_and_caller_order` — expected compile failure because `ClientBatchItem` does not yet expose `extension_count`, `extension_identity_at`, or `extension_criticality_indicator_at`.
- In WSL Ubuntu, `cargo build -p kmipkit-ffi` — passed for the baseline ABI.
- In WSL Ubuntu, `gcc -std=c11 -Wall -Wextra -Werror -pedantic -I bindings/c/include bindings/c/tests/extension_registry.c -L target/debug -lkmipkit_ffi -Wl,-rpath,$PWD/target/debug -o target/debug/kmipkit-c-consumer` — expected link failure because all seven newly manifested identity/batch symbols are declared by the generated header but not exported yet; C compilation itself succeeded.
- `git diff --check` — passed.

## 2026-10-07 — Python 3.12 CFFI facade T051 Green and Refactor

Implemented the typed `kmipkit` facade over the manifest-backed C ABI with
Maturin/CFFI packaging, limits/configuration/registry APIs, generic TTLV
builders and views, extension inspection and preservation, explicit outbound
criticality, native identity getters, and Discover Versions batch extension
inspection. `NativeHandle` owns all CFFI pointers privately, closes
idempotently, supports context managers, and atomically transfers consumed
handles under an ownership lock. Errors map to stable redacted categories.
Python-owned strings and byte values are copies that the Python runtime cannot
deterministically zeroize; this limitation is documented in the package and
facade module docstrings.

Red evidence:

- Existing manifest tests committed as `c73a89b` failed before the facade was
  present. Ownership transfer tests in `0860d8d` established consumed-handle
  close behavior.
- `python -m unittest discover -s tests -p test_handle_ownership.py -v` — the
  new concurrent close/transfer test failed before the ownership lock with two
  releases for one handle; after the fix it passes with the other ownership
  cases.
- The Python batch parity Red test in `2e619b1` failed with missing
  `client_batch_item_discover_versions`; native identity getter assertions in
  `82a5e9b` likewise required the newly manifested getters.
- Shared preservation fixtures were corrected in `255ab97` to use allocated
  KMIP extension-range tags (`0x540010`–`0x540015`, `0x540021`–`0x540022`), as
  required by the checked-tag model. Its Python normalizer correction retains
  the fixture's nested `children` array shape.

Green and Refactor commits:

- `5deaa09` — typed CFFI facade, native handle adapter, stable errors, TTLV
  views/builders, and Maturin packaging.
- `d6aaf0a` — consolidated copied identity hydration and deterministic view
  closure; UTF-8 conversion errors suppress source exception context.

Verification, run from `bindings/python` with Python 3.12:

- `maturin develop` — passed; generated CFFI bindings, built the native-backed
  wheel, and installed it into the Python 3.12 environment.
- `python -m unittest discover -s tests -v` — passed, 13 tests, 0 failures or
  errors. This includes immutable configuration ownership, every registry
  limit boundary, redacted diagnostics, preservation, identity getters,
  Discover Versions repeated attachment order and criticality, out-of-range
  inspection errors, close-after-consume idempotence, conversion rollback, and
  concurrent close/transfer.
- `ruff check src/kmipkit --exclude _generated --exclude _ffi` — passed.
- `ruff format --check src/kmipkit --exclude _generated --exclude _ffi` —
  passed; 5 source files already formatted.
- `python -m compileall -q src/kmipkit tests` — passed.
- Manifest symbol audit — all 97 declared C symbols were available through
  the freshly built CFFI `lib`; no missing symbols.
- `pyproject.toml` parse via Python 3.12 `tomllib` — passed.

The batch route exercised here constructs Discover Versions items, the only
public `ClientBatchItem` constructor in the approved manifest. Generic batch
item construction and wire encoding remain outside this T051 facade slice.

## 2026-10-07 — Java facade, generated parity, and OASIS vectors Green

### Java 17 JNI facade (T050)

- Red: `7287362`; Green: `5ba20d7`.
- `mvn -f bindings/java/pom.xml test` — passed on Windows; JNI DLL built and loaded, 14 tests passed (0 failures/errors/skips). A rerun after the current Java changes also passed 14/14.
- Native symbol audit: all 91 Java native declarations have JNI implementations.
- `python tools/api_manifest/generate.py --check` — passed; all six generated outputs match the manifest.
- The 42-test manifest suite below exercises Java API legality and mappings. Linux and macOS JNI builds remain for the platform matrix in T061.

### Cross-adapter Green evidence (T053)

- C Green: `f08dcf4`; WSL Ubuntu real C consumer passed all eight groups, and CMake/CTest passed 1/1.
- Java Green: `5ba20d7`; JNI-enabled Maven suite passed 14/14.
- Python Green: `5deaa09`; Python 3.12 CFFI suite passed 13/13 after a fresh Maturin build; Ruff and `compileall` passed.
- The adapters consume the shared extension corpus; the C consumer, Java and Python suites cover registration/identity, limits, lifecycle, outbound order/criticality, recognition and preservation cases.

### Manifest generation and parity scaffolding (T052, T055)

- `python tools/api_manifest/generate.py --check` — passed; six generated outputs current.
- `.venv\\Scripts\\python.exe -m unittest discover -s tools/api_manifest/tests -v` — passed, 42 tests; two symlink tests were skipped because Windows denied symlink creation without the required privilege.
- `docs/development/api-manifest.md` documents write/regenerate and read-only `--check` workflows for maintainers.

### OASIS vectors and schema fuzz target (T056)

Red/Green/Refactor commits: `b8769b5` / `4165ac2` / `8acf3ae`.

- The Table 365 vector caught the wrong Extension Enumeration tag: the implementation returned `0x4200A8` instead of KMIP 2.1 §11.56's `0x420129`. Green corrects the allocation. Vectors cover §7.13 Table 365, §8.3 Table 396 repeated Message Extension ordering, §9.13 Table 418, and §11.44 Table 476.
- `cargo test -p kmipkit-protocol --test extension_information --test extension_oasis_vectors` — passed (2 + 4 tests).
- `cargo clippy -p kmipkit-protocol --all-targets --all-features -- -D warnings` — passed.
- `cargo check --manifest-path fuzz/Cargo.toml --bin extension_schema` and fuzz-target Clippy — passed.
- WSL Ubuntu, `cargo +nightly fuzz run extension_schema -- -runs=1000 -max_len=4096 -timeout=5` — completed 1,000 runs from the checked-in valid seed without a crash; coverage reached 463 counters and 697 features.
- `cargo fmt --all --check` and `git diff --check` — passed.

## 2026-10-07 — cross-adapter lifecycle Refactor (T054)

- Python Refactor: `d6aaf0a` — serialized native-handle ownership transfer and tightened copied identity/view lifetime handling; Python 3.12 CFFI suite passed 13/13.
- FFI Refactor: `c6afef9` — centralizes typed release validation and ownership consumption. C consumer verified NULL and wrong-kind releases are no-ops while the original live handle remains usable.
- Java Refactor: `030c912` — replaces mixed atomic/monitor handle state with `volatile long` and the existing lifecycle lock for writes; Cleaner, close, and transfer semantics remain unchanged.
- FFI/client Rust tests passed (192 client unit tests plus integration/UI/doctests); Clippy `-D warnings` and rustfmt passed. WSL FFI build, CMake/CTest (1/1), and C consumer (9 groups) passed.
- Java Maven suite passed 14/14 before and after Refactor; generated manifest check passed; parity test module passed 6/6; 91 native declarations match 91 JNI implementations.
- Python package suite passed 13/13 after Refactor, with Ruff, formatting, `compileall`, and 97-symbol CFFI audit passing.

## 2026-10-07 — CI API-manifest regeneration gate (T059)

- Red: `7bd2416`; `python -m unittest discover -s scripts/tests -p test_workflow.py -v` failed only the new workflow contract because no API manifest check was wired into CI.
- Green: `72bfe04` adds `python -B tools/api_manifest/generate.py --check` to the cross-platform script-contract job; any stale generated bytes now fail the job on Linux, Windows, or macOS.
- `python -m unittest discover -s scripts/tests -p test_workflow.py -v` — passed (19 tests).
- Pinned virtual environment, `python -B tools/api_manifest/generate.py --check` — passed; all six outputs match.

The Python adapter's Extension Information test expectation was also corrected to the §11.56 Extension Enumeration allocation `0x420129`; the old `0x4200A8` value is Fresh. This aligns Python with the OASIS-derived protocol and Java tests.

## 2026-10-07 — requirement traceability links (T057)

- `specification/compliance/requirements/KMIPKIT-0012.csv` contains 14 unique
  rows matching FR-001 through FR-014 in the approved feature specification.
- The FR-013 links identify the Rust, C, Java, and Python example files, both
  language guides, and the runnable example targets. FR-014 identifies the
  manifest generator tests and the CI `--check` invocation.
- A path and symbol audit of every `implementation_location` and `test_ids`
  entry found 0 unresolved references.

## 2026-10-07 — cross-language examples and user guidance (T029, T041, T058)

- English and Spanish guides now point to the actual Python example file,
  `vendor_extension_registry.py`, and explain registration, typed validation,
  explicit criticality, ordered attachment, recognition, unknown criticality
  handling, generic TTLV preservation, and runtime-owned copy limitations.
- Rust example run: `cargo run -p kmipkit-client --example vendor_extension` —
  passed; it demonstrates valid recognition, a schema-invalid generic payload,
  retained TTLV structure, and ordered outbound attachments.
- C example: `cargo build -p kmipkit-ffi`, bundled CMake configure/build, and
  CTest — passed (2/2, including the example and real consumer); strict GCC
  compilation and direct example execution also passed. The sample verifies
  recognition and generic preservation for valid and schema-invalid values.
- Java: `mvn -f bindings/java/pom.xml test` — passed (14/14); the JNI-backed
  example ran and printed its no-network confirmation.
- Python 3.12: example smoke test — passed (1/1); full package suite — passed
  (14 tests and 85 subtests). The sample uses shared fixtures and prints
  preserved TTLV tags plus ordered criticality selections.
- Rust, C, Java, and Python package examples document that unknown response
  criticality remains governed by KMIPKIT-0007 and describe the language's
  secret-copy/zeroization limits. All example inputs are local non-secret data;
  none sends a KMIP request.
- `git diff --check` — passed.

## 2026-10-07 — independent acceptance and security review (T062–T063)

- QA reviewed `5574a1b` read-only. It confirmed shared outbound fixtures
  exercise two caller-ordered attachments with explicit `false`/`true`
  criticality through Rust, C, Java, and Python. The public API audit found no
  raw-body, arbitrary-Item, caller-conversion, or executable-plugin attachment
  route; compile-fail fixtures cover these boundaries. It also confirmed that
  FR-014 links the manifest, generators, shared corpus, outputs, tests, and CI.
- The QA process finding is resolved by the standing direct user authorization
  recorded in `spec.md` and above. It authorizes implementation without further
  approval requests and explicitly does not waive PR approval or merge rules.
- Security review of `be9efb1..5574a1b` found no reportable vulnerabilities.
  It verified bounded runtime extension validation, redacted diagnostics,
  JNI-owned scratch-buffer cleanup, read-only CI permissions, and the fixed
  real-C-consumer sanitizer job. Its CI findings about omitted generator
  rejection tests, dependency installation, and Windows Python selection were
  fixed in `aad390b` through `a01a430` and verified by current-HEAD workflow
  checks on Windows and macOS.
- Residual review notes: the build-time fixture generator has no global
  byte/depth/record ceilings; its inputs are version-controlled build fixtures,
  not runtime or network input. Self-hosted runner isolation and cleanup are
  configured outside the repository and were not established by this review.
  A qualified human security review remains a separate 1.0.0 release gate.
- T062 and T063 are complete on this evidence. Coverage thresholds and the full
  current-HEAD platform CI remain open under T060 and T061.

## 2026-10-07 — draft PR evidence refresh (T064)

- Updated draft PR #52 from the terminal with Red/Green/Refactor commit groups,
  current local verification, the six generated artifact classes, independent
  QA/security results, and known release risks.
- Recorded current-head Actions run `37654461313` at `c20f823` as still in
  progress. T060/T061 remain open until the multi-platform matrix and aggregate
  coverage gate complete successfully.
- PR body: https://github.com/NeverWe1come/KMIPKit/pull/52

## 2026-10-07 — Rust 1.94 FFI coverage-harness Clippy regression (Red)

- Current-head CI run `37654461313`, job `112905929106`, failed the Rust 1.94
  Clippy gate in `crates/kmipkit-ffi/tests/c_api_coverage.rs`: `borrow_as_ptr`
  at the raw FFI output pointer and possible truncation/wrap from `usize as i32`.
- Reproduced before changing code with WSL Ubuntu and Rust 1.94.1:
  `cargo +1.94 clippy -p kmipkit-ffi --test c_api_coverage --all-features -- -D warnings`.
  It failed with the same three diagnostics.
- The Ubuntu coverage job `112905929059` did not execute its coverage steps; GitHub
  reported that the job failed to be acquired after five attempts. This is a
  runner-acquisition failure, not a coverage-threshold result, and must be rerun.
- No source implementation changes are included in this Red evidence.
## 2026-10-07 — Rust 1.94 FFI coverage-harness Clippy fix (Green)

- `crates/kmipkit-ffi/tests/c_api_coverage.rs` now passes an explicit `&raw mut`
  output pointer and converts the C `argc` with checked `i32::try_from`.
- WSL Ubuntu / Rust 1.94.1:
  `cargo +1.94 clippy -p kmipkit-ffi --test c_api_coverage --all-features -- -D warnings` — passed.
- WSL Ubuntu / Rust 1.94.1:
  `cargo +1.94 test -p kmipkit-ffi --test c_api_coverage --all-features -- --nocapture` — passed (1 Rust test); the linked C consumer passed all 13 groups.
## 2026-10-07 — FFI coverage-harness unsafe-scope Refactor

- Split the FFI defaults, release, and C-consumer calls into individual unsafe
  blocks with one `SAFETY` explanation per call. Assert successful status and
  non-null handle separately so the test fails if the C API returns no handle.
- Rust 1.94.1 verification: `cargo +1.94 fmt --all --check` — passed;
  `cargo +1.94 clippy -p kmipkit-ffi --test c_api_coverage --all-features -- -D warnings`
  — passed; `cargo +1.94 test -p kmipkit-ffi --test c_api_coverage --all-features -- --nocapture`
  — passed (1 test; 13 C consumer groups).

## 2026-10-07 — workspace coverage gate shortfall (T060 Red)

- The clean Linux Rust 1.94 workspace coverage run, collected with an isolated
  `CARGO_TARGET_DIR`, does not meet T060's unchanged thresholds. The raw report
  is `/home/ramp1953/kmipkit-0012-coverage-full.json` (outside the worktree).
- Measured totals are TTLV 712/750 (94.93%, minimum 95%), protocol 3979/4242
  (93.80%, minimum 95%), client 2504/2673 (93.68%), transport 88/88 (100%),
  preliminary FFI 1428/2180 (65.50%), and Rust workspace 8943/10169 (87.94%,
  minimum 90%). The preliminary FFI workspace report does not include the
  separately collected C-consumer ABI coverage report; FFI must be assessed
  from that dedicated report before its 85% gate can be concluded.
- Segment inspection found the TTLV-owned `try_clone_value` arms and
  `Structure::default` uncovered. Protocol gaps include schema-validation and
  message/credential error paths; the complete map is retained in the raw LLVM
  report for follow-up. No generated-source exclusion or threshold change is
  proposed.
- Red command/result: `cargo llvm-cov --workspace --all-features --locked
  --json --output-path coverage-raw.json` followed by the documented
  `coverage_gate.py normalize` step completed, but the resulting package and
  workspace totals are below T060's required gates. This is a coverage-gate
  failure, not a Rust test failure. T060 remains open pending targeted
  behavior tests and a fresh measured report.

## 2026-10-07 — targeted Rust coverage tests (Green; T060 remains open)

- Green development commit: `1ad6080` (`test(coverage): add targeted protocol coverage`).
- Added behavior assertions for deep TTLV value cloning and child-order
  preservation; definition compatibility and safe error-category display;
  schema-only mismatch preservation and codec-limit propagation; a scalar
  intermediate discriminator path; required bit-mask type rejection; cloning
  definitions both with and without Extension Information; malformed required
  message-field types; and the typed credential-type accessor.
- Rust 1.94 focused tests passed:
  `cargo +1.94 test -p kmipkit-ttlv --test value_clone`;
  `cargo +1.94 test -p kmipkit-protocol --test extension_definition`;
  `cargo +1.94 test -p kmipkit-protocol --test extension_error`;
  `cargo +1.94 test -p kmipkit-protocol --test extension_schema`;
  `cargo +1.94 test -p kmipkit-protocol --test message_validation`;
  `cargo +1.94 test -p kmipkit-protocol --test credential_public_api`.
- A fresh clean Linux Rust 1.94 run passed all workspace test binaries and
  emitted `/home/ramp1953/kmipkit-0012-coverage-final.json` using an isolated
  `CARGO_TARGET_DIR`. Raw line summaries are TTLV 737/750 (98.27%), protocol
  4006/4242 (94.44%), client 2504/2673 (93.68%), transport 88/88 (100%), FFI
  Rust-only 1428/2180 (65.50%), and Rust workspace 8995/10169 (88.46%). TTLV
  clears its gate; protocol remains 24 covered lines short of the 4030/4242
  minimum. The raw workspace figure does not include the separately normalized
  C-consumer FFI data.
- The dedicated normalized FFI report remains 2009/2180 (92.16%) with all 13 C
  consumer groups passing, above the 85% FFI gate. Adapter, changed-production
  line, and cross-platform aggregate results have not been collected here; the
  repository 90% aggregate must be determined by merging the complete CI
  artifacts.
- `cargo +1.94 fmt --all --check` passed. No production behavior, thresholds,
  generated output, or coverage exclusions changed. T060 and T061 remain open.

## 2026-10-07 — definition-accounting coverage follow-up

- Added public-contract assertions for accounting totals with and without
  optional Extension Information, including text bytes, maximum field size,
  and discriminator bytes. `cargo +1.94 test -p kmipkit-protocol --test
  extension_definition` passed (13 tests).
- Repeated the clean Linux Rust 1.94 workspace coverage command after the
  follow-up; all test binaries passed and line totals remain TTLV 737/750,
  protocol 4006/4242, and workspace 8995/10169. The additional scenario
  confirms the accounting behavior but does not cover a new production line.
  No production behavior or gate configuration changed.

## 2026-10-07 — coverage-test fixture Refactor

- Refactor evidence: consolidated the repeated
  malformed-response header setup in `message_validation.rs` into
  `response_tree_with_header_fields`; the fixture still contains the same
  valid response batch item, and no production behavior changed.
- Rust 1.94.1: `cargo +1.94 test -p kmipkit-protocol --test
  message_validation` passed (15 tests); `cargo +1.94 fmt --all --check` and
  `git diff --check` passed. Coverage totals are unchanged from the Green
  report, and T060/T061 remain open.

## 2026-10-07 — repeated-child clone-order QA correction

- Independent QA found that the original clone-order fixture contained one
  nested child, so its sequence comparison could not detect reordering.
- Added a separate case with three same-tag children carrying distinct text
  values. A mutation experiment temporarily reversed the production clone
  iteration; the new test failed on the first-versus-third value comparison.
  The production source was restored without a retained change.
- After restoration, `cargo +1.94 test -p kmipkit-ttlv --test value_clone`
  passed both tests. `cargo +1.94 fmt --all --check` and `git diff --check`
  passed. No production behavior changed.
- The first PR CI run exposed `clippy::match_same_arms` in the shared value
  comparison helper on macOS and Windows. The compatible `i64`, `u32`, and
  byte-slice variants now share match arms; no comparison behavior changed.
- Red: `cargo +1.94 clippy --workspace --all-targets --all-features -- -D
  warnings` reported the three identical match-arm pairs. Green/refactor:
  after merging those arms, the same full-workspace Clippy command passed;
  `cargo +1.94 test -p kmipkit-ttlv --test value_clone`, formatting, and
  `git diff --check` also passed.

## 2026-10-07 — allocation-before-limit boundary regressions (Red)

- Added Python probes for TTLV bytearray materialization, over-limit identity
  text encoding, and schema/registry collection traversal; added Java probes
  for schema and registry list traversal.
- Python Red: `bindings/python/.venv/Scripts/python.exe -m pytest -q
  bindings/python/tests/test_extension_registry.py -k precedes` failed all
  four new cases. The bytearray was converted, text was encoded, and the
  oversized collections were traversed before the expected resource-limit
  error.
- Java Red: `mvn -f bindings/java/pom.xml -Dtest=ExtensionRegistryTest test`
  failed the two new cases because `List.copyOf` and schema iteration reached
  the adversarial `get` methods before rejecting their sizes.
- C ABI Red: `cargo +1.94 test -p kmipkit-ffi
  registry_construction_checks_aggregate_limits_before_definition_clones --
  --nocapture` failed because one definition clone occurred before the
  aggregate schema-node limit returned `ERROR_RESOURCE_LIMIT`.
- These are expected Red outcomes; the production fix and Green evidence are
  not recorded in this commit.

## 2026-10-07 — true-size and underreported-list probes (Red)

- Strengthened the Python collection probes to contain 257 and 4,097 actual
  list entries while overriding `__len__` to underreport and `__iter__` to
  detect traversal. The same four focused Python cases still failed for the
  expected preflight reasons before the fix:
  `PYTHONPATH=bindings/python/src bindings/python/.venv/Scripts/python.exe -m
  pytest -q bindings/python/tests/test_extension_registry.py -k precedes`.
- Added a Java list whose advertised size is zero but whose iterator yields
  more than the registry maximum. The focused test failed because registry
  creation returned without enforcing the actual bounded iteration count:
  `mvn -f bindings/java/pom.xml
  -Dtest=ExtensionRegistryTest#registryDefinitionLimitBoundsAListThatUnderstatesItsSize
  test`.

## 2026-10-07 — bounded recursive schema preflight (Red)

- Added protocol unit probes for aggregate schema-node and constraint-member
  limits. Both fail before the fix because `validate_registry_limits` returns
  success after traversing every sibling despite the lowered aggregate budget.
- Red: `cargo +1.94 test -p kmipkit-protocol --lib
  t065_aggregate_preflight_tests -- --nocapture` failed both expected cases;
  the assertions reported `Ok(())` where `ResourceLimit` was required. Test-only
  visit instrumentation also records schema recursion depth for Green checks.

## 2026-10-07 — bounded recursive schema preflight (Green)

- The schema validator now counts aggregate nodes and constraint members during
  recursion and returns at the first over-budget node. Client registry preflight
  passes the remaining budgets before full accounting or any definition clone.
- Green: `cargo +1.94 test -p kmipkit-protocol -p kmipkit-client -p kmipkit-ffi
  --all-features` passed, including both bounded-visit probes and the existing
  exact/over aggregate-boundary tests. T065 remains open for Refactor and final
  validation evidence.
- The Java and Python allocation-boundary fixes also pass their adapter suites:
  `mvn -f bindings/java/pom.xml test` (28 tests plus the runnable example) and
  `PYTHONPATH=bindings/python/src bindings/python/.venv/Scripts/python.exe -m
  pytest -q bindings/python/tests` (25 tests). Rust FFI tests ran in the
  three-crate test command above.
- Focused Clippy for protocol, client, and FFI, workspace formatting check, and
  `git diff --check` pass after the test helper lint correction.
- Refactor: centralized checked aggregate-counter updates without changing the
  limit outcomes. After refactor, `cargo +1.94 test -p kmipkit-protocol -p
  kmipkit-client -p kmipkit-ffi --all-features`, focused Clippy for the same
  crates, `cargo +1.94 fmt --all --check`, and `git diff --check` passed.
- T065 evidence is separated into commits: Red `f4eec99`, Green `f702bbd` and
  `3e83d04`, Refactor `3c41a77`. The wider T060 coverage gate and T061 final
  release validation remain open.

## 2026-10-07 — full post-fix workspace coverage

- Windows Rust 1.94: `cargo +1.94 llvm-cov --workspace --all-features
  --summary-only` exited successfully, with 77.74% line, 76.27% function, and
  80.21% region coverage. This is a single-platform summary, not the repository
  aggregate: the 90% workspace gate merges Linux, Windows, and macOS reports.
  T060 and T061 remain open pending the aggregate CI result.

## 2026-10-07 — security remediation for schema clones and Java byte copies

- The independent diff scan `49ad7ef4-80d1-489d-8625-9c849c642ab4` reported
  two findings: repeated deep clones of caller-composed extension schemas in
  the C ABI before aggregate schema-budget enforcement, and Java factory
  temporaries that retained KMIPKit-owned secret bytes after synchronous JNI
  calls. This entry records the remediation; a fresh diff scan is still
  required for the final branch head.
- Red evidence is separated into commits: `501e476` adds the pointer-sized
  schema-clone regression; `7b33026` adds aggregate node/constraint hard-limit
  regressions; `5dc9276` adds Java temporary-copy success/failure probes. The
  Rust tests failed before the shared immutable schema/preflight fix, and the
  Java test failed before the zeroizing JNI-copy helper existed.
- Green evidence: `96cc464` changes `ExtensionSchema` to a shared immutable
  handle, caches checked depth/node/constraint counts, and rejects aggregate
  construction limits before compiling indexes; `30c5e44` clears KMIPKit-owned
  temporary Java byte arrays in `finally` after synchronous JNI success or
  failure. Caller arrays remain unchanged, and documentation describes the
  JVM-copy boundary. Signed Green commit `4c8b2df` handles Java copies.
- Refactor evidence: signed commit `d77b7fd` simplifies cached metric access,
  makes the new Java helper test direct instead of reflective, applies strict
  Clippy `let-else` style to the boundary tests, and updates English/Spanish
  guidance, FR-011/FR-012 traceability, and tasks T066/T067. Its SSH signature
  was verified with the loaded ED25519 key.
- Final local checks on Windows, Rust 1.94.1: `cargo +1.94 fmt --all --check`,
  `cargo +1.94 clippy --workspace --all-targets --all-features -- -D warnings`,
  and `cargo +1.94 test --workspace --all-features` passed. The first Clippy
  pass identified four `manual_let_else` test lints; those were corrected and
  the complete Clippy command passed afterward. The workspace test command
  exited 0, including the schema hard-boundary and clone-handle regressions.
- `cargo +1.94 llvm-cov --workspace --all-features --summary-only` exited 0
  with 77.80% Windows line coverage (76.27% functions; 80.30% regions). On
  this exact Windows report, all 111/111 changed executable Rust lines and
  all 12/12 changed executable Java lines were covered. The local Rust FFI
  report does not include the Linux-only C-consumer coverage feature, and a
  single-platform total does not establish the required three-platform 90%
  workspace gate; T060 remains open for the aggregate CI result.
- `mvn -f bindings/java/pom.xml verify` passed all 29 Java tests, the runnable
  vendor-extension example, and the configured JaCoCo coverage check.
  `PYTHONPATH=bindings/python/src bindings/python/.venv/Scripts/python.exe -m
  pytest -q bindings/python/tests` passed all 25 tests. The Python API manifest
  check verified all six generated files. Python, C-consumer, FFI-sanitizer,
  fuzz, dependency-policy, and supported-platform coverage for the corrected
  head remain subject to the fresh pull-request CI run; T061 remains open.

## 2026-10-07 — JNI ownership and fixture-writer security fixes

- The independent diff scan `70e5c9a7-b2a1-47ce-8a0c-d7edf8dd29a0` found two
  low-severity issues: malformed UTF-16 could orphan a transferred Java
  native handle, and the extension fixture writer could follow a symlink or
  reparse point outside the checkout. The additional Rust/TTLV review found
  no new candidate. A separate unbounded coverage-XML candidate was suppressed
  because the pull request controls the producer and the read-only aggregator
  crosses no privilege or secret boundary.
- Red commit `fc5850f` adds both regression tests. The Java case failed because
  the invalid-description call consumed the original native handle; the
  generator case failed because a Windows directory junction redirected the
  write outside the checkout. The file-symlink variant is skipped on this
  Windows host because creating file symlinks requires an unavailable
  privilege.
- Green commit `819f827` rejects unpaired UTF-16 surrogates before Java handle
  transfer and adds resolved-root, symlink/reparse-point checks with atomic
  output writes to the fixture generator. The 16-test fixture module passed
  (one platform-limited skip), and the focused Java ownership regression passed.
- Refactor commit `512979f` isolates the preflighted multi-output write path.
  After refactor, the fixture module passed again (16 tests, one platform-limited
  skip), `mvn -f bindings/java/pom.xml test` passed all 30 Java tests and the
  runnable example, `python -m py_compile` passed for the generator and test
  module, and `git diff --check` passed. The full supported-platform CI and a
  fresh security scan of the updated diff are still pending; T060/T061 remain
  open.
- Final local contracts also passed: script tests (199 passed, 26 expected
  cargo-deny skips), normative catalog tests (170 passed, 7 platform skips),
  API manifest tests (51 passed, 3 platform skips), both generator `--check`
  commands, and `mvn -f bindings/java/pom.xml verify` (30 tests, the runnable
  example, and JaCoCo coverage gate).

## 2026-10-07 — final security review and CI coverage-tool follow-up

- Fresh diff scan `a8989cb5-601a-443e-9d79-3e696a2cbf3b` reviewed the Java/JNI invalid-UTF-16 ownership guard, fixture-generator path checks, atomic writes, FFI coverage collector, related tests, and documentation in `5482c34..6e6172f`. It completed with zero candidates and zero findings; both previously reported low-severity issues were fixed by the tested regressions.
- The remote CI/tooling commits `897f44b` and `96dee30` were merged into this branch as `05c23b7`. A second complete diff scan, `6ce18f02-921b-4339-ba4f-c162bc948a85`, reviewed the exact `6e6172f..05c23b7` range across the workflow, FFI coverage collector, and workflow contracts. It completed with zero candidates and zero findings.
- After that integration, `git diff --check 6e6172f..HEAD` passed and the complete script-contract suite passed: 199 tests, with 26 expected skips because pinned `cargo-deny` is not installed on this Windows host.
- T060/T061 remain open until the updated PR head passes its fresh GitHub Actions matrix and aggregate coverage gate. The WSL run of `bash scripts/collect_ffi_coverage.sh /home/ramp1953/kmipkit-0012-ffi-coverage-report` with an isolated `CARGO_TARGET_DIR` exited 0: the C consumer test passed all 13 groups and the report recorded 92.14% line, 88.19% function, and 86.54% region coverage. An initial run reused Windows-generated trybuild metadata under `target` and failed during coverage export; isolating the Linux target directory removed that environment collision.

## 2026-10-07 — hosted JDK JNI path regression correction

- The PR branch advanced with a broader JNI-header exclusion and a fixture-generator path-alias fix. After merging those remote commits as `bd0b05e`, the hosted-JDK contract test initially missed a project-header false positive on Windows because `Path.__str__` produced backslashes. Normalizing the test path to POSIX exposed the issue: Red commit `466e08b` failed because `bindings/java/native/include/jni.h` matched the broad exclusion.
- Green commit `42bcda1` limits JNI exclusions to `jni.h` and `jni_md.h` beneath the known `/usr/lib/jvm/<jdk>/include` and GitHub-hosted `/opt/hostedtoolcache/Java_<distribution>/<version>/<architecture>/include` roots, plus the existing C public header. The regression checks hosted and system JDK headers, including `include/linux/jni_md.h`, and confirms project headers remain included. The focused test module passed (12 tests); the complete script suite passed (200 tests, 26 expected skips because pinned `cargo-deny` is unavailable on this Windows host); `git diff --check` passed.
- A fresh WSL Linux `cargo +1.94.0 llvm-cov --workspace --all-features --locked --json` run exited successfully. Its single-platform report measured TTLV 737/750 (98.27%), protocol 4112/4349 (94.55%), and workspace 9181/10363 (88.59%). This does not satisfy the local single-report protocol/workspace percentages; the repository gate uses the supported-platform aggregate, so T060/T061 remain open until the latest PR head completes that gate.
- The GitHub Actions run on the prior remote head `c6847fa` was still in progress when this evidence was recorded. It does not include the scoped-exclusion correction; a fresh run on the pushed head is required. A fresh independent security diff scan of the integrated generator and coverage changes is also pending.
