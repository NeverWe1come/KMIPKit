# KMIPKIT-0092 C route evidence implementation (WIP)

The approved KMIPKIT-0092 contract is consumed directly by
`interop/c-kmip-validator/tools/route_matrix.py`. It invokes the installed public
C ABI for exact local catalog values and the compiled C consumer for controlled
network recipes. Library sources are unchanged.

| Requirement | C evidence and remaining work |
|---|---|
| KMIPKIT-0092-FR-001, FR-006 | Shared matrix generator, installed header/symbol/constant probe, exact `release/1.0.0` pin `489fafc79ef89db2524eafa7a6c6482f5cf75d97`, library digest, compiled-consumer provenance comparison. |
| KMIPKIT-0092-FR-002, FR-007, FR-009 | Per-cell evidence and explicit pending sidecar; duplicate protection; common schema and validator in CI. Incomplete recipes keep the gate failing. |
| KMIPKIT-0092-FR-003 | Exact assigned scalar values, reserved-tag rejection, typed attributes, and cryptographic-parameter field/TTLV round trips are checked locally. Broader semantic branch/model recipes remain pending. No absent encoder/send/high-level API is fabricated. |
| KMIPKIT-0092-FR-004 | Ordered, redacted tree digests are retained per cell. Cross-language digest normalization/parity is pending. |
| KMIPKIT-0092-FR-005 | Controlled peers parse all six encoding/transport pairs, check recipe fields, numeric ordered results, lifecycle transitions and deletion. Check, Get Attribute List, Log, Revoke, and Interop response TTLV accessors are read and structurally checked. The C peer covers catalogued asynchronous Cancel results, KMIP 2.1 Poll/Cancel and Process/Poll interleavings, and Query filters whose intersection returns only the matching outstanding request; other asynchronous and positive operation recipes remain pending. |
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

### Normative Date Time constructor route

The installed TTLV route now runs `kmipkit_ttlv_value_date_time` through the
same construction and decode-view path for the normative message Time Stamp
field. This covers KMIP 2.0 §9.20 Table 387 and KMIP 2.1 §9.20 Table 425.

- Red commit `0dd58eab`: the new route test failed because the Date Time
  constructor was not registered as an executable local route.
- Green commit `28d294f3`: registered the constructor for the existing
  catalog-driven generic TTLV roundtrip. The focused test passed and verified
  the actual installed ABI call and exact constructed/decoded bytes.
- Refactor: added the normative references and development evidence here.
- CTest passed 7/7, including the new route contract test.

The full installed C matrix and exact-head CI still determine the updated
pending-cell count; this route addition closes only the two timestamp cells.

### C Ping KMIP 2.0 version boundary

The installed C consumer now builds the typed Ping request under KMIP 2.0 and
confirms that KMIPKit rejects it before transport. KMIP 2.1 §6.1.36 defines
Ping; the catalog's KMIP 2.0 version-boundary case requires a not-sent result.
The controlled peer remains unconnected, and the route evidence records both
the Ping builder and typed batch execution call.

- Red commit `bba45156`: the new recipe test failed because
  `test-v20-reject-ping` had no executable C peer recipe.
- Green commit `4799d5dc`: added the boundary recipe, typed request path,
  not-sent matrix evidence, and controlled-consumer assertions.
- Refactor: derive the synthetic request operation value from its recipe and
  record this OASIS reference and verification evidence here.
- Focused Ping boundary tests: 2/2 passed; all six encoding/transport runs
  recorded both API calls and zero transport invocations for their send cells.
- CTest: 7/7 passed, including route-matrix contracts and the controlled
  KMIP 2.0/2.1 TLS peers.

This closes the 12 Ping boundary cells in the installed C matrix only. Other
C routes, cross-language parity, and exact-head CI remain outstanding.

### Generic C TTLV value constructors

The C local TTLV route matrix now executes every value constructor supported
by its recursive installed-ABI roundtrip harness, including Boolean, Long
Integer, Interval, Date-Time Extended, Big Integer, Text String, Byte String,
and Structure. Representative payloads roundtrip through public views for
KMIP 2.1 §7.2, §7.40, §8.1, and §9.10 targets. No network behavior is inferred
from these local constructor checks.

- Red commit `40202cfe`: the coverage assertion failed because the matrix
  admitted only Integer, Enumeration, and Date-Time value constructors.
- Green commit `5955ff9c`: catalogued the remaining constructors already
  implemented by `TreeAbi`; representative byte-string, text-string,
  big-integer, and structure payloads roundtripped through the installed C ABI.
- Refactor: record the normative target sections and exact matrix evidence.
- Focused generic TTLV tests: 3/3 passed.
- The full installed C matrix reported 37,841 invoked cells, 0 failed cells,
  and 28,887 pending cells. A cell-identity comparison with CI head
  `83e405f6` confirms the increase consists of 424 generic value-constructor
  routes, 18 Modify Attribute routes integrated from the remote branch, and
  12 KMIP 2.0 Ping boundary routes.

This adds local generic-TTLV coverage only; it does not discharge the remaining
typed operation and transport cells or the cross-language matrix requirements.

### Nested C TTLV Structures

Generic Structure fixtures now include required child Items selected from the
pinned normative catalog. The installed ABI route therefore executes both
`kmipkit_ttlv_structure_with_item` and `kmipkit_ttlv_value_structure` for
Asynchronous Request (§7.2), Request Header (§8.1), object structures, and
other nested structures with catalogued required members.

