# KMIP 2.1 normative inventory

`kmip-2.1.json` is the reviewed source of truth for the KMIP 2.1 standards
inventory. It records the four pinned OASIS work products, structural source
clauses, protocol elements, tag ranges, requirements, project policies,
profiles, test evidence, discrepancies, and accepted decisions. Stable IDs and
explicit relationships make source coverage and later implementation
assignments auditable.

The catalog contract is documented in
[`specs/002-normative-inventory/data-model.md`](../../specs/002-normative-inventory/data-model.md)
and [`contracts/catalog-format.md`](../../specs/002-normative-inventory/contracts/catalog-format.md).
The Python standard-library validator checks the record shape, source hashes,
relationships, and bounded input rules. The source auditor reads only the
allowlisted local Specification and Profiles HTML after the immutable-base
check; neither tool downloads documents or follows catalog URLs or fixture
paths.

`coverage-report.md` is generated from the reviewed catalog by
`tools/normative_catalog/report.py`. Do not edit the report manually. Empty
coverage assignments remain visible as unassigned; inventory presence does
not imply implementation, profile support, certification, or conformance.

This inventory is evidence for future protocol specifications. It does not
replace the OASIS work products, amend their wording, resolve open source
discrepancies, or serve as the separate public API manifest used for repetitive
binding generation.
