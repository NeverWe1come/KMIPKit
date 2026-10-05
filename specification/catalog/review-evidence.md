# KMIP 2.1 inventory review evidence

This record documents the source review for the KMIPKIT-0002 inventory. The
four source work products are pinned in [`SOURCES.md`](../oasis/kmip-2.1/SOURCES.md)
and `CHECKSUMS.sha256`. The normative Specification and Profiles were reviewed
from their local pinned HTML; no page was fetched or edited.

## Source coverage and reconciliation

| Source | Review scope | Catalog evidence | Independent and executable checks |
|---|---|---|---|
| KMIP Specification 2.1 | §§1–14, operation and protocol-element tables | 877 normative-text candidate locators in 297 candidate-bearing sections; 57 client-to-server and 5 server-to-client operations; typed element records | Exact candidate/ledger comparison; table reconciliation tests for tags, objects, attributes, message fields, structures, credentials, enumerations, bitmasks, options, and results |
| KMIP Profiles 2.1 | §§1–6, including all profile and conformance clauses | 534 candidate locators in 144 candidate-bearing sections; 35 profile definitions paired with 35 clauses in §§6.1–6.35; 93 linked fixture references | Exact candidate/ledger comparison; independent profile and fixture cross-check; profile inventory tests |
| KMIP Test Cases 2.1 CN01 | §§2.1–2.110 | 110 section IDs and fixture labels, with raw links and display labels kept distinct | Exact section-set test; explicit operation and protocol-element links only where the pinned HTML establishes them; malformed label/link tests |
| KMIP Usage Guide 2.1 CN01 | Informative guidance and examples | Source is pinned as informative and creates no normative requirements | Source hierarchy and authority checks prevent informative text from creating normative obligations |

The generated [coverage report](coverage-report.md) lists candidate counts and
dispositions for each Specification or Profiles section containing a detected
normative-text candidate. `audit_sources.py --check` independently compares the
complete emitted locator set to the checked-in clause ledger; it reported
1,411 candidates with exact set equality. The section table is a review aid and
does not replace the full locator comparison.

The typed source tables reconcile to the acceptance counts in
`specs/002-normative-inventory/spec.md`. Test evidence contains 110 CN01 cases
and 93 Profiles references; all 203 linked XML fixtures are unavailable in the
pinned source tree. Their raw `href` values remain traceable and are not
followed. The independently checked, bidirectional crosswalk contains 76
operation associations across 65 test records and 123 protocol-element
associations across 47 test records (60 CN01 and 63 Profiles associations).
Together, 91 test records name at least one operation or protocol element.
Tests require exact case-to-element and element-to-case links, and explicitly
retain exclusions where a phrase is ambiguous. Neither operation nor element
links are inferred from case-ID prefixes or ordinary verbs.

The profile JSON sample in §5.5.4.1 labels Object Type value `Template`, while
Specification §11.34 labels wire value `0x00000006` reserved. This is recorded
as open `KMIPKIT-DISC-038`, affecting the JSON client/server profiles and the
reserved Object Type value. The test case links the Object Type tag shown in
the sample but does not treat `Template` as an assigned enum value. No
interpretation is selected; dependent implementation remains gated.

## Operation table reconciliation

An independent review of the pinned Specification parsed body captions (excluding
TOC entries) and operation headings in §§6.1–6.2. The 57 client and 5 server
operation headings match the catalog names and section references. The review
checked 186 operation-context table captions: all 117 request/response payload
table number and literal-caption pairs match the catalog (117/117); 59 error
tables and 10 operation-specific auxiliary tables were also inspected. The
only caption/context mismatches found were Tables 286 and 315, recorded as open
`KMIPKIT-DISC-039` and `KMIPKIT-DISC-040` without changing source captions.

The catalog schema records operation payload tables but has no separate
`error_tables` field. The 59 error tables were checked in source context;
result reasons, statuses, and normative clauses remain represented in the
catalog. The table-number relationship itself is not represented as a distinct
record in this inventory.

## Profile and discrepancy reconciliation

An independent profile review matched all 534 Profiles source IDs to the
pinned HTML inventory. It checked the 186 profile requirement keywords against
the source candidate ledger; 34 rows gained conditional requirements and 44
rows were reclassified with source-based rationales. Every remaining
conditional or applicable profile row has linked requirement evidence. The
catalog has 40 unique open discrepancies, no selected decisions, and no
resolved status without erratum or accepted-decision evidence. The generated
coverage report lists affected records as implementation gates.

Independent table review identified two further Specification source defects.
Under §6.1.41, Table 286 is printed as “PKCS#11 Response Payload” inside Query
Asynchronous Requests; it remains verbatim in the operation record and is
gated by open `KMIPKIT-DISC-039`. Under §6.1.48.1, Table 315 is printed as “RNG
Retrieve Errors” after the Re-Provision response table, while §6.1.49 contains
Table 318 with that caption; the source caption is retained and gated by open
`KMIPKIT-DISC-040`.