- Red commit `19c13405`: the tests showed that both Asynchronous Request and
  Request Header fixtures were empty and did not invoke `structure_with_item`.
- Green commit `ad2e9818`: recursively built fixtures from each Structure's
  required catalog members and verified exact decode/encode roundtrips.
- Refactor: record the source hierarchy and full-matrix evidence here.
- Focused nested-structure test passed; the full installed C matrix reported
  37,926 invoked cells, 0 failed cells, and 28,802 pending cells. Comparing
  cell identities with the preceding run shows 85 new structure routes.

### Delegated Login controlled-peer ordering

After the Delegated Login response route was integrated, the controlled peer
rejected the first KMIP 2.0 and 2.1 exchange because its expected operation
sequence still began with the older catalog cases. The fixture now expects
operation 47 and `test-op-delegated-login` first for both versions, then
validates the response's typed and generic Ticket fields. This follows OASIS
KMIP 2.0 §6.1.12 Table 185 and KMIP 2.1 §6.1.12 Table 200.

- Red commit `0d4964a0`: the peer-plan test failed because there was no
  Delegated Login entry in the expected sequence; CTest reproduced the peer's
  `unexpected operation batch (47,)` failure for both protocol versions.
- Green commit `799289e3`: prepended the request and result case to the
  controlled-peer plan. Both focused TLS peer tests passed.
- Refactor: preserve the incident, normative references, and matrix counts.
- Full CTest passed 7/7. The full installed C matrix reported 37,966 invoked
  cells, 0 failed cells, and 28,762 pending cells; the 40 newly invoked
  Delegated Login cells cover construction, execution, response decoding, and
  typed and TTLV response accessors across both protocol versions.

These validator changes close only their C route cells; remaining C routes,
other language matrices, and exact-head CI are still required.

### Query response field route evidence

The controlled C Query exchanges now credit the three typed response routes
against the exact Query response field target in each KMIP version. The peer
checks that the response payload belongs to the Query operation and contains
the catalogued field before recording the route. This follows KMIP 2.0
§6.1.37 Table 260 and KMIP 2.1 §6.1.40 Table 283.

- Red commit `c1c90e1b`: the new assertion failed because the typed accessors
  were observed only against the Query operation target, while the exact
  response-field target cells remained pending.
- Green commit `f8d69ff8`: validate response payload fields against the
  matching response batch item and emit the exact target cell after successful
  typed decoding.
- Refactor: share request/response payload-field matching and select the
  direction-specific payload tag explicitly.
- The focused Query response test passed. The C route-matrix suite passed
  109/109 tests, and CTest passed 7/7 tests.
- The full installed C matrix reports 66,728 expected cells, 38,014 invoked,
  0 failed, and 28,714 pending. Comparing to 37,966 invoked cells immediately
  before this change confirms six additional cells: three response accessors
  for each protocol version. The asynchronous outcome response-payload
  accessor remains pending for these cases.

This evidence covers the typed Query response views in the C validator only;
other C routes, language parity, and exact-head CI remain outstanding.

### Combined matrix after registered-key batch integration

The controlled peer now also validates the registered-key
Register/Activate/Encrypt/Decrypt/Revoke/Destroy batch. This is a read/write
fixture workflow with fixed public identifiers and checked batch ordering; no
request body or key material is retained in evidence.

- The peer regression Red commit `d3a276ed`: the focused KMIP 2.0/2.1 tests
  failed because the six-operation recipe was not recognized. Green commit
  `150964b0`: require each item identifier exactly once and return matching
  response identifiers. The earlier installed batch recipe commits are
  recorded in `specs/092-interop-api-route-exhaustiveness/tasks.md`.
- After merging the concurrent batch change and rebuilding the C consumer,
  the full route matrix reports 66,728 expected cells, 38,318 invoked,
  0 failed, and 28,410 pending. The batch case has no pending cells; the
  increase from 38,014 is 304 executed batch routes.
- The merged branch passed the C route-matrix suite (111/111) and CTest
  (7/7), including both KMIP 2.0 and 2.1 controlled TLS peers.

The C route matrix remains incomplete and exact-head CI is still required;
this measurement does not establish full API parity or external KMS support.

### Current release artifact and response-matrix verification

After KMIPKIT-0120 merged into `release/1.0.0`, the C validator now installs
the current shared artifact pin `489fafc79ef89db2524eafa7a6c6482f5cf75d97`.
Red commit `93c512fcf` demonstrated that the C pin no longer matched the
shared route manifest; Green commit `240b81ca3` aligned the installer, CMake,
CI checkout, and documented commands with the regenerated common pin.

- Installed package SHA-256: `7ec610437a0700aefa5163c6bd6e268b383bae2c32749bbdd9311134eb4dbe41`.
- Installed shared library SHA-256: `0cc7800552ecae4c0f0ba019e64a32a933302b85fe72af05e116c93dc56ed14c`.
- Artifact pin contract tests: 6/6 passed. CTest: 7/7 passed, including the
  KMIP 2.0 and 2.1 controlled TLS peers.
- Full matrix on the current artifact: 38,318/66,728 cells invoked, 0 failed,
  and 28,410 pending validator recipes. The completeness gate therefore remains
  red. In particular, the 104 STOP-on-first-failure cells remain pending; this
  C run does not claim to exercise the partial-response behavior added by
  KMIPKIT-0120.
