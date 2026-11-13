# KMIPKIT-0092 C route evidence implementation (WIP)

The approved KMIPKIT-0092 contract is consumed directly by
`interop/c-kmip-validator/tools/route_matrix.py`. It invokes the installed public
C ABI for exact local catalog values and the compiled C consumer for controlled
network recipes. Library sources are unchanged.

| Requirement | C evidence and remaining work |
|---|---|
| KMIPKIT-0092-FR-001, FR-006 | Shared matrix generator, installed header/symbol/constant probe, verified exact release commit and library digest, compiled-consumer provenance comparison. |
| KMIPKIT-0092-FR-002, FR-007, FR-009 | Per-cell evidence and explicit pending sidecar; duplicate protection; common schema and validator in CI. Incomplete recipes keep the gate failing. |
| KMIPKIT-0092-FR-003 | Exact assigned scalar values and reserved-tag rejection are checked locally. Broader semantic branch/model recipes remain pending. No absent encoder/send/high-level API is fabricated. |
| KMIPKIT-0092-FR-004 | Ordered, redacted tree digests are retained per cell. Cross-language digest normalization/parity is pending. |
| KMIPKIT-0092-FR-005 | Controlled peers parse all six encoding/transport pairs, check recipe fields, numeric ordered results, lifecycle transitions and deletion. Cryptographic, asynchronous, and remaining positive operation recipes are pending. |
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

### Register request and response routes

The Red commits `7cafc62a2`, `22fff7a51`, and `b42bdc902` added installed-
consumer checks for the missing Register recipe and its generic response-TTLV
accessor. The C consumer now registers an isolated certificate, checks the
typed Register response, checks that the generic response TTLV contains the
same Unique Identifier, and verifies independent Destroy cleanup. The
controlled peer validates both KMIP 2.0 and 2.1 over TTLV, JSON, and XML with
raw TLS and HTTPS; request and response bodies remain out of evidence.

Verification on the implementation worktree:

- CMake build and CTest: 7/7 passed.
- `python3 -m unittest interop.c-kmip-validator.tests.test_route_matrix -v`:
  75/75 passed.
- The complete generated C route matrix: 66,728 expected cells, 36,786
  invoked, 0 failed, and 29,942 still pending. All 80 `test-op-register`
  evidence cells were invoked; none remain pending. The completeness command
  correctly exits unsuccessfully while the other recipes remain incomplete.

This Register path is one executable operation recipe, not completion of the C
interoperability implementation. In the same matrix, the largest remaining
gaps include the Register/Encrypt/Decrypt batch (304 cells), asynchronous
Cancel outcomes, and multiple operation and object routes. The PR remains WIP.

### Refactor verification

The response-TTLV comparison now returns the typed status directly instead of
combining a boolean with an output status. After that refactor, the CMake build
and CTest passed 7/7, and the focused installed Register exchange passed 1/1.