## Normative classification

Every detected clause has a stable ID, exact source keyword list, section,
structural locator, role, direction, scope, disposition, and either linked
requirement IDs or an exclusion rationale. All ten normative keywords defined
by the inventory contract appear in the ledger and are tested. The 567 records
preserve the exact keyword and canonical strength. The catalog deliberately
leaves feature, implementation, and verification assignments empty because
this specification creates the inventory, not the KMIP protocol behavior.
The 85 source-backed profile test-to-requirement links are now reciprocal and
cover 15 requirements. The remaining 552 requirements each record why no
requirement-specific official Test Cases ID is linked by the pinned sources;
the generated report lists each gap. All 203 cited XML fixtures remain
unavailable, as recorded by DISC-036 and each TestCase fixture state.

The source audit and section review preserve unresolved source wording as
discrepancies. The catalog contains 40 open discrepancy records and selects no
decision for them. Affected records are listed in the generated report as
implementation gates. No profile is claimed; all 35 claim states remain
`not_claimed`.

The independent Specification reconciliation counted 877 candidate locators
by top-level section: front matter 8; §§1–12 respectively 11, 16, 29, 163, 12,
475, 71, 20, 34, 6, 22, and 4; §13 has 0; §14 has 6. Exact locator-set
equality is 1,411/1,411 across Specification and Profiles. Section 13.1 was
read independently: it contains split-key algorithm prose but no candidate
keyword from the approved inventory contract and no table. The catalog records
the split-key method values, and does not invent a keyword-backed requirement
for the prose. Table-driven checks reconcile the element tables in §§2–5, 7–9,
and 11–12; §6.1–§6.2 table review is described above.

## Review limits

OASIS XML fixture bodies are not present in the pinned tree, so this review
confirms their source links and availability state but does not validate their
payload contents. Test Cases CN01 does not directly cross-reference Profiles
profile IDs; the inventory does not infer such a relationship. A later
conformance specification must obtain the fixtures and keep every open
discrepancy that affects its scope gated until resolution evidence exists.

## Final QA and execution verification

Independent QA rechecked the acceptance evidence and found no additional catalog blockers. Requirement evidence is reciprocal for 85 official case-to-requirement links across 15 requirements; the remaining 552 requirements carry explicit source-evidence gap notes and appear individually in the generated report. All 19 catalog-readiness criteria were checked against the source review, validation rules, generated report, and recorded reconciliation evidence. There are 40 open discrepancies with no selected interpretations, 35 profile claim states remain `not_claimed`, and all 203 cited XML fixtures are unavailable in the pinned source tree.

Fresh verification on 2026-10-04:

- `python -B -m unittest discover -s tools/normative_catalog/tests -v`: 130 passed, 6 skipped (Windows directory-symlink privilege unavailable).
- `python -B -m unittest discover -s scripts/tests -v`: 36 passed, 3 skipped (same Windows symlink limitation).
- `pwsh -NoProfile -File scripts/tests/Test-Wsl.ps1`: all 8 passed.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and `cargo test --workspace --all-features --locked`: passed with Windows stable and WSL Rust 1.94.0.
- `python -B tools/normative_catalog/validate.py`: 4 sources, 1,411 clauses, 4,018 records valid.
- `python -B tools/normative_catalog/audit_sources.py --repo-root . --base-sha ce34179cd8bf96812af53b5ec88daeae330fce35 --check`: 1,411 source candidates exactly match the ledger.
- `python -B tools/normative_catalog/check_immutable_sources.py --repo-root . --base-sha ce34179cd8bf96812af53b5ec88daeae330fce35`: pinned OASIS tree unchanged.
- `python -B tools/normative_catalog/report.py --check` and `git diff --check`: passed.
- Coverage preflight scanned all 7 Rust source files and found no production function bodies, so line-percentage gates are not applicable to this catalog-only feature. `cargo llvm-cov` is not installed in this environment; no coverage percentage is claimed.

The independent QA review and these command results complete T039. Security review (T040) and draft PR creation (T041) were complete at the time; PR #4 has since merged into `release/1.0.0`, and its later report-correction follow-up PR #12 is also merged.

## Security review and follow-up

A Codex Security diff scan (`a9259b9e-aafd-4b03-98bd-739fea2c3c4d`) covered `ce34179cd8bf96812af53b5ec88daeae330fce35..fdd430858781cc06f3a10b4f69bb20cd3eaeda6e`. It completed with zero reportable findings. It validated and suppressed two low-severity local availability candidates: blocking on a POSIX FIFO and unbounded capture of Git tree metadata. The normal CI path gates validation with the exact-base immutable-source check. Both cases were nevertheless hardened; the scan report and retained threat model are available in the Codex Security scan artifacts.