- Refactor/documentation commit records the current installed artifact and
  keeps earlier matrix snapshots tied to their original source pins.

The current pin and local fixture results do not establish interoperability
with any deployed KMS; exact-head CI and the remaining route recipes are still
required.

### Batch Stop partial Get response

The C validator now sends two typed Get items with Batch Error Continuation
Option `Stop`: the first identifier is missing and the second exists. The
controlled peer returns only the first response item with
`OperationFailed/ObjectNotFound`, verifies that the second item was not
executed, and confirms that peer state did not change. The consumer checks the
response count, item identifier, operation and status, then exercises metadata
v1/v2, the typed Get view, raw payload absence, header TTLV, and absent pending
correlation. This follows OASIS KMIP v2.0 §§9.6, 11.5, and 6.1.19 and KMIP
v2.1 §§9.6, 11.5, and 6.1.19.

- Red commits `509a1e99a` and `46faba903` made the controlled TLS peer and
  route model require the partial Stop response; the peer timed out and the
  route tests failed before implementation. Red commit `0dc631407` also
  required successful use of each applicable C response accessor and explicit
  evidence for the Stop option.
- Green commit `319fa7c0f` implements the C request, validates the partial
  response and calls the failure-safe typed and generic response accessors.
  The fixture peer stops its state machine at the first failed item.
- The 114 C route-matrix contract tests passed, and CTest passed 7/7. The Stop
  case passed 120 of 132 generated route cells with zero failures over KMIP
  2.0/2.1, TTLV/JSON/XML, and raw TLS/HTTPS. The remaining 12 cells are
  inapplicable to this response shape: ten associate response accessors with
  the request Batch Item (§8.3), and two address a second response item that
  the server correctly omits. The current matrix still counts these as pending
  because the shared applicability model does not yet encode these exclusions.
- The full installed C matrix reports 38,533/66,728 invoked, zero failed, and
  28,195 pending validator-recipe cells. The completeness gate remains red.

This fixture validates the KMIPKit client against a controlled KMIP peer; it
does not establish behavior of an external KMS. The shared route-applicability
gap and all other pending C routes remain open.

### Ticket-backed Logout

The Logout live recipe now obtains a server-issued Ticket from a successful
Login with Request Count 2, then sends Logout with that exact Ticket and checks
the typed response and empty Response Payload. It follows OASIS KMIP 2.0
§§6.1.29–6.1.30 and KMIP 2.1 §§6.1.30–6.1.31. The Ticket is kept in memory
only and is never recorded in diagnostics, fixture reports, or matrix evidence.

- Red commit `140c066dd` required the Ticket-backed recipe. The focused matrix
  test first failed because Logout had no executable recipe. A separate
  regression then required the matrix to attribute route evidence to the
  second exchange, rather than crediting the prerequisite Login.
- Red commit `596338725` corrected the Login Request Count tag to the OASIS
  `Request Count` tag `0x42014C`; the controlled peer now rejects any other
  tag. Red commit `825d9e551` protects the Logout exchange index.
- Green commit `6d471c07a` implements the in-memory Ticket handoff and verifies
  both operation responses. The controlled peer passed all six KMIP 2.1
  encoding/transport combinations: TTLV, JSON, and XML over raw TLS and HTTPS.
- Verification: CMake build passed and CTest passed 7/7, including controlled
  KMIP 2.0 and 2.1 TLS fixtures. The KMIP 2.0 fixture confirms that the Logout
  path is explicitly not run without a valid Login Ticket.
- The complete C matrix reports 38,429 of 66,728 cells invoked, zero failed,
  and 28,299 pending. Logout accounts for 23 passed route cells and 19 pending
  KMIP 2.0 cells. The installed C Login builder only accepts KMIP 2.1, so a
  valid KMIP 2.0 server-issued Ticket cannot be acquired; this is an API
  limitation and no invalid or synthetic Ticket is sent to a server.

This recipe verifies KMIPKit against a controlled peer only. External KMS
compatibility and the remaining C route recipes are still unverified.

### Normative Item Type values

The C local matrix now routes normative `Item Type` enumeration cases through
the installed TTLV value constructors and decoder using their catalogued
parent identity. The generated per-case matrix previously left those values
pending because their parent is a TTLV type enumeration, not a KMIP tag-backed
enumeration. This follows OASIS KMIP 2.0 and 2.1 §11.23.

- Red commit `634ca89e3` added a regression using the normative Big Integer
  `Item Type` case. It failed because the value executor accepted only
  hand-built enum recipes and could not consume the catalog target.
- Green commit `05947cd69` derives the Item Type recipe from the target's
  normative parent and dispatches those cells through the existing full
  Item Type and TTLV boundary checks.
- Verification: the focused regression passed; all 122 C route-matrix tests
  passed. The full installed matrix improved from 38,467 to 38,533 invoked
  cells with zero failures; pending cells decreased from 28,261 to 28,195.
  The completeness gate remains incomplete and the exhaustive-recipes task
  stays open.

### Ephemeral message field through TTLV and typed parsing

