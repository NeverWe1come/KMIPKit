# KMIP 2.1 normative inventory

`kmip-2.1.json` is the reviewed source of truth for the KMIP 2.1 standards
inventory. It records the four pinned OASIS work products, structural source
clauses, protocol elements, tag ranges, requirements, project policies,
profiles, test evidence, discrepancies, and accepted decisions. Stable IDs and
explicit relationships make source coverage and later implementation
assignments auditable.

The independent source review and its reconciliation evidence are recorded in
[`review-evidence.md`](review-evidence.md). The catalog contract is documented in
[`specs/002-normative-inventory/data-model.md`](../../specs/002-normative-inventory/data-model.md)
and [`contracts/catalog-format.md`](../../specs/002-normative-inventory/contracts/catalog-format.md).
The Python standard-library validator checks the record shape, source hashes,
relationships, and bounded input rules. The source auditor reads only the
allowlisted local Specification and Profiles HTML after the immutable-base
check; neither tool downloads documents or follows catalog URLs or fixture
paths.

Scope states distinguish `client_1_0`, `client_1_1`, `server_only`,
`profile_conditional`, `out_of_scope`, `mixed`, and `unclear`. Requirement IDs
retain each exact OASIS keyword and map it to a canonical strength. Clause
dispositions account for every extracted normative-text candidate, including
informative context and reviewed source discrepancies. The catalog records
source-backed profile/test relationships and retains malformed labels and raw
fixture links separately from normalized display fields. An unavailable
fixture is evidence of a missing local artifact, not a failed conformance
result.

Profile records keep `direct_source_clause_ids` as reviewed closure roots and
store `source_clause_ids` and `requirement_ids` as their deterministic computed
closure. `normative_inclusions` records source-backed incorporation edges;
`included_section: null` means the entire pinned source document, while a
section value includes that section and its dot-delimited subclauses. The
validator rejects unresolved or duplicate edges, cycles, and stored profile
closures that differ from their roots and inclusions. Informative references
are not represented as normative inclusion edges. Baseline Client §6.1 item 1
includes the whole KMIP Specification v2.1 source, item 2 includes Profiles
§3.1, and item 3 includes Profiles §5.1.1. \`profile-targets.json\` selects the
product target independently; the generated report shows this beside
\`claim_state\`. Those catalog links and
target membership do not imply that the profile is runtime-eligible or that
conformance has been established.

The clause ledger records keyword-bearing candidate units. Some profile
sections also contain non-modal conformance conditions and enumerated values;
the independent profile-closure oracle records these in
`provenance.profile_condition_inventories` with source locators, the governing
profile requirement, and explicit mapping status. They do not acquire separate
RFC-style requirement IDs by inference. A pending condition mapping blocks a
complete profile conformance claim even when the keyword-clause closure is
current.

`coverage-report.md` is generated from the reviewed catalog by
`tools/normative_catalog/report.py`. Do not edit the report manually. Empty
coverage assignments remain visible as unassigned. Prohibited requirements
that need negative verification are listed explicitly. Open discrepancies show
their downstream gate and affected record counts; implementation specifications
must resolve a discrepancy before changing affected behavior. The report
includes pinned source checksums and count reconciliation, plus requirement-
level gaps where no official Test Cases ID is explicitly source-linked.

## Profile readiness and claim lifecycle

The catalog `claim_state` records evidence lifecycle; it is not a public
conformance claim. `profile-targets.json` records the selected project target,
while runtime validation availability is determined separately by complete
typed rules and category-appropriate executable tests. The generated report
shows these states and the mandatory fixture execution gaps independently. This feature
never publishes a profile claim.

Baseline Client is the selected initial 1.0 target and currently has
`claim_state: evidence_incomplete`. Its 16 mandatory official XML fixtures
are pinned, but remain unpassed until complete case execution evidence is
recorded. The typed-rule registry is not empty: it includes a
mapping for Wrapped Secret Data under KMIPKIT-REQ-SPEC-2.7-001. Baseline's
294-requirement closure now has typed mappings and category-appropriate
behavior tests, so runtime rule selection is available. Its catalog evidence
remains incomplete: T036 still records three blocked mandatory fixture replays
and 77 linked cases without local fixtures. Runtime selection does not
establish official test success or a profile-conformance claim.

Readiness may accept test results only from a strict, ephemeral manifest made
by the final aggregate job during a push to protected `release/1.0.0`. The
manifest must bind the current repository, commit, workflow, run and attempt,
all required successful job IDs, and a freshly verified branch policy with
required review, no bypass actors, admin enforcement, and force-push and
deletion disabled. Pull-request, scheduled, and local runs cannot certify
their own evidence. Missing or unverifiable branch-policy or app-token
evidence keeps readiness blocked. A passing evaluator never advances catalog
state automatically; evidence-complete updates require human review, and
public publication remains a separate release decision. The
[conformance strategy](../../docs/compliance/conformance.md) records the exact
workflow trigger, required job IDs, and branch-policy predicate.

See the [profile conformance guidance](../../docs/user-guide/en/profile-conformance.md)
and [KMIPKIT-0010 specification](../../specs/010-profile-conformance/spec.md)
for the evidence workflow. Inventory presence does not imply implementation,
profile support, certification, or conformance.

Tag reporting lists all five source-ordered ranges and named-tag counts by
OASIS allocation. Unassigned protocol elements expose their names, wire values,
and allocation states. Profile rows include clause, requirement, element,
dependency, test, transport, and encoding links; linked tests retain their
official case ID and mandatory/optional status when the source records it.

This inventory is evidence for future protocol specifications. It does not
replace the OASIS work products, amend their wording, resolve open source
discrepancies, or serve as the separate public API manifest used for repetitive
binding generation.
