# KMIPKIT-0092 C route evidence implementation (WIP)

The approved KMIPKIT-0092 contract is consumed directly by
`interop/c-kmip-validator/tools/route_matrix.py`. It invokes the installed public
C ABI for exact local catalog values and the compiled C consumer for controlled
network recipes. Library sources are unchanged.

| Requirement | C evidence and remaining work |
|---|---|
| KMIPKIT-0092-FR-001, FR-006 | Shared matrix generator, installed header/symbol/constant probe, exact `release/1.0.0` pin `9a16cc90a12205101d22a38501a5e74c665df4dc`, library digest, compiled-consumer provenance comparison. |
| KMIPKIT-0092-FR-002, FR-007, FR-009 | Per-cell evidence and explicit pending sidecar; duplicate protection; common schema and validator in CI. Incomplete recipes keep the gate failing. |
| KMIPKIT-0092-FR-003 | Exact assigned scalar values, reserved-tag rejection, typed attributes, and cryptographic-parameter field/TTLV round trips are checked locally. Broader semantic branch/model recipes remain pending. No absent encoder/send/high-level API is fabricated. |
| KMIPKIT-0092-FR-004 | Ordered, redacted tree digests are retained per cell. Cross-language digest normalization/parity is pending. |
| KMIPKIT-0092-FR-005 | Controlled peers parse all six encoding/transport pairs, check recipe fields, numeric ordered results, lifecycle transitions and deletion. Check, Get Attribute List, Log, and Revoke response TTLV accessors are read and structurally checked. Cryptographic, asynchronous, and remaining positive operation recipes are pending. |
| KMIPKIT-0092-FR-008 | Reports contain public identities, axes, digests, sanitized failure codes and numeric outcomes. Request bodies and fixture keys/tickets never enter evidence. |

The exact OASIS document and section remain on each case in the generated shared
catalog. Scalar generic-TTLV checks follow KMIP 2.0/2.1 Specification §10.1; local
numeric tag/value fixtures follow the source references attached to the exact
normative target. Controlled recipe tests follow each operation's catalog
§6.1 reference and verify ordered batch structure from Specification §8.

This work does not satisfy the complete 0092 C matrix, the 1.0 coverage gates,
or vendor interoperability acceptance. PR #170 remains a draft marked WIP.

## CI repair Red evidence (2026-11-12)

PR #170 head `02b4aafe3964825829f046bbdfbb3b53e3e62d54` failed
existing executable tests before this repair:

- CI run `43297892137`, Ubuntu stable job `131461810582`: workspace
  tests `operation_parity_0053` and `c_api_coverage` fail to link
  `-lkmipkit_ffi`.
- The same run, Windows Rust 1.94 job `131461810515`: `c_api_coverage`
  links both the Rust rlib and the native library, producing LNK2005/LNK1169.
- Coverage Ubuntu job `131461810884`: the instrumented
  `operation_parity_0053` test fails to find the native library under the
  newer Cargo build-output layout. The selected coverage gate consequently
  rejects the incomplete producers.

Reproduce the affected executable tests with
`cargo test --locked -p kmipkit-ffi --all-features --test operation_parity_0053 --test c_api_coverage`.
The regression is explicit native-library linkage in tests that must resolve
C ABI symbols from their instrumented Rust crate, in the same process.
The complete C matrix remains required and pending recipes must keep failing.

### Green linkage verification

The Red command above was reproduced locally with Cargo 1.100.0 in an isolated
Linux target directory: `operation_parity_0053` failed to find `-lkmipkit_ffi`.
After removing native-library linkage from both Rust integration tests and
explicitly retaining the Rust crate in the operation parity test, the same
command passes: `c_api_coverage` 50/50 and `operation_parity_0053` 1/1.
Compiled C consumers still invoke the exported ABI and now resolve it from the
same Rust rlib as the coverage process. This avoids a second uninstrumented
native library and its Windows duplicate symbols; Windows requires exact-head
CI verification. No library API or implementation was changed.

### Exact placeholder batch and independent cleanup

The shared evidence validator exposed a second regression: a successful
five-operation peer recipe did not match the catalog's exact
Create/Activate/Get scenario. The new installed-consumer regression
`test_placeholder_batch_evidence_matches_the_catalog_before_cleanup` first
failed with `batch request operation order does not match the catalog`.
The driver and controlled peer now retain the three-operation batch and issue
Revoke/Destroy separately, using the returned identifier. The five focused
placeholder tests pass across both KMIP versions and all six wire pairs.
The peer still rejects arbitrary identifiers and accepts the normative omitted
or explicit ID Placeholder representation.

### Controlled peer state evidence