The shared normative catalog now records the `Ephemeral` field's Boolean
encoding from OASIS KMIP 2.0 §8.3/Table 358 and KMIP 2.1 §8.3/Table 396. The C
fixture embeds both Boolean values in a valid Get Request Message, preserving
the eight-octet encoding from §10.1.2. The installed C API constructs and
decodes each generic TTLV tree, then parses it with
`kmipkit_request_message_from_ttlv` and reads back the original typed message
tree. The CI route runner now recognizes that model parser as an executable
local route.

- Red commits `85eaf12cf` and `dc8a0103c` showed the missing contextual
  Request Message recipe and that the matrix omitted the public parser symbol
  from its local execution set.
- Green commit `3c05d47a7` adds the versioned Get fixture, exercises false and
  true Boolean branches, and validates both the generic TTLV and typed parser
  routes without recording message bodies.
- The shared source-catalog correction is in the contract PR #180; it records
  `source_encoding: Boolean` and regenerates the shared test catalog.
- Verification: focused Ephemeral coverage passed for KMIP 2.0 and 2.1; all
  123 C route-matrix tests passed; CTest passed 7/7. The complete matrix now
  reports 38,539 of 66,728 cells invoked, zero failed, and 28,189 pending.
  Exhaustive coverage and exact-head CI remain incomplete, so the PR stays WIP.

### Get request payload through the typed C client

The C route manifest now maps the normative Get Request payload examples for
KMIP 2.0 and 2.1 to the installed typed `kmipkit_client_batch_item_get`
operation. The controlled peer receives these requests across TTLV, JSON, and
XML over raw TLS and HTTPS. This executes the catalogued payload recipes for
both protocol versions without retaining request bodies in the report. The
cases follow OASIS KMIP 2.0 §6.1.19/Table 205 and KMIP 2.1 §6.1.19/Table 220.

- Red commit `d1ccec5e2` added a regression requiring all twelve version and
  encoding/transport cells across the two cases to execute; it failed with no
  invoked cells before the route was implemented.
- Green commit `10d86288c` maps both case IDs to the existing typed Get recipe,
  includes their local-model peer conversions, and enables the Get workflow in
  the C driver.
- Verification: the focused route-matrix regression passed. The full matrix
  reports 38,601 of 66,728 expected cells invoked, zero failed, and 28,127
  pending. The two cases have no pending cells (62 cells newly invoked).
  CTest passed 7/7 after this addition; the focused regression was also rerun. Matrix completeness and exact-head CI remain outstanding, so the PR
  stays WIP.

### Interop Begin, End, and Reset

The installed C validator executes the KMIP Interop operation as one ordered
three-item batch: Begin, End, then Reset. Each item has a unique Batch Item ID,
as required for multi-item batches; the peer checks the ordered function values
and one shared identifier without recording the identifier. It returns three
successful responses with matching item IDs and empty Response Payloads. The C
consumer checks operation and status metadata for every item and exercises the
generic Interop response-TTLV accessor for the complete item shape. The peer
also models the transient test state using only a SHA-256 identifier key and
reports only the public transition labels. This follows OASIS KMIP 2.0
§6.1.25/Table 223 and KMIP 2.1 §6.1.26/Tables 241–243.

- Red commits `4337c0254`, `abaface48`, `150afe86e`, and `f885a79b2` added
  executable-recipe, ordered-fields, state-transition, and unique batch-ID
  regressions. The missing-recipe and missing-ID checks were observed failing.
- Green commit `c4cf549c3` implements the typed C batch, response checks, and
  controlled peer behavior. The first live attempt exposed the KMIPKit rule
  that every item in a multi-item batch needs a unique Batch Item ID; the
  request and response fixtures now preserve and check those IDs.
- Verification: CMake build passed; the three focused Interop tests passed;
  the controlled peer passed all 12 version/encoding/transport combinations
  (KMIP 2.0 and 2.1; TTLV, JSON, XML; raw TLS and HTTPS); CTest passed 7/7,
  including the full C route-contract suite and the KMIP 2.0/2.1 TLS fixtures.
- The full C route-matrix completeness gate remains incomplete. These results
  demonstrate the installed C consumer against a controlled peer only; no
  external KMS compatibility is asserted. Interop's test-state mutation must
  be classified and gated by the shared catalog update in PR #180.

### Get Attribute List optional Unique Identifier

The validator exercises the KMIP 2.0 and 2.1 Get Attribute List Request
payload through both C construction routes: the direct batch-item builder and
the typed request builder followed by `get_attribute_list_from_request`. The
controlled peer checks the present Unique Identifier against the created
fixture object and verifies it is present only on the Get Attribute List item.
The installed-package C test roundtrips both field branches—identifier present
and omitted—through the typed payload encoder and invokes both batch-item
constructors for each branch. Live runs without a configured fixture
identifier omit the field so KMIP's batch ID Placeholder resolves from the
preceding Create operation. This follows OASIS KMIP 2.0 §6.1.21/Table 211 and
KMIP 2.1 §6.1.21/Table 226.

- Red commits `b6b8845dc` and `a38831798` exposed the missing route recipe and
  the absent request-field-presence assertion.
- Green commit `5ff230c63` maps both cases to the controlled peer, supports a
  fixture UID in the request, verifies the payload-field route, and reports
  only per-item field-presence booleans. It adds installed C API checks for
  both optional branches without recording identifiers or KMIP bodies.
