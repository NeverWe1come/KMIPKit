# KMIPKIT-0010 T002 — Message Extension crosswalk report

Date: 2026-10-15
Branch: feature/KMIPKIT-0010-profile-conformance-t001-t002
Workspace: C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit
Recorded BASE: 600d809289fbe813a059ea20f01f176fa51e6bdd
Final Refactor/evidence HEAD: this report is included in the DCO-signed Refactor/evidence commit; the full commit ID is recorded in the final handoff.

## Scope and source decision

The exact-head CI gate 38709213120 passed on BASE before this slice. The slice adds the canonical Message Extension mapping to the independent profile-closure fixture, adds the exact source-backed regression, and documents the source audit/evidence. It does not change catalog ownership, runtime behavior, generated output, or pinned upstream OASIS files.

The mapping uses element KMIPKIT-ELEM-MESSAGE-FIELD-9-13-MESSAGE-EXTENSION and source clauses KMIPKIT-CLAUSE-SPEC-8.3-004, KMIPKIT-CLAUSE-SPEC-8.6-004, KMIPKIT-CLAUSE-SPEC-9.13-001. It links client requirements KMIPKIT-REQ-SPEC-8.3-004 and KMIPKIT-REQ-SPEC-9.13-001-004/-005 only. §8.6 is response placement provenance, with no client requirement. The known catalog role/direction inconsistency is deferred to KMIPKIT-0002; it was not used to infer behavior. The three §9.13 shape candidates -001/-002/-003 remain unresolved and excluded. No Criticality Indicator default was invented. The parent condition stays null / profile_conditional / pending_t002_crosswalk; no T002 closure or conformance claim is made.

Pinned locators: Profiles §5.1.1 item 5 Message Extension lines 4594–4596; Specification §8.3 Table 396 row [42956,42974], §8.6 Table 399 row [43479,43503], §9.13 paragraph [44858,44872], Table 418 [44894,44944], and §11.56 tag entries Message Extension [59976,59986], Vendor Identification [60964,60974], Criticality Indicator [59416,59427], Vendor Extension [60950,60961]. The regression also checks seven mapped values and the exact seven unmapped values in source order.

## TDD commits and results

RED commit 22828166846b783deb5ecfb1c807b8c847a9ef7c (DCO signed):

Command:
py -3 -B -m unittest tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_data_structures_condition_has_exact_reviewed_batch_crosswalk tools.normative_catalog.tests.test_generate_profiles.ProfileMetadataGeneratorTests.test_baseline_message_extension_condition_has_exact_reviewed_client_crosswalk -v

Result: expected Red, 2 tests / 2 failures / 0 errors. The oracle lacked the Message Extension row, so the exact crosswalk regression failed on the missing reviewed mapping. No implementation code was present in this commit.

GREEN commit 3aab78dfdd5de142a39b73f3f9b474f537b55762 (DCO signed):

Command: same focused command above.

Result: PASS, 2 tests. Added only the reviewed oracle row; a source-span normalization assertion was corrected to account for whitespace before punctuation emitted by the pinned HTML fragment. The regression pins exact mapping IDs, locators, obligations, source behavior, shape/tag data, unresolved exclusions, source order, and parent status.

REFACTOR/evidence commit: recorded after creation below. It changes the research and implementation evidence documents, this report, and corrects earlier evidence wording that mislabeled region coverage as line coverage. No runtime refactor was warranted.

## Normative suite and validator

Command:
py -3 -B -m unittest discover -s tools/normative_catalog/tests -p 'test_*.py'

Result: PASS; 558 tests, 9 skipped, 314.878 seconds.

An invocation of the normal validator run concurrently with the suite exited 1. Complete emitted error line:
catalog validation failed: pinned OASIS source file is unavailable

That output contained no source_id and no path. It did not establish which source, if any, was unavailable. A single in-process diagnostic wrapped validate.safe_read_bytes using a temporary PowerShell here-string piped to py -3 -B -. The wrapper logged resolved paths and byte counts for successful reads and would have printed READ_FAIL with the relative path, resolved path, and exception if any read failed. Diagnostic result, exit 0:
READ_OK source_path='specification/oasis/kmip-2.1/SOURCES.md' resolved_path=C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\specification\oasis\kmip-2.1\SOURCES.md bytes=9864
READ_OK source_path='specification/oasis/kmip-2.1/CHECKSUMS.sha256' resolved_path=C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\specification\oasis\kmip-2.1\CHECKSUMS.sha256 bytes=403
READ_OK source_path='specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html' resolved_path=C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\specification\oasis\kmip-2.1\upstream\kmip-spec-v2.1-os.html bytes=3349715
READ_OK source_path='specification/oasis/kmip-2.1/upstream/kmip-profiles-v2.1-os.html' resolved_path=C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\specification\oasis\kmip-2.1\upstream\kmip-profiles-v2.1-os.html bytes=664753
READ_OK source_path='specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html' resolved_path=C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\specification\oasis\kmip-2.1\upstream\kmip-spec-v2.1-os.html bytes=3349715
READ_OK source_path='specification/oasis/kmip-2.1/upstream/kmip-testcases-v2.1-cn01.html' resolved_path=C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\specification\oasis\kmip-2.1\upstream\kmip-testcases-v2.1-cn01.html bytes=115695
READ_OK source_path='specification/oasis/kmip-2.1/upstream/kmip-ug-v2.1-cn01.html' resolved_path=C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\specification\oasis\kmip-2.1\upstream\kmip-ug-v2.1-cn01.html bytes=466864
Catalog valid: sources=4 clauses=1411 records=4117