The approved shared base at `1f6f73f0c` requires before/after SHA-256 state
observations whenever batch effects are emitted. The C peer now owns a small
fixture object-state map, resolves each actual request identifier, and applies
Create/Register/Activate/Revoke/Destroy and read operations before generating
their response statuses. Only public fixture identifiers and lifecycle states
enter the redacted digest; request bodies and key material do not.

The Continue regression verifies that a missing-key failure is followed by
an executed read of the existing object, while its object state remains
unchanged. The ID Placeholder regression verifies an empty initial store, an
active object after the three-item scenario, and deletion after independent
cleanup. Both tests first failed for missing effects/state hashes; four
focused state, lifecycle, and correlation tests now pass. The shared schema
and completeness validator retain all of their existing requirements.

### Cargo-independent FFI coverage artifact discovery

The Linux FFI collector also failed after the full LLVM export because it
assumed every library and executable lived in `debug/deps`. Cargo 1.100 uses
per-artifact output directories. The collector now uses the executables
reported by the completed Cargo test run and Cargo's current compiler-artifact
JSON, captured under the exact environment emitted by cargo-llvm-cov. It keeps
both the complete shared/static libraries and the unit/C-consumer executables;
missing, foreign, or ambiguous current artifacts fail closed. Unreferenced
stale build outputs cannot enter the export. A single merged profile is
required without assuming the checkout directory is named KMIPKit.

Six discovery tests cover both Cargo layouts, paths containing spaces,
hardlinks, missing libraries, ambiguous metadata, and foreign executables.
All 96 workflow tests pass. The actual collector passes with Cargo 1.100
and writes its normalized C ABI coverage report. The full workspace LLVM
collection also completes, with 4,086 instrumented tests passed and three
repository-marked tests ignored. All three-platform and adapter gates remain
required; these local runs do not claim that the aggregate gate has passed.

### Expanded Tag targets use their actual tag bytes

The updated shared contract classifies reserved decoding from each expanded
normative target. The C coordinator now executes those Tag-family target
cells through the actual raw/checked tag and decoder path instead of reusing
an aggregate numeric enumeration roundtrip under a surrogate Tag field.
The regression selects an assigned and a reserved target, calls the real
installed ABI, checks the assigned target tree and reserved decoder rejection,
and retains failure of the complete gate for a selected subset.

### Coverage helper CLI confinement

Sonar identified unrestricted log/receipt paths and an unnecessary foreign
Cargo manifest override in the new helper. The manifest override is removed;
only this repository's manifest is compiled. Report directory creation and
fixed log/receipt paths follow the allowed output roots documented in
`docs/development/testing.md`, and metadata replacement is atomic. Twelve Linux
functional tests, all 96 workflow contracts, and the actual Linux coverage
collector pass with the guarded CLI. Exact-head Sonar verification remains
required before treating the quality gate as satisfied.

The real-shell symlink regression first demonstrated an output-log alias
overwriting a foreign fixture before Cargo failed. Preparation now validates
all fixed output paths before invoking Cargo or redirecting a report. The
foreign fixture remains unchanged, and the full collector still succeeds.
Sonar reported zero new issues and zero hotspots at `a6407e43e`; later heads
still require their own quality-gate result.

The subsequent exact-head CI run `43419410034` exposed one remaining Windows
short-path assertion in `test_preserves_both_libraries_and_consumers_in_both_cargo_layouts`
(job `131866846011`, classic layout: `RUNNER~1` versus `runneradmin`). The
expectation now compares file identity for every collected artifact, as the
hardlink case already does. All 12 local tests in
`scripts.tests.test_ffi_coverage_objects` pass; exact-head CI is required to
verify this follow-up on Windows.

### Register request and response routes

The Red commits `7cafc62a2`, `22fff7a51`, and `b42bdc902` added installed-
consumer checks for the missing Register recipe and its generic response-TTLV
accessor. Red commit `e6526470f` requires Certificate and Certificate Request
Register exchanges, and `77dde2ba5` requires their direct typed object
constructors for both protocol versions. The C consumer registers an isolated
certificate or generated PKCS#10 Certificate Request, checks the typed
Register response, checks that the generic response TTLV contains the same
Unique Identifier, and verifies independent Destroy cleanup. The Certificate
Request fixture uses an RSA-2048 key and required Algorithm, Length, and Name
attributes. The controlled peer validates both KMIP 2.0 and 2.1 over TTLV,
JSON, and XML with raw TLS and HTTPS; request and response bodies remain out
of evidence.

Verification on the implementation worktree:

- CMake build and CTest: 7/7 passed.
- `python3 -m unittest interop.c-kmip-validator.tests.test_route_matrix -v`:
  77/77 passed.
- `python3 -m unittest scripts.tests.test_ffi_coverage_objects -v`: 12/12
  passed.