- Verification: all 24 controlled version/encoding/transport/constructor
  checks passed; CTest passed 7/7. The full matrix reports 38,729 of 66,728
  cells invoked, zero failed, and 27,999 pending. Both cases have no pending
  matrix cells. External KMS compatibility and remaining matrix recipes remain
  outstanding; PR #170 stays WIP.

### Typed Request Message parser: Request Header fields

The installed `kmipkit_request_message_from_ttlv` parser now executes the
catalogued Request Message root tag and root field cases using complete valid
Get requests. The request fixtures also exercise Maximum Response Size,
Asynchronous Indicator, Attestation Capable Indicator, Attestation Type,
Client Correlation Value, Server Correlation Value, and Time Stamp in Request
Header for KMIP 2.0 and 2.1. Batch Error Continuation Option and Batch Order
Option use two batch items with distinct Unique Batch Item IDs and a matching
Batch Count, as required by the parser's message-shape validation. These local
checks follow OASIS KMIP 2.0 and 2.1 §8.2; the shared route matrix maps the
Request Message parser to the applicable §8.1–§8.3 fields.

- Red commits `1c2b72a7d`, `023facdcf`, and `243ce4f9c` added the missing
  installed-parser assertions. They failed because the generic fixture was an
  isolated field item rather than a complete Request Message.
- Green commits `8d1b6cfbf`, `c2979ff24`, and `c75f9d3fc` construct valid
  complete request trees, including the two-item batch required for batch
  options, and compare the parser's read-back TTLV tree with the input.
- Refactor commits `dbcda97f8`, `43e30cb86`, and `a1a6d6436` share ordered
  Request Header insertion, isolate exact fixture value encoding, and unify
  contextual Header-field dispatch.
- Verification: focused parser tests passed for both KMIP versions; the full
  C route-matrix contract suite passed 133/133; CTest passed 7/7. The complete
  installed-artifact route matrix records 39,291/66,736 invoked cells, zero
  failed, and 27,445 pending. `validate_route_evidence.py` correctly fails on
  the remaining missing cells, beginning with the async Cancel response
  payload routes. CI run `43537438013` on head `ec21efe7531d6e74a849fbd72ab6991e9c92113e`
  recorded 39,273 invoked, zero failed, and 27,463 pending, all classified as
  `validator_recipe_pending`; it had no `fixture_preparation_failed` cells.
  The route matrix and completeness gate remain open, and PR #170 remains WIP.

### Typed Request Message parser: structural fields

The same installed parser fixture now covers the Request Header and Batch Item
structures, Protocol Version, Batch Count, Operation, Request Payload, and
Unique Batch Item ID. For the ID case the complete Get request includes the
optional byte-string ID in its defined Batch Item position. These cases follow
OASIS KMIP 2.0/2.1 §§8.1–8.3 and invoke the public parser on the complete tree,
not an isolated structure or scalar.

- Red commit `ddace71a3` required all nine structural targets to be present in
  complete request trees and observed the existing generic fixtures fail for
  both KMIP versions.
- Green commit `e555569e1` dispatches those targets through the complete Get
  request fixture and adds the optional ID in the correct position.
- Refactor commit `8972387ad` shares one builder for single- and multi-item
  batch identifiers while keeping the matching Batch Count for multi-item
  options.
- Verification: the nine-target installed-parser test passed for both
  versions; the full C route-matrix contract suite passed 134/134; CTest passed
  7/7. The complete installed-artifact matrix records 39,313/66,736 invoked
  cells, zero failed, and 27,423 pending. The shared completeness validator
  still fails on missing routes, starting with async Cancel response payload
  cells. This recipe slice does not complete the matrix; PR #170 remains WIP.

### Typed Response Message parser: complete and optional fields

The installed `kmipkit_response_message_from_ttlv` parser now receives a full
Get Response for the Response Message root and its structural fields. Complete
response fixtures also exercise Attestation Type, Client and Server
Correlation Value, Nonce with Nonce ID and Nonce Value, Server Hashed
Password, Unique Batch Item ID, Result Message, and Message Extension for KMIP
2.0 and 2.1. The Result Message case uses `Operation Failed` with `General
Failure` and omits Response Payload: OASIS KMIP 2.0/2.1 §8.6 Table 399 allows
the Failure shape without a Response Payload, and §9.17 defines Result
Message. Response Header fields follow §8.5 Table 398; §9.14 defines the
Nonce structure. Message Extension is in §8.6 Table 399 and §9.13; Result
Message is in §8.6 Table 399 and §9.17. The typed parser reads back each full
TTLV tree.

- Red commits `8991e2de0`, `c1a868f96`, and `502a8c2d6` required the complete
  parser routes, response context fields, and optional response fields. The
  Result Message test exposed a fixture that incorrectly combined the field
  with `Success` and Response Payload.
- Green commits `e73b38bf6`, `f311827ec`, and `dac861e51` use the installed
  response parser for those routes and construct a valid Failure/Reason/Message
  shape for Result Message.
- Refactor commits `57664357a`, `e5e5c5132`, and `6db3a4f12` share the typed
  request/response roundtrip, response fixture dispatch, and Response Header
  builder.