This success logged all source paths that the validator read and did not reproduce a failed read or identify a root cause. The normal CLI was run once after the normative suite:

py -3 -B tools/normative_catalog/validate.py --repo-root .

Result: exit 0; Catalog valid: sources=4 clauses=1411 records=4117.

No validator source was changed.

## Remaining verification commands and results

1. py -3 -B tools/normative_catalog/generate_profiles.py --repo-root . --check
   PASS: Profile metadata verified: crates/kmipkit-protocol/src/profiles_generated.rs
2. py -3 -B tools/normative_catalog/report.py --repo-root . --check
   PASS: coverage report verified at specification/catalog/coverage-report.md
3. py -3 -B -m tools.normative_catalog.check_immutable_sources --repo-root . --base-sha 600d809289fbe813a059ea20f01f176fa51e6bdd
   PASS: pinned upstream OASIS sources match the exact base.
4. cargo fmt --all --check
   PASS; exit 0, no output.
5. cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
   PASS; exit 0. Non-fatal external AWS-LC compiler feature and NASM probe warnings appeared.
6. cargo test --workspace --all-features --locked
   PASS; exit 0 across workspace test targets and doc tests. Non-fatal AWS-LC/NASM and Windows linker warnings appeared.
7. cargo llvm-cov --workspace --all-features --locked --summary-only
   PASS; exit 0. Table totals: workspace regions 93.00%; workspace lines 95.39% (30,038/31,491). For kmipkit-ttlv/src/codec/decoder.rs, regions 91.99% (517/562), lines 96.90% (406/419).
8. PowerShell saved JSON command:
   $coverageSummaryPath = Join-Path $env:TEMP 'kmipkit-t002-llvm-cov-summary.json'; cargo llvm-cov report --json --summary-only --locked --output-path $coverageSummaryPath
   PASS; report saved to C:\Users\ramp1953\AppData\Local\Temp\kmipkit-t002-llvm-cov-summary.json. Summing lines for seven files under crates\kmipkit-ttlv yields 748/761 = 98.29%; regions yield 981/1,033 = 94.97%. The repository's stated gate is 95% line coverage, which this current TTLV crate result passes. The old T002 evidence/PR wording that treated 93.00% workspace regions and 91.99% decoder regions as line percentages was inaccurate; those are region metrics. No Rust production code changed.
9. git diff --check
   PASS; exit 0. Git emitted only CRLF-to-LF normalization warnings for the two edited evidence documents.

A syntax-only report attempt, cargo llvm-cov report --workspace --all-features --locked --summary-only, exited 1 with error: invalid option '--all-features' for subcommand 'report'. The JSON report command above is the valid report subcommand form. This was command usage, not a test or coverage failure.

## Scope and outstanding ambiguity

The intended changed files are tools/normative_catalog/tests/fixtures/profile-closure-v2.1.json, tools/normative_catalog/tests/test_generate_profiles.py, specs/010-profile-conformance/research.md, specs/010-profile-conformance/implementation-evidence.md, and this report. There is no implementation behavior or catalog change. The §8.6 role/direction mismatch remains deferred under KMIPKIT-0002; the initial validator unavailable-source error remains unreproduced with no source path/source_id in its output. These do not support an inferred client rule. The parent condition remains pending and the T002 crosswalk is not a conformance claim.


## Additional generated-report check result

The parallel post-documentation command py -3 -B tools/normative_catalog/report.py --repo-root . --check exited 1 and emitted exactly: coverage report failed: catalog or report input is invalid or unsafe. A serial rerun of the same command exited 0 and emitted: Coverage report verified: C:\Users\ramp1953\.codex\worktrees\0010-profile-refresh\KMIPKit\specification\catalog\coverage-report.md. The failure was not reproduced; no generator or validator code changed.
