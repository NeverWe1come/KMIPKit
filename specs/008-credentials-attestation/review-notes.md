# KMIPKIT-0008 Review Record

**Record status**: Sections through “Verification boundary” document the preliminary review of the 2026-10-05 revision. The 2026-10-06 release evidence refresh and final review section supersede their earlier gate/status statements.

**Date**: 2026-10-05

**Scope**: Specification, plan, tasks, requirements checklist, Rust contract, data model, and test scenarios. No implementation code was reviewed.

## Independent QA

The reviewer checked requirement coverage, OASIS references, decision gates, task sequencing, and cross-artifact consistency. Findings and resolutions:

1. **Secret-send gate could be satisfied without testing the candidate callsite.** KMIPKIT-0008 is now explicitly in-memory only and cannot add a production credential writer or send path in any gate state. A later client feature must own its candidate callsite and owner-through-transport lifecycle test.
2. **OD-006 was not represented consistently in the prior revision.** That revision listed all six decisions as unresolved and identified the OTP wording as `KMIPKIT-CLAUSE-SPEC-9.11-008`. This is separate from KMIPKIT-0007 OD-006, which governs future secret-bearing request lifecycle-test ownership.
3. **Final approval could follow a changed specification.** T001 was preliminary QA only; T004 requires independent review of the final specification revision and updated checklist after catalog and interface reconciliation.
4. **Missing fixture evidence omitted its catalog discrepancy.** Both the spec and research record cite `KMIPKIT-DISC-036` and classify the acceptance tests as derived, not official vectors.
5. **Two success criteria lacked explicit task references.** Round-trip preservation (SC-002) and the Attestation Capable Indicator (SC-004) are now directly mapped to test/implementation tasks.

The prior final QA re-review confirmed the OD-006 separation and absolute no-send boundary, with no remaining findings in its requested scope at that revision.

## Independent security review

The reviewer checked secret handling, zeroization limits, send-path ownership, source traceability, and the Device/timestamp ambiguities. Findings and resolutions:

1. **Transmission boundary was conditional.** The plan, test scenarios, requirements, and tasks now state that KMIPKIT-0008 never adds a production send path; any later path requires a separate feature and lifecycle test.
2. **Project policies were mapped to OASIS IDs.** FR-014 and T006 now separate normative OASIS traceability from project-policy provenance and tests.
3. **Zeroization scope was too broad.** The Rust contract and data model limit the guarantee to initialized KMIPKit-owned bytes before owner deallocation and identify spare capacity, pre-transfer reallocations, borrowed/stack/register copies, and caller/dependency/runtime copies as outside the guarantee unless cleanup is explicitly initialized and verified.
4. **One edge case implied approval could enable sending through KMIPKIT-0008.** The test scenario now verifies that no approval state enables a send path through this feature.

The final security re-review confirmed these corrections and reported no remaining findings in its requested scope.

## Task 5 disposition reconciliation (preliminary snapshot, 2026-10-05)

The independent source audit updated the decision register without approving the specification or changing any human-owned checklist marker:

- OD-001 remains open for catalog-owner review. The §9.4 lowercase “must” classification and `KMIPKIT-REQ-SPEC-9.11-001` summary/element links are tracked in a separate catalog workflow item; this feature task does not edit catalog input or generated output, and no client “all Credentials satisfied” test is assigned.
- OD-002 is resolved: typed Device values require at least one of the four identifier members named in §9.11 so the caller can satisfy the uniqueness SHALL; Password or Device Identifier alone is insufficient. Preserve all six fields and keep presence independent of text content. The caller owns actual uniqueness; the source comparison scope is unspecified and KMIPKit does not verify uniqueness from client-local data. Generic TTLV remains lossless for unvalidated trees.
- OD-003 remains open for Timestamp owner, comparison scope, and clock behavior. Caller Timestamp/hash bytes and effective SHA-256 default are explicit; no hash calculation or monotonicity check/test is specified before review.
- OD-004 is resolved by KMIPKIT-0008's permanent in-memory scope. Any future send path requires its own approved feature and candidate-callsite/owner-through-transport lifecycle evidence.
- OD-005 remains open only for execution integration, including inherited defaults, request/batch replacement, omission, precedence, and one Request Header Authentication applying to the whole batch. Standalone in-memory models do not require an execution API.
- OD-006 is resolved by the catalog's `informative_context` classification for `KMIPKIT-CLAUSE-SPEC-9.11-008`, which has no requirement ID. No library-wide OTP replay/single-use state or normative client enforcement is introduced; request-scoped use follows architecture. It remains distinct from KMIPKIT-0007 OD-006.