- Verification: the C route-matrix unit suite passed 139/139; CTest passed
  7/7. The complete installed-artifact matrix records 39,373/66,736 invoked
  cells, zero failed, and 27,363 pending, all `validator_recipe_pending`.
  `validate_route_evidence.py` exits unsuccessfully on those missing cells,
  beginning with async Cancel response payload routes, as required by FR-009.
  CI runs `43539660499` and `43539654754` on head `5adcf0c` likewise had only
  pending recipe cells (27,423) and zero failed cells. The complete matrix is
  still open, and PR #170 remains WIP.

### Async Cancel local payload routes

The local C recipe invokes `kmipkit_cancel_request_create` and
`kmipkit_cancel_request_to_ttlv_payload`, reads the request's borrowed
Asynchronous Correlation Value back, and compares the encoded payload to the
same bytes. It covers the Cancel operation target and the versioned KMIP 2.0
and 2.1 request payload targets under both protocol versions. For each of the
five catalogued Cancellation Result cases, the generic TTLV route constructs
and decodes the exact Cancellation Result enum value for both versioned
response payload targets and both protocol versions. These local routes follow
OASIS KMIP 2.0 §6.1.5 Tables 161–162 and KMIP 2.1 §6.1.5 Tables 176–177.

- Red commits `6b3a0dd5a` and `c5b091aca` require the typed request constructor,
  payload encoder, and per-case Cancellation Result roundtrips; the
  unimplemented routes failed locally.
- Green commit `3d8f7d4d4` executes those routes through the installed C ABI
  and catalog-assigned enum values.
- Refactor commit `37057e327` isolates the case-specific response payload
  fixture from the generic TTLV fixture dispatcher.

The installed C network recipe now issues an asynchronous Create, reads the
server-issued correlation value from the Pending response, sends Cancel for
that exact value, and checks the typed Cancel response against each case's
catalogued Cancellation Result. The controlled peer returns all five assigned
results independently for KMIP 2.0 and 2.1. The network matrix exercises the
five cases across TTLV, JSON, and XML over raw TLS and HTTPS (60 checks), in
addition to the direct Cancel recipe. This evidence follows OASIS KMIP 2.0
§§8.2–8.3 and §6.1.5 Tables 161–162, and KMIP 2.1 §§8.2–8.3 and §6.1.5 Tables
176–177.

- Red commit `49e408a6b` requires the controlled peer to return each
  catalogued result and exposed failures for Unable to Cancel, Completed,
  Failed, and Unavailable when the C driver assumed Canceled.
- Green commit `f27a9e056` loads the scenario's result name from the catalog,
  compares the typed response with its assigned enum value, and exercises all
  five responses on the controlled peer.
- Refactor commit `eb27e57d4` table-drives the named Cancellation Result
  mapping.
- Verification: the focused matrix test passed for all 60 network cells;
  CTest passed 7/7, including the KMIP 2.0 and 2.1 controlled TLS peers. The
  installed-artifact matrix records 39,989/66,736 invoked cells, zero failed,
  and 26,747 pending, all `validator_recipe_pending`. The required evidence
  validator exits unsuccessfully on the remaining Create seed constructor,
  send, response-decoder, and local Create-response accessor cells. The C
  completeness gate and PR #170 remain WIP.

### C asynchronous Poll and Cancel interleaving

The C installed-ABI route seeds an asynchronous Create and reads the
server-issued correlation value from its Pending response. It constructs and
inspects typed Poll and Cancel request payloads with that exact value, sends
Poll first, and verifies the Poll response is Pending with no Response Payload.
It then sends Cancel and verifies the echoed correlation and Canceled result.
The controlled peer records exactly three requests (Create, Poll, Cancel) and
the response statuses Pending, Pending, Success, proving one terminal result
in this deterministic interleaving. This does not simulate simultaneous
threads. The recipe runs on KMIP 2.1 over TTLV, JSON, and XML with raw TLS and
HTTPS. It follows OASIS KMIP 2.1 §§6.1.5, 6.1.38, 8.2–8.3, 8.6, and 9.1–9.2.

- Red commit `632e64300` required the Poll-before-Cancel peer route and six
  passing network cells; it failed because the C route recipe was absent.
- Green commit `a2f8da437` adds the C request construction, Poll and Cancel
  execution, response checks, and deterministic peer fixture.
- Refactor commit `0e34f770c` shares asynchronous seed authentication and
  generalizes correlation-payload validation.
- A follow-up matrix merge regression was captured by Red commit `b4fce5944`:
  the exact validator rejected duplicate local Cancel route cells emitted by
  both the offline runner and network peer. Green commit `99f23c5e8` passes
  already-covered local cells into the peer runner to avoid duplicate receipts.
- Refactor commit `b44ebf347` documents that the peer runner's returned key
  set includes both reused local receipts and newly observed network cells.
- Verification: the focused race test passed for all six network cells and
  both controlled TLS CTest peers passed. The full C matrix recorded 40,250
  invoked cells, zero failed cells, and 26,486 pending cells, all
  `validator_recipe_pending`. The matching completeness validator now reaches
  the missing-route check without duplicate-cell errors and exits unsuccessfully
  because the overall matrix remains incomplete. PR #170 remains WIP.

### C mandatory asynchronous Create

The installed C consumer now constructs Create with
`Asynchronous Indicator=Mandatory` and reads the Pending result and exact
server-issued correlation through the public response accessors. The peer
checks all six encoding/transport pairs for KMIP 2.0 and 2.1. The Pending
Response Batch Item includes its required empty Response Payload structure;
the no-payload Pending exception is specific to Poll. This follows OASIS KMIP
2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, and 11.3, Create §6.1.8 Table 171 (2.0) and
Table 186 (2.1), and Response Batch Item §8.6 Table 361 (2.0) and Table 399
(2.1).