- The complete generated C route matrix: 66,728 expected cells, 36,942
  invoked, 0 failed, and 29,786 still pending. All cells for
  `test-op-register`, `test-obj-certificate`, and
  `test-obj-certificate-request` were invoked; none remain pending for those
  cases. The completeness command correctly exits unsuccessfully while the
  other recipes remain incomplete.

This Register path is one executable operation recipe, not completion of the C
interoperability implementation. In the same matrix, the largest remaining
gaps include the Register/Encrypt/Decrypt batch (304 cells), asynchronous
Cancel outcomes, and multiple operation and object routes. The PR remains WIP.

### Refactor verification

The response-TTLV comparison now returns the typed status directly instead of
combining a boolean with an output status. After that refactor, the CMake build
and CTest passed 7/7, and the focused installed Register exchange passed 1/1.

The fixture readers for Certificate and Certificate Request now share one
bounded binary-file helper. The helper rejects empty and oversized files,
checks read completion, and clears owned fixture bytes before freeing them.
After this refactor, CMake build passed, CTest passed 7/7, the C route matrix
unit suite passed 77/77, and the FFI coverage object suite passed 12/12.

### Add Attribute direct and standalone routes

The installed consumer now exercises Add Attribute through both the direct
batch-item constructor and the standalone `AddAttributeRequest` constructor.
Each test creates an isolated symmetric key, obtains its text Unique
Identifier, sends the attribute mutation as a separate request, verifies the
typed response identifier, then revokes and destroys the object. The controlled
peer validates the KMIP 2.0/2.1 request fields, observes the attribute state
change, and confirms cleanup returns the fixture to its initial state. The
direct and standalone requests have identical normalized request-tree digests
for each encoding and transport.

Verification on the implementation worktree:

- Focused Add Attribute integration: 24/24 exchanges passed (two protocol
  versions, six encoding/transport pairs, and both constructor routes).
- CTest: 7/7 passed; this includes 78 C route-matrix contract tests and the
  controlled KMIP 2.0 and 2.1 TLS peers.
- FFI coverage object tests: 12/12 passed.
- Complete generated C route matrix: 66,728 expected cells, 37,018 invoked,
  0 failed, and 29,710 pending. All 80 Add Attribute cells passed. The
  completeness gate remains intentionally unsuccessful while other recipes
  remain pending.

This route does not complete C interoperability coverage. Batch attribute
mutators, the remaining operations and objects, asynchronous flows, and other
catalogued variants still need executable C recipes. The PR remains WIP.

### Set Attribute direct and standalone routes

The installed consumer now exercises Set Attribute through both the direct C
batch-item constructor and standalone `SetAttributeRequest` constructor. It
creates an isolated symmetric key, obtains its returned text Unique Identifier,
sends Set Attribute with a Comment, validates the typed response identifier,
and revokes and destroys the object. The controlled KMIP 2.0/2.1 peer checks
the Set Attribute operation value and request fields, records the attribute
mutation, and verifies cleanup restores the fixture state. For each version,
encoding, and transport, the direct and standalone requests have matching
normalized request-tree digests.

Verification on the implementation worktree:

- Focused Set Attribute integration: 24/24 exchanges passed (two protocol
  versions, six encoding/transport pairs, and both constructor routes).
- CTest: 7/7 passed, including both controlled KMIP 2.0 and 2.1 TLS peers.
- FFI coverage object tests: 12/12 passed.
- The complete C route evidence matrix records 66,728 expected cells, 37,094
  invoked, 0 failed, and 29,634 pending. It records 80 passed events for each
  of Add Attribute and Set Attribute; each operation also passed all 24
  controlled peer combinations.
- The completeness validator still exits unsuccessfully because catalogued
  operation, object, attribute, batch, asynchronous, and normative constructor
  routes remain pending.

The PR remains WIP while the C catalogue has outstanding operation, object,
attribute, batch, asynchronous, and normative constructor routes.

### Delete Attribute and cleanup-state evidence

The installed consumer now exercises Delete Attribute with both direct and
standalone request constructors. It creates an isolated symmetric key, seeds a
Comment attribute, deletes that attribute by its exact Tag reference, checks the
typed response identifier, then revokes and destroys the key. The controlled
peer verifies the seed, mutation, cleanup operation order, and restoration of
the initial fixture state. The placeholder-chain recipe also reports the
observed revoke/destroy cleanup operations and final state.

Historical verification against the previous feature-branch source pin
`8281fef2e9ca211fde1e1d8e21525a672287eead` (superseded by exact release-pin
verification below):

- Artifact build and provenance verification passed; library SHA-256 is
  `e2b7a9c21c22551cbbf5ff55d7c1997525413bc2adbe958577283e95e6993d7b`.
