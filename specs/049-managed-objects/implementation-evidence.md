# KMIPKIT-0049 Implementation Evidence

## Source and branch baseline

- Authorized feature chain root: `feature/KMIPKIT-0043-t076-shared-client-mapping` (PR #104).
- Immediate predecessor: `feature/KMIPKIT-0048-typed-attributes`, PR #115, latest synchronized commit `ba01f43c45bfa851d3456713bcb840260bd332d8`.
- Current feature branch: `feature/KMIPKIT-0049-managed-objects`; the maintainer authorized the phase stack to start from #104 and to use predecessor branches until eventual integration into `release/1.0.0`.
- Pinned OASIS source: `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, SHA-256 `8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF`.
- Normative catalog: `specification/catalog/kmip-2.1.json`, SHA-256 `0D68D403956FE0BA4057BED346FDE57D1BC8938D975338E02BE2D435EC16FBEE`.
- Source check: `traceability.md` contains 53 unique operative requirement IDs; all 53 exist in the pinned catalog. The 9 structure tag associations were independently checked against catalog `wire_value` entries sourced from KMIP 2.1 §11.56.
- No clarification question remained: KMIP 2.1 tables resolve structure shape, ordering, tags, and conditional requirements; unresolved server behavior and out-of-scope capabilities remain explicit exclusions.

## Red — tests before implementation

The following tests were added before production code. Their compile failures are the expected Red state because KMIPKIT-0049 types and additive request helpers do not yet exist.

| Command | Result | Expected Red evidence |
|---|---|---|
| `cargo test --offline -p kmipkit-protocol --tests --no-run` | Failed as expected | `E0432` for missing managed-object, Key Block/Key Material, wrapping, and transparent key types; no implementation has been added yet. |
| `cargo test --offline -p kmipkit-client --no-run` | Failed as expected | `E0432` for missing typed models and `E0599` for the not-yet-added `RegisterRequest::from_managed_object` and `ImportRequest::from_managed_object`. |
| `cargo fmt --all --check` | Passed | All Red tests and current Rust workspace files are formatted. |

The Red tests cover all nine object tags and fields, the §3.1–§3.3 key structures, the nine transparent material structures, conversion preservation, validation negatives, redaction, and fake-transport Register/Import requests. The exact test output is retained in the task command history; implementation acceptance will be based on Green results, not on these expected compile failures.

## Development commits

- Specification/design: `686394817a34467bf4757fc9c7af7069df422575`.
- Normative order/tag correction: `d83cfec699ddf913258c37479f2bc78c2ca4e1d3`.
- Red — initial object/key structure contracts: `403466131c31c2b162eef7f88444cd65bfad8709`.
- Green — typed object/key structure implementation: `9f04e1fee39280f7ae2c7501a3495ad06f891266`.
- Red — parsed SplitKey/SecretData constraint rejection: `d27e322ac15cdc0a2e7859d330c36fc8e9f066c8`.
- Green — parsed object constraint validation: `8a7a3cb4886c18b4679dcef50ead2b30a0d884d6`.
- Red — wrapped Key Value key-information requirement: `351827162a644be11aa6fe65f2763c434414f6e1`.
- Green — enforce wrapping key information: `452751a1fdf814c1fa5e1d4181ee607374590ffc`.
- Red — reject malformed transparent key structures: `a78fd41f1fd2a94d41ed9505e3e90952dd59dda7`.
- Green — validate transparent formats and zeroize buffers: `d4d60044e1e6146b325da5bfeb545aa26515ae50`.
- Refactor — validate nested secret TTLV by reference, avoiding temporary clones: `56a6cbe58`.

## Local verification

- `cargo fmt --all --check`: passed.
- `cargo clippy --offline -p kmipkit-protocol --all-targets --all-features -- -D warnings`: passed.
- `cargo test --offline -p kmipkit-protocol --all-features --quiet`: passed; 1,273 unit tests plus all protocol integration and documentation tests.
- `cargo test --offline -p kmipkit-client --all-features --quiet`: passed; 635 unit tests plus client integration, UI/trybuild, and documentation tests. Windows emitted non-fatal `aws-lc-sys` compiler-feature and missing NASM warnings.
- `cargo test --offline -p kmipkit-protocol --test managed_object_examples --quiet`: passed; plaintext, wrapped, transparent, unknown-object, and secret-owner examples compile and run.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0049_managed_objects`: passed; 2 tests.
- `python -B tools/api_manifest/generate.py --check`: passed; all 12 generated outputs are current.
- Rust public-route audit: passed; 6,621 manifest declarations matched, 0 missing, 0 issues, and 15,748 exact exclusions.
- Binding public-route audit: passed with exit code 0.
- After synchronizing with #115 in merge commit `37bb75154`, the API manifest generator again verified all 12 outputs, and `python -B -m unittest tools.api_manifest.tests.test_reconcile_0049_managed_objects -v` passed both tests. `cargo clippy --offline --workspace --all-targets --all-features -- -D warnings` passed across the workspace. Windows emitted the same non-fatal AWS-LC `stdalign_check.c` and missing NASM warnings described above.
- After synchronizing with the #115 follow-up correction at `5c68f0006` in merge commit `a5d8c7b20`, `python -B -m unittest -v tools.api_manifest.tests.test_encrypt_decrypt_parity_manifest` passed all 5 tests, and `python -B tools/api_manifest/generate.py --check --repo-root .` verified all 12 generated outputs. The correction fixes the Decrypt helper attribution to KMIP 2.1 §6.1.11 and its generated Java parity fixture; it does not change the KMIPKIT-0049 runtime implementation.
- Windows-local coverage: `cargo llvm-cov --offline --workspace --all-features --json --output-path "$env:TEMP\kmipkit-0049-workspace-final-coverage.json" --quiet` completed successfully and ran the workspace tests. After repository source preflight and the exact documented-exclusion rules, the current report measures `kmipkit-protocol` at 21,067/21,891 (96.24%), the Rust workspace at 45,896/48,819 (94.01%), and changed executable production code at 772/806 (95.78%). The 25 new exclusions cover only impossible `ValueView`/TTLV variant fallthroughs after validated private model invariants; each has an adjacent source comment. The generated files excluded by repository policy remain subject to generator checks and consumer tests.
- T040's Windows-local protocol, changed-code, and global numeric thresholds pass. This is not the full cross-platform aggregate: reports from Linux/macOS and the C, Java, Python, and JNI coverage scopes are not part of this local run. The required hosted coverage and platform checks remain outstanding, so the PR must stay in draft.
- Independent QA identified malformed parsed SplitKey/SecretData and transparent RSA structures, missing wrapping key information, and documentation gaps. Red tests demonstrated the defects; the corresponding Green commits, expanded examples, and borrowed validation refactor remediate them. Independent security review also identified avoidable secret-bearing TTLV clones; these validation routes now borrow the source Item. No FFI, transport, or unsafe code changed.
- The Codex Security diff-scan launcher rejected its selected working-tree artifact as stale before creating a scan. No automated security scan is claimed. The independent static security review and local source review are recorded above; the automated review limitation remains explicit.
- Hosted cross-platform CI has not been used as evidence for this branch. The draft PR is intentionally not merge-ready while coverage and platform checks remain outstanding. No profile conformance, certification, or interoperability claim is made.

## Remaining evidence

The branch remains a draft pending hosted CI and the complete Linux/macOS plus
language-adapter coverage aggregate. These external platform results were not
queried during the GitHub billing issue. No independent human security review
or release-level interoperability evidence is claimed for this feature.

## Pull request

- T043 completed through terminal-based `gh pr create`.
- Draft PR: https://github.com/NeverWe1come/KMIPKit/pull/116
- Original PR-open head: `ed22113a260b4d01ad9163f2729890c47c85eece` (historical).

## Current PR metadata and verification basis (2026-11-07)

- Immediately before this documentation-only refresh, `git status` was clean;
  the local branch and `origin/feature/KMIPKIT-0049-managed-objects` both
  resolved to `28d209fd0a159376a8b7ca2b58c71704b37cc9a2`.
- `gh pr view 116` reported `OPEN`, draft, base
  `feature/KMIPKIT-0048-typed-attributes` at
  `a4fffdbac60563b0aeac5795902230df8e00039a`, and head
  `feature/KMIPKIT-0049-managed-objects` at
  `28d209fd0a159376a8b7ca2b58c71704b37cc9a2`.
- The Windows local verification above applies to the source/test tree at
  `0a5f661c9a7964825dab597d4472f8edee2cca69`, which is an ancestor of the
  audited head. The name-only diff from that source snapshot to the audited
  head contained only these documentation/evidence files:
  `specs/047-named-protocol-values/implementation-evidence.md`,
  `specs/047-named-protocol-values/tasks.md`,
  `specs/048-typed-attributes/implementation-evidence.md`, and this file.
  No implementation, test, or generated path changed after that verified
  source snapshot. This refresh also changes documentation only.
- Remote CI checks were not queried under the maintainer's billing instruction.

At that historical refresh, the local branch included the #115 follow-up
merge above; the remote #116 head had not yet advanced to that commit.

Hosted cross-platform coverage aggregation and CI remain outstanding; the PR is not merge-ready and the phase has not been declared complete.


## Reviewed predecessor synchronization (2026-11-07)

Merged the reviewed and published 0048 predecessor head
`ba01f43c45bfa851d3456713bcb840260bd332d8` from the existing 0049 head
`cbe09ce2fc34a042357f31533de23b00c877c956`. The merge retains the predecessor's
release-stack changes and its Red, Green, Refactor ownership-filter corrections.
All incoming manifest declarations, exclusions, capabilities, and other sections
are preserved exactly. All 356 0049 declarations and 566 0049 exclusions remain
identical to the original 0049 parent. The 0049 model/runtime source and original
source evidence are unchanged; four-language typed-object parity stays blocked
for KMIPKIT-0053.

The combined manifest contains 12,230 declarations and 17,384 approved
exclusions. T102 totals and the declaration/other-manifest fingerprints now pin
that exact combined inventory; the operation fingerprint and normalization
logic are unchanged. The official 0049 reconciler reports zero missing rows,
and the official generator produced all 12 outputs. No generated output was
edited by hand.

The official auditors generated current-source reports under
`specs/049-managed-objects/reports/`. Current-snapshot references in the shared
Rust auditor and inventory/schema tests now select this 0049 evidence. All 42
incoming historical 0047/0048 reports remain byte-identical to the predecessor.

Fresh verification against the final predecessor:

- `cargo test --offline --locked -p kmipkit-protocol --all-features --quiet`
  passed: 1,277 unit tests, all integration tests, and 3 doctests.
- `cargo fmt --all --check` passed.
- `cargo clippy --offline --locked -p kmipkit-protocol --all-targets --all-features -- -D warnings`
  passed.
- `python -B -m unittest tools.api_manifest.tests.test_reconcile_0048_typed_attributes tools.api_manifest.tests.test_reconcile_0049_managed_objects -q`
  passed all 16 tests, including all 14 0048 ownership-filter regressions.
- The combined API regression command below passed all 92 tests in 153.882
  seconds, with no failures, errors, or skips. It includes the original
  87-test selection and the five newest 0048 ownership-filter regressions.
- `python -B tools/api_manifest/generate.py --check` verified all 12 outputs.
- `python -B tools/api_manifest/reconcile_0049_managed_objects.py --report specs/049-managed-objects/reports/rust-export-report-post-sync-2026-11-07.json`
  reported zero added declarations and zero added exclusions.
- `python -B tools/api_manifest/audit_rust_exports.py --report specs/049-managed-objects/reports/rust-export-report-post-sync-2026-11-07.json --check`
  passed: 22,429 observed routes, 6,641 exact manifest matches, 15,788 exact
  exclusions, and zero missing, unobserved, kind, conditional, or scope issues.
- `python -B tools/api_manifest/audit_shared_rust_inventory.py --report specs/049-managed-objects/reports/shared-rust-inventory-post-sync-2026-11-07.json --check`
  passed. The paired inventory retains 3,697 mapped shared declarations.
- `python -B tools/api_manifest/audit_binding_exports.py --report specs/049-managed-objects/reports/binding-export-report-post-sync-2026-11-07.json --check`
  passed for C (1,708 exact matches), Java (1,881), and Python (2,000), with
  no missing routes, unobserved declarations, kind mismatches, or issues.
- `python -B tools/normative_catalog/check_immutable_sources.py --base-sha cbe09ce2fc34a042357f31533de23b00c877c956 --repo-root .`
  passed: pinned OASIS upstream sources match the original 0049 parent.

Exact combined API command:

```text
python -B -m unittest tools.api_manifest.tests.test_reconcile_0049_managed_objects tools.api_manifest.tests.test_reconcile_0048_typed_attributes tools.api_manifest.tests.test_t102_manifest_size_limit tools.api_manifest.tests.test_c_api_docs tools.api_manifest.tests.test_t102_response_view_mappings tools.api_manifest.tests.test_encrypt_decrypt_parity_manifest tools.api_manifest.tests.test_047_binding_audit_reconciliation tools.api_manifest.tests.test_t102_remaining_response_view_mappings tools.api_manifest.tests.test_parity tools.api_manifest.tests.test_catalog_parity tools.api_manifest.tests.test_t109_binding_export_audit_reconciliation tools.api_manifest.tests.test_shared_rust_inventory tools.api_manifest.tests.test_manifest_v2.PublicApiManifestV2SchemaTests.test_checked_in_manifest_inventories_all_rust_public_access_paths -q
```

The conflict-resolution paths pass the whitespace check. The complete merge
retains the predecessor's two existing whitespace diagnostics: the final blank
line in `interop/c-kmip-validator/src/c_validator.c` and trailing whitespace in
`interop/shared/vectors/generate_vectors.py`. Neither file is changed by the
resolution. Rust export auditing also emitted the established non-fatal Windows
AWS-LC `stdalign_check.c` and missing NASM warnings.

No PR was created, closed, or retargeted, and no remote CI status was queried.
Untracked files and existing target directories were preserved; target sizes
were not inspected. This synchronization adds no fresh coverage or platform
aggregate. The feature remains in draft pending its existing external review,
coverage, and cross-platform controls.


## T102 non-target snapshot review correction (2026-11-07)

Independent review reproduced two stale declaration fingerprints in the
Certify request-builder and optional request-structure modules. The earlier
92-test selection did not include these two modules. The extension-registry
namespace module passed unchanged.

Before updating either fingerprint, a semantic comparison against the reviewed
0048 parent `ba01f43c45bfa851d3456713bcb840260bd332d8` confirmed that the merge
adds exactly 356 managed-object declarations and removes or modifies no incoming
declaration. Both affected target subsets and all incoming operation rows are
unchanged. The fingerprint introduction history predates parts of the approved
0047/release-stack and typed-attribute reconciliation, so the drift is historical
snapshot coverage, not a new modification to the target mappings. Both operation
fingerprints already match and require no change.

### Red

The two existing snapshot tests now also assert that their normalized non-target
inventories retain all 286 typed-attribute and 356 typed-managed-object
declarations and leave every such route blocked for KMIPKIT-0053. This prevents
fixing the stale snapshots by dropping the stacked features or promoting parity.

`python -B -m unittest tools.api_manifest.tests.test_t102_certify_request_builder_mapping tools.api_manifest.tests.test_t102_client_extension_registry_namespace tools.api_manifest.tests.test_t102_optional_request_structure_mapping -q`
ran 14 tests with exactly two expected assertion failures and no errors: Certify
expected `06836a3a5acda28b85051dbfb9658beb6fb6c92ef7640b886bb8b7bcaa9455a9`
versus `3477efd6002b5732681fb9601016ba82f83e069608ff4c26ca6f4d93ac21fa9d`,
and optional structures expected
`c5853de988901f59c51ba64d80accc51a4b873161da1a75ef1c3315b7e4ae541` versus
`47183651eebdc81364d32a6155454357e30ecc65886d3acbfdd535831f2a516e`.
The new inventory/parity assertions passed before each old fingerprint failed.
No manifest, generated output, or production source changed in this Red step.


### Expanded Red: complete T102 inventory fixtures

After the initial Red commit `6df97ca68`, the complete command
`python -B -m unittest discover -s tools/api_manifest/tests -p 'test_t102_*.py' -q`
ran 399 tests in 132.496 seconds and exposed six failures plus one error beyond
the now-corrected two fingerprints. This is preserved as Red evidence before
updating those fixtures; no production validator or manifest is being changed.

Replaying the seven exact failing cases against the immutable reviewed 0048
manifest at `ba01f43c45bfa851d3456713bcb840260bd332d8` establishes provenance:

- Three cases pass on 0048 and fail after 0049: Import request traits,
  key-transfer request states, and Register request traits omit the new
  `from_managed_object` routes. The implementation in `import.rs` and
  `register.rs` derives Object Type from the supplied managed object; both
  exact manifest routes intentionally remain blocked for KMIPKIT-0053.
- Four cases already fail on 0048: two Locate expectations omit 23 approved
  `kmipkit::protocol` aliases (12 of them request surfaces); the profile error
  code fixture omits 20 aliases through the same public protocol namespace;
  and the response-conversion fixture omits the approved-exclusion argument
  supplied by the production generator entry point.
- The latter error concerns exactly one approved C function target,
  `kmipkit_client_batch_item_response_metadata_v1`, referenced by two ready
  mappings. It does not justify accepting an unmatched target. The corrected
  fixture must preserve explicit rejection when that exact approved target is
  removed, together with its existing four-language wrong-target tests.

The facade namespace is exported by
`pub use kmipkit_protocol as protocol` in `crates/kmipkit/src/lib.rs`.
Existing exact-ID, state, normative-requirement, and missing-target rejection
checks remain the intended contracts; only their stale fixture inputs change.


### Green

The exact declaration fingerprints are now
`3477efd6002b5732681fb9601016ba82f83e069608ff4c26ca6f4d93ac21fa9d` (Certify)
and `47183651eebdc81364d32a6155454357e30ecc65886d3acbfdd535831f2a516e`
(optional structures). Their target filters, normalization logic, and operation
fingerprints are unchanged.

The six expanded fixtures now pin the intended current inventory precisely:

- Import/Register expected-state sets include only their exact new
  `from_managed_object` methods as blocked routes. The tests also pin
  KMIPKIT-0053, the typed-object capability, FR-006/FR-007, and absent foreign
  mappings for these constructors.
- Locate pins all 74 operation declaration IDs, with an explicit 23-entry
  facade alias ID/name map and its 12 request aliases. Every alias must retain
  its canonical declaration's capability, state, requirements, and approval;
  server-only requirements remain rejected across the complete inventory.
- Profile diagnostic tests pin all 80 exact paths across the four approved
  public prefixes, including the 20 `kmipkit::protocol` aliases. Each namespace
  still checks exact language targets, diagnostic codes, owner requirements,
  and approval evidence.
- The response-conversion fixture passes the same approved exclusions as the
  production generator entry point. Removing the sole approved metadata C
  target still raises `not an exact listed export`; all four wrong execution
  mappings still raise their precise normalized-mapping diagnostic.

Verification:

- The three original reviewer modules passed all 14 tests after the fingerprint
  correction.
- The nine affected modules passed all 36 tests, with no failures/errors.
- `python -B -m unittest discover -s tools/api_manifest/tests -p 'test_t102_*.py' -q`
  passed all 399 tests in 144.441 seconds, with no failures, errors, or skips.
- `python -B tools/api_manifest/generate.py --check` verified all 12 outputs.
- `git diff --check` passed for this review correction.

Only test fixtures and this evidence file differ from the synchronized source
snapshot `5f8536a93f4e753d80ed8ac9544ed3e14399be17`. The production manifest,
validators, runtime, generated outputs, and audit reports remain unchanged.
No PR metadata or remote CI was queried or changed.