- Red commit `bbab6bb04` failed because no C peer recipe existed for the
  mandatory asynchronous Create case.
- Green commit `01a3ea25d` adds the typed installed-ABI route and its controlled
  peer response check.
- Refactor commits `4551ce1ec` and `f1eccd192` share the payload-shape predicate
  and generalize the asynchronous Create seed check.
- Verification: the Mandatory Create route passed 12/12 network cells. The
  five Cancel-result route cases, Poll-before-Cancel case, Cancel response
  payload roundtrip, and both controlled TLS CTest peers passed. `cmake
  --build interop/c-kmip-validator/build --parallel 2` and Python syntax checks
  passed.
- The matching full C route matrix recorded 40,336 invoked cells, zero failed,
  and 26,400 pending, all `validator_recipe_pending`. The matching completeness
  validator still fails on missing routes, beginning with
  `test-async-indicator-omitted-default`; PR #170 remains WIP.

### C omitted asynchronous indicator default

The installed C consumer now constructs Create without setting the optional
Asynchronous Indicator. For KMIP 2.0 and 2.1 over all six supported
encoding/transport pairs, the peer verifies omission, synchronous Success,
the Create response payload, and absence of an Asynchronous Correlation Value.
This tests the default-Prohibited behavior in OASIS KMIP 2.0/2.1 §§8.2–8.3,
8.6, 9.1–9.2, and 11.3; Create is §6.1.8 Table 171 (2.0) and Table 186
(2.1).

- Red commit `3ba8b1a5a` failed because the C route registry had no recipe for
  `test-async-indicator-omitted-default`.
- Green commit `c22343d5f` adds the typed installed-ABI route and peer checks.
  The peer omits Unique Batch Item ID for this request, matching the request's
  absent ID and allowing the client to associate the synchronous response.
- Refactor commit `cdd9f7d45` shares Create request setup and response
  assertions with the Mandatory asynchronous route.
- Verification: the focused omitted-default route passed all 12 network
  cells; the Mandatory Create route, five Cancel-result routes, Poll-before-
  Cancel route, Cancel response payload roundtrip, C build, Python syntax
  checks, and both controlled TLS CTest peers passed.
- The matching full C route matrix recorded 40,422 invoked cells, zero failed,
  and 26,314 `validator_recipe_pending` cells. The exact completeness
  validator exits unsuccessfully, next reporting
  `test-async-indicator-optional-completes-or-pends`. The gate remains open and
  PR #170 remains WIP.

### C Optional asynchronous Create completion

The installed C consumer now sets `Asynchronous Indicator=Optional` and
handles the synchronous-completion branch of Create. The controlled peer
verifies the Optional value, successful Create response, absence of a
correlation value, and that no Poll is sent after completion. This exercises
KMIP 2.0/2.1 §§8.2–8.3, 8.6, 9.1–9.2, and 11.3; Create is §6.1.8 Table 171
(2.0) and Table 186 (2.1). The catalog also permits a Pending branch, which
remains separate coverage.

- Red commit `7eda42010` failed because the C controlled peer had no recipe for
  `test-async-indicator-optional-completes-or-pends`.
- Green commit `8d6e93641` adds the typed C route and peer checks across KMIP
  2.0/2.1 and all six encoding/transport pairs. The peer returns synchronous
  Success with no correlation and records one exchange.
- Refactor commit `8ab2db743` gives the synchronous Create response assertion a
  shared name for the omitted and Optional indicator cases.
- Verification: all three focused async Create route tests passed; the CMake
  build, Python syntax checks, `test_c_validator`, and both controlled TLS
  CTest peers passed. Temporary-file based tests used a worktree-local
  `TMPDIR` after the shared `/tmp` filled during parallel runs.
- The matching full C route matrix recorded 40,508 invoked cells, zero failed,
  and 26,228 `validator_recipe_pending` cells. The exact completeness
  validator exits unsuccessfully, next reporting
  `test-async-indicator-prohibited-synchronous`. The gate remains open and PR
  #170 remains WIP.

### C explicit Prohibited asynchronous Create

The installed C consumer now sets `Asynchronous Indicator=Prohibited` and
requires Create to complete synchronously. The controlled peer verifies the
exact indicator, successful Create response, absence of correlation, and no
Poll request across KMIP 2.0/2.1 and all six encoding/transport pairs. This
follows §§8.2–8.3, 8.6, 9.1–9.2, and 11.3; Create is §6.1.8 Table 171 (2.0)
and Table 186 (2.1).

- Red commit `7472db396` failed because the C controlled peer had no recipe for
  `test-async-indicator-prohibited-synchronous`.
- Green commit `f204ff0c7` adds the typed C route and verifies synchronous
  Success with no correlation for the explicit Prohibited value.
- Refactor commit `161155dbd` shares synchronous Async Indicator request,
  response, and peer-evidence checks across the Optional and Prohibited cases.
- Verification: the four focused async Create routes passed together; the C
  build, Python syntax checks, `test_c_validator`, and both controlled TLS
  CTest peers passed.
