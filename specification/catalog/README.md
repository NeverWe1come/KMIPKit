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

`coverage-report.md` is generated from the reviewed catalog by
`tools/normative_catalog/report.py`. Do not edit the report manually. Empty
coverage assignments remain visible as unassigned. Prohibited requirements
that need negative verification are listed explicitly. Open discrepancies show
their downstream gate and affected record counts; implementation specifications
must resolve a discrepancy before changing affected behavior. The report
includes pinned source checksums and count reconciliation, plus requirement-
level gaps where no official Test Cases ID is explicitly source-linked.
Inventory presence does not imply implementation, profile support,
certification, or conformance; all current profile claims remain `not_claimed`.

This inventory is evidence for future protocol specifications. It does not
replace the OASIS work products, amend their wording, resolve open source
discrepancies, or serve as the separate public API manifest used for repetitive
binding generation.