At this 2026-10-05 snapshot, T002–T004 and all implementation tasks remained unchecked. The release evidence refresh below supersedes this snapshot and records the updated dependency/decision state; final T004 review evidence is recorded separately below.

## Verification boundary

At this preliminary-review snapshot, these reviews covered design artifacts only. They did not close OD-001, approve ADR-0012, authorize implementation, or replace T004. No code tests were run because that review contained no implementation code.

## Release evidence refresh (2026-10-06)

The accepted release catalog now contains the correction tracked by OD-001.
KMIPKIT-0002 T045–T047 records the §9.4 `role=server`, `server_only`,
`unassigned` disposition, the corrected §9.11-001 summary and reciprocal
Credential link, the generated report, pinned-source review, and independent
parent QA. `KMIPKIT-DISC-041` remains open because the lowercase `must` force
has not been decided; no explicit catalog-owner sign-off is named. The 0008
specification and research were refreshed to match this evidence without
changing catalog inputs, generated output, or the immutable OASIS source.

The accepted 0005/0006/0007 dependencies and ADR-0012 were also checked against
the release and exact merged PR SHAs in `research.md`. The 0007 authentication
selection handoff does not exist; OD-005 remains limited to a future execution
integration and does not block standalone in-memory credential models. These
updates are coordinator evidence only. The requirements checklist remains
reviewer-owned and has not been checked or approved by this update.

## Independent QA findings disposition (2026-10-06 working revision)

The independent QA review identified two blocking issues and two wording issues:

- The Device Credential Value cannot be empty because §9.11 says the client SHALL provide at least one field. The spec now interprets “field” as one of the six Table 412 members, rejects an empty structure, and separates presence from text length and uniqueness. A second normative review confirmed this as the strongest literal reading: the four-field uniqueness rule does not narrow the separate minimum-presence rule. Uniqueness remains unverified and is not part of the cardinality check.
- The first QA report said indicator emission could not be implemented because the 0006 header view is read-only. A focused reassessment inspected the existing 0007 `build_request_message` path in `crates/kmipkit-client/src/execute.rs` and confirmed that 0008 can add the non-secret indicator there without adding a writer, permit, Authentication selection, Credential payload, or secret-bearing path. The spec, plan, and tasks now target this existing path, and a fake-transport capture must verify Authentication remains absent.
- Known members are validated by their OASIS table, while unknown children from an existing generic TTLV tree remain preserved and accessible without typed interpretation.


The QA re-review at `e0e6af1493d1988e0103728916482057860a85fb` found that accepting any single Table 412 field could treat Password or Device Identifier alone as satisfying a Credential that must also meet the separate identifier uniqueness SHALL. This revision requires at least one of the four named identifier members in typed Device values, preserves the two other fields as supplementary values, keeps actual uniqueness with the caller, and leaves comparison scope unspecified. It separately documents generic TTLV preservation for unvalidated trees. Request independent normative and QA re-reviews of this exact revision before marking T004 complete.

The independent normative disposition reviewed pinned OASIS §9.11 at `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html:44184-44189` (source hash `8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF`). It concluded that a typed Device value must conservatively contain at least one of the four identifiers named by the uniqueness SHALL; §9.11 does not state that exact presence rule verbatim. Keep actual uniqueness with the caller, comparison scope unspecified, and no inferred non-empty text rule. Preserve all six fields; Device Identifier's omission from the named uniqueness set is not reinterpreted. The final spec marks this as a conservative interpretation. Request a read-only QA re-review of the exact resulting revision before passing T004.
- T026 now asks for contract tests that execute the quickstart acceptance scenarios; the quickstart is not represented as executable code examples.

At this earlier readiness snapshot, the checklist remained unchecked and T004 was incomplete pending exact-revision review, the active-release rebase, and application of delegated authorization.

## Active release refresh (2026-10-06)