- The focused Delete Attribute controlled-peer test passed all 24 combinations
  (two protocol versions, six encoding/transport pairs, and two constructors).
- CTest passed 7/7, including all 80 route-matrix contract tests and the
  controlled KMIP 2.0/2.1 TLS peers.
- The full matrix recorded 66,728 expected cells, 37,170 invoked, zero failed,
  and 29,558 pending. Add Attribute, Set Attribute, and Delete Attribute each
  recorded 80 passed events, including 24 controlled-peer exchanges.
- The completeness validator continues to fail on the pending recipe cells.
  This branch is not ready for review until the remaining matrix routes and
  exact-head CI pass.

### Exact release pin and typed local route follow-up

The C artifact and CI now use the shared manifest's exact `release/1.0.0`
commit `9a16cc90a12205101d22a38501a5e74c665df4dc`. The rebuilt external C
artifact was verified with SHA-256
`2eeffb9357b08da5812faf96b3a1b801a5f998c4f29513efa3d1848100298023`.

The cryptographic-parameters route now builds a typed value from fields,
converts it to TTLV, reads that TTLV back with
`kmipkit_cryptographic_parameters_from_ttlv`, converts the returned typed value
to TTLV again, and checks the exact ordered tree. A Red assertion first failed
because the route was not invoked; the Green implementation now exercises it
for each applicable field fixture. Local typed attribute routes also run
independently for cases that have a separate server-exchange scenario.

Verification on the current C worktree:

- Python validator tests: 91/91 passed.
- Full route matrix: 66,728 expected cells, 37,227 invoked, zero failed, and
  29,501 still pending. The completeness gate correctly remains unsuccessful.
- CTest: 7/7 passed after the latest route changes.
- Cryptographic Parameters `from_ttlv` Red/Green/Refactor commits are
  `72f2593c3`, `ddb2540d2`, and `796239c33`; the focused field and extension
  tests pass.
- Check, Get Attribute List, Log, and Revoke response TTLV accessors now run
  from observed responses for both protocol versions. The focused Red test
  failed for all four missing routes; Red, Green, and Refactor commits are
  `a30493064`, `6802292ba`, and `50ac4fc89`. The full matrix records twelve response-TTLV
  route events across those operations and the Create/Activate/Get Attribute
  List/Revoke/Destroy workflow.

This is incremental evidence only. Thousands of C route cells and the Rust,
Java, and Python validators still require implementation and exact-head CI.
The PR remains a WIP draft.

### Get Usage Allocation optional identifier and Locate placeholder route

The KMIP 2.0 §6.1.22 Table 214 and KMIP 2.1 §6.1.23 Table 232 recipes now
execute a three-item ordered batch: Locate the fixture's single active,
usage-limited object; allocate one unit using its explicit Unique Identifier;
then allocate one unit with Unique Identifier omitted. The omission consumes
the ID Placeholder established by Locate. Get Usage Allocation is not itself
one of the ID Placeholder producers listed in §6.1, so an earlier explicit
Get Usage Allocation request cannot establish the value for the omitted form.
The controlled peer checks the exact request fields, unique batch item IDs,
response correlation, success statuses, and returned identifiers. The C
consumer checks the typed Locate result and each typed allocation result.

Red/Green/Refactor evidence:

- Red commit `939b6aaf`: updated the failing route contract to require a
  preceding placeholder producer. The old two-item recipe failed its
  operation-order and fixture-shape assertions; its installed TLS exchange
  had previously been rejected locally with `InvalidInput` and `NotSent`.
- Green commit `13a885bf`: added the Locate producer and the explicit and
  omitted Get Usage Allocation items; updated the peer state model and added
  the actual TLS route test.
- Refactor: extracted the typed Locate-response check into a dedicated helper
  and recorded the normative behavior and verification here.

Verification on the current implementation head:

- Focused Get Usage Allocation route tests: 9/9 passed.
- The installed C consumer passed all 24 controlled TLS 1.3/mTLS route runs
  across KMIP 2.0 and 2.1, TTLV/JSON/XML, raw TLS/HTTPS, and the explicit and
  omitted identifier selectors. Every request sent the ordered Locate/Get
  Usage Allocation/Get Usage Allocation batch, so each run exercised both
  identifier forms and verified both typed and TTLV allocation responses
  against the Locate result.
- CTest: 7/7 passed, including route-matrix contracts, both controlled TLS
  peers, artifact checkout history, and the offline generic route.
- `cmake --build interop/c-kmip-validator/build --parallel 4` and
  `git diff --check` passed.

This closes the two optional-identifier recipe rows only. The C route matrix
and the remaining language validators still have outstanding routes; the PR
remains WIP until the full matrix and exact-head CI gates are green.