The follow-up used distinct Red, Green, and Refactor commits:

- Red `baf39a3`: three focused tests failed for the expected reasons: FIFO open exceeded the 2-second timeout; oversized Git output reached malformed-record parsing instead of a size-limit rejection; equivalent numeric sections rendered in input-dependent order.
- Green `fcc5e55`: POSIX reads use nonblocking flags before checking file type; Git tree output is capped at 16 MiB and 100,000 entries; numeric section sorting now has a stable text tie-breaker. All three focused regression tests passed.
- Refactor `2a0c558`: consolidated link-safe, nonblocking read flags in one helper; the three focused tests passed again.

The full Windows catalog suite then passed 130 tests with 6 platform-specific skips. The three POSIX regression tests also passed in WSL. The full WSL suite could not resolve this Windows worktree's `.git` pointer, so native Linux CI remains the full-suite Linux confirmation. T040 and T041 are complete; PR #4 has since merged into `release/1.0.0`, and the follow-up report-correction PR #12 is also merged.

The final Codex Security diff scan (a745d0fd-1e54-4d35-af9c-e919fa0f09d3) covered the exact PR range `ce34179cd8bf96812af53b5ec88daeae330fce35..957d277b403f27e323923fc52163796e5a45170f`. It completed with zero reportable findings. Three candidates were suppressed after validation: duplicate clause IDs are rejected by the following catalog validator; bare report URLs require a user click and have no automatic fetch path; and source immutability is gated against the exact PR base before catalog validation. The scan report and retained threat model are available in the Codex Security scan artifacts.

## Credential catalog follow-up

An independent review of the pinned Specification HTML at
`specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html` checked the
exact text and table entries in §§1.2, 9.4, and 9.11 (including Table 412).
Section 1.2 defines the uppercase key words, including `MUST`, as described by
RFC 2119. Section 9.4 prints “they must ALL be satisfied” with lowercase
`must`. The inventory retains the detected keyword candidate and records the
case-classification alternatives in open `KMIPKIT-DISC-041`; no interpretation
is selected. Requirement `KMIPKIT-REQ-SPEC-9.4-001-003` describes the server's
authentication process, remains `server_only` and `unassigned`, and has no
client feature, implementation, or verification assignment.

Section 9.11 says a Credential is used for client identification and that it
MAY be used for authentication as indicated by KMIP Profiles. The catalog
summary and condition now keep general identification separate from
profile-dependent authentication. Requirement `KMIPKIT-REQ-SPEC-9.11-001`
links to `KMIPKIT-ELEM-CREDENTIAL-CREDENTIAL` with a reciprocal element link.
The §9.4 Authentication and repeated Credential requirements likewise link to
their corresponding existing message fields in both directions.

The §9.11 Device paragraph names Device Serial Number, Network Identifier,
Machine Identifier, and Media Identifier for uniqueness, permits server
policies on individual-field uniqueness, permits a shared secret or password,
and says the client SHALL provide at least one field. Table 412 contains six
Device Credential fields: Device Serial Number, Password, Device Identifier,
Network Identifier, Machine Identifier, and Media Identifier. Open
`KMIPKIT-DISC-042` retains the alternatives that “at least one field” refers to
any of the six Table 412 fields or to one of the four fields named for
uniqueness. No minimum set is selected. Requirement
`KMIPKIT-REQ-SPEC-9.11-004-001` now summarizes the source-neutral “at least one
field” rule, its review note points to DISC-042, and it links to all six fields;
`KMIPKIT-REQ-SPEC-9.11-004-002` links only to the four named uniqueness
fields; and `KMIPKIT-REQ-SPEC-9.11-004-003` links to Device Password. No local
uniqueness-enforcement requirement was added.

Changed records are clauses `KMIPKIT-CLAUSE-SPEC-9.4-001` and
`KMIPKIT-CLAUSE-SPEC-9.11-001`; requirements `KMIPKIT-REQ-SPEC-9.4-001-001`,
`KMIPKIT-REQ-SPEC-9.4-001-002`, `KMIPKIT-REQ-SPEC-9.4-001-003`,
`KMIPKIT-REQ-SPEC-9.4-002`, `KMIPKIT-REQ-SPEC-9.11-001`, and
`KMIPKIT-REQ-SPEC-9.11-004-001` through `-003`; discrepancy records
`KMIPKIT-DISC-041` and `KMIPKIT-DISC-042`; and the reciprocal element links
listed above. The pinned OASIS source files and source manifest were not
edited. The exact-base audit found 1,411 candidates and the immutable-source
check confirmed the OASIS tree matches base
`849b46f772fd7cbcea2f42393cdbcf58e92cac94`.