PR #44 for KMIPKIT-0009 merged into `release/1.0.0` as
`ae87b89d43957e4fc028e785dc181e69b0165dac`. The KMIPKIT-0008 feature branch
was rebased onto that exact commit; `git merge-base HEAD origin/release/1.0.0`
returns the same SHA. `git diff --check` passed after the readiness corrections.
The independent final QA review of this exact rebased revision was still pending at that point.

## Final requirements review and T004 disposition

Independent QA reviewed exact HEAD `7943d090c129482c022e1b9a5197ac2e3433aa3a`
against release base `ae87b89d43957e4fc028e785dc181e69b0165dac`. The reviewer
reported no substantive blockers and substantiated CHK001–CHK024. The review
confirmed the conservative Device identifier-presence rule, T022 test-module
registration, T025 FR-008 documentation coverage, and the unchanged
single-writer/no-credential-send boundary. No tests were run because the review
covered specification quality only.

The delegated authorization in `approval-record.md` was applied to this
revision. T004 is complete, the checklist is checked, and `spec.md` is approved
for implementation. The QA reviewer identified trailing whitespace on
`approval-record.md:3`; it was removed in the gate-record update. Run
`git diff --check origin/release/1.0.0...HEAD` passed after the cleanup in
commit `f1afc67910509603f4ec60906ca4655de1bbd818`. The independent security
design review is recorded below.

## Independent security design review

The independent security reviewer examined exact HEAD
`f1afc67910509603f4ec60906ca4655de1bbd818` against release base
`ae87b89d43957e4fc028e785dc181e69b0165dac` before implementation. Result:
PASS, with no blocking security design issue or required specification/task
change. The review confirmed in-memory credential scope, redaction and
zeroization constraints, the existing-writer/non-secret-indicator boundary,
caller-owned Device uniqueness, and no added production dependency. No tests
were run because this was a design review. Final implementation security review
remains T028 and the independent qualified review remains required before 1.0.

## Dependency and traceability gates (2026-10-06)

The independent dependency reviewer examined `dependency-review.md` and the
resolved QuickCheck 1.1.0 test graph at `bbc2d83`. The review found no need for
a new direct dependency or Cargo manifest/lockfile change. Its corrections are
recorded in that review: include target-specific `r-efi 6.0.0`, distinguish
QuickCheck from its existing `rand`/`getrandom` transitive dependencies, and
limit fixed-seed reproducibility claims because `SmallRng` is not portable
across platforms or releases. The reviewed graph stays test-only.

T006's CSV was checked against `specification/catalog/kmip-2.1.json`: all 11
catalog requirements assigned to KMIPKIT-0008 are present with their exact
`source_clause_ids` and requirement IDs; the existing 0006 default-indicator
requirement is separately mapped; server-only and deferred rows remain
classified; project policies cite KMIPKit sources, not OASIS. Nonce byte
preservation is a separate project-policy row rather than being attributed to
the Attestation Credential requirement. A 31-row uniqueness/schema/catalog
consistency check passed. Executable paths are assigned now and become
verified evidence only after their respective tests are implemented.

## User Story 1 RED — Authentication contract (2026-10-06)

Added `crates/kmipkit-protocol/tests/credential_contract.rs` with cases for an
absent header Authentication, rejection of a present-empty typed value,
ordered repeated Credential values, and client-side non-assertion of server
credential satisfaction. These are derived local-model cases, not official
OASIS test vectors. `cargo +1.94.0 fmt --all --check` passed. The focused RED
command
`cargo +1.94.0 test -p kmipkit-protocol --test credential_contract`
failed with `E0432` for the intentionally not-yet-implemented public imports
`Authentication` and `Credential`; no other compiler errors were reported.
This compile-level failure records the missing public API required by the test.
No production code has been added.

## User Story 1 RED — generic Credential round-trip (2026-10-06)

Added `crates/kmipkit-protocol/tests/credential_roundtrip.rs`. Fixed cases
cover all six assigned values and the Extensions range, an unknown raw type,
unknown children, and exact source-order/value preservation. A bounded,
seeded QuickCheck property varies unknown raw type bits and opaque payload
bytes; its custom `Debug` output reports only type bits and payload length.
`cargo +1.94.0 fmt --all --check` passed. The focused command
`cargo +1.94.0 test -p kmipkit-protocol --test credential_roundtrip` failed
with only `E0432` for the intentionally not-yet-implemented public `Credential`
API. No production code has been added.