- The matching full C route matrix recorded 40,594 invoked cells, zero failed,
  and 26,142 `validator_recipe_pending` cells. The exact completeness
  validator exits unsuccessfully, next reporting `test-async-lifecycle-poll`.
  The gate remains open and PR #170 remains WIP.

### C asynchronous Create, Query, and Poll lifecycle

The installed C consumer now creates a key asynchronously, queries for the
request using its exact server-issued correlation value, polls that same
correlation to terminal Success, validates the returned Create payload, and
destroys the created key with authenticated cleanup. The peer verifies the
Create → Query → Poll → Destroy order, exact correlation filtering, terminal
response shape, and restored fixture state. This follows OASIS KMIP 2.1
§§6.1.8, 6.1.11, 6.1.38, 6.1.41, 8.2–8.3, 8.6, 9.1–9.2, and 11.3.

- Red commit `5a1e59d84` failed because the C route matrix had no implementation
  for `test-async-lifecycle-poll`.
- Green commit `9ee886593` implements the typed Query/Poll lifecycle and checks
  the correlation and terminal Create result through the installed ABI.
- Refactor commit `02ae1ba91` consolidates controlled-peer state and exchange
  checks shared by the asynchronous lifecycle routes.
- Verification: six focused asynchronous route tests passed; the CMake build,
  Python syntax checks, and three CTest cases (`test_c_validator` and both
  controlled TLS peers) passed. The full route matrix recorded 40,655/66,736
  invoked cells, zero failed, and 26,081 `validator_recipe_pending` cells.
  The installed C artifact SHA-256 is
  `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`. The
  completeness validator still fails, next reporting
  `test-async-no-recursive-async-poll-cancel`; PR #170 remains WIP.

### C asynchronous Process then Poll state

OASIS KMIP 2.1 §6.1.39 says Process changes the handling of the correlated
asynchronous request so the next Poll SHALL NOT return Pending. Tables 278–279
define the required correlation in the Process request and an empty Process
response payload. The installed C consumer now sends Create(Pending) →
Process(correlation, Asynchronous Indicator=Prohibited) → Poll(correlation)
→ authenticated Destroy. It checks Process Success and its empty payload, then
checks that Poll returns the original Create Success payload. The controlled
peer checks the correlation and order, records Process as synchronous, and
verifies that final fixture state matches setup.

- Red commit `de923f2f`: the focused route test failed because the C driver had
  no isolated Process case; the consumer reported that Create variations were
  not mapped and the peer received no request.
- Green commit `3e0d4a0f`: added the installed-ABI Process/Poll route, TTLV
  construction checks, response assertions, controlled network recipe across
  the six KMIP 2.1 encoding/transport pairs, and authenticated cleanup.
- Green fixture correction `bb4aca1d`: added the route to the broad TLS peer
  and reset its pending state after Process/Poll so the subsequent independent
  Poll/Cancel route remains isolated.
- Refactor/documentation commit: records the normative scope, traceability,
  and exact verification evidence here and in the KMIPKIT-0092 task record.
- Verification: the focused route test passed; `ctest --test-dir
  interop/c-kmip-validator/build --output-on-failure` passed 7/7. The installed
  C matrix recorded 40,931/66,736 invoked cells, all 40,931 passed, and 25,805
  `validator_recipe_pending` cells. Its library SHA-256 is
  `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`.
  The completeness validator correctly remains failing at the next missing
  recipe, `test-async-query-outstanding-filter`; PR #170 remains WIP.

### C asynchronous Query outstanding filters

OASIS KMIP 2.1 §6.1.41 Tables 285–287 defines the optional operation and
correlation filters and the returned asynchronous-request details; §7.2
Table 353 defines the returned `Asynchronous Request` structure. The installed
C consumer starts two asynchronous Creates and one asynchronous Get, polls the
first Create to completion, then sends Query with the Create operation filter
and all three correlations. It verifies that only the second, still-pending
Create is returned, excluding both the completed Create and the pending Get.
It polls the remaining Create to success, polls the Get to its terminal
failure, and destroys both created keys. Pending Create responses are checked
through the typed Create response accessor and must expose no identifier while
preserving their server-issued correlations.

- Red commit `62e393db`: the route test failed because the C driver had no
  Query outstanding-filter recipe.
- Green commit `31fb6298`: added the installed-ABI lifecycle, Query filter
  construction and inspection, typed Query response checks, controlled peer
  state assertions, and cleanup across the six KMIP 2.1 encoding/transport
  pairs.
- Supplemental Red commit `8b250b0d`: the focused route test failed because
  the Query scenario did not call the typed Create response accessor for its
  Pending responses.
- Supplemental Green commit `42a26232`: the route now uses
  `read_pending_create_correlation()` for both Pending Create responses; that
  helper calls the typed accessor and checks the absent Create payload and
  exact correlation.
- Verification: the focused Query test passed, including six controlled peer
  checks; CTest passed 7/7. The full installed C matrix recorded
  40,971/66,736 invoked cells, all 40,971 passed, and 25,765 pending cells,
  all `validator_recipe_pending`. The artifact SHA-256 is
  `c93eb8f7daa60d3b6832d297887e99cd2101e4f9baf2d04f55ce4042ba4bb7a2`.
  The matching completeness validator now reports
  `test-async-unknown-correlation-rejected` as the next missing recipe; PR
  #170 remains WIP.
