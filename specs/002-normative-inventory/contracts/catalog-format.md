# Catalog Format Contract

The normative inventory is a versioned JSON document. Its record fields and relationships are defined in [data-model.md](../data-model.md). The checked-in validator enforces this contract using Python's standard library; it does not depend on a JSON Schema package.

## Top-level document

The document contains `schema_version`, `sources`, `source_clauses`, `elements`, `tag_ranges`, `requirements`, `policies`, `profiles`, `test_cases`, `discrepancies`, and `decisions`. Unknown fields are rejected. Report sections have a fixed order; collections use these deterministic sort keys:

- `sources`: `source_id` (ASCII ordinal order).
- `source_clauses`: `source_id`, numeric section tuple, structural locator index, `clause_id`.
- `elements`, `requirements`, `profiles`, `discrepancies`, `decisions`, `policies`: stable ID (ASCII ordinal order).
- `tag_ranges`: source ID, source-order index, range ID.
- `test_cases`: source ID, official case ID (ASCII ordinal order), stable test ID.

No generation timestamp, absolute path, host name, locale-dependent text ordering, or machine-specific path is serialized. The report uses UTF-8, LF newlines, and fixed section ordering.

## Stable ID families

All IDs are at most 128 ASCII characters and must match their entire anchored pattern:

- `KMIPKIT-SRC-*`: `^KMIPKIT-SRC-[a-z0-9]+(?:-[a-z0-9]+)*$`
- `KMIPKIT-CLAUSE-*`: `^KMIPKIT-CLAUSE-(SPEC|PROF)-[0-9]+(?:\.[0-9]+)*-[0-9]{3}$`
- `KMIPKIT-ELEM-*`: `^KMIPKIT-ELEM-[A-Z0-9]+(?:-[A-Z0-9]+)*$`
- `KMIPKIT-RANGE-*`: `^KMIPKIT-RANGE-[0-9]{3}$`
- `KMIPKIT-REQ-*`: `^KMIPKIT-REQ-(SPEC|PROF)-[0-9]+(?:\.[0-9]+)*-[0-9]{3}$`
- `KMIPKIT-PROFILE-*`: `^KMIPKIT-PROFILE-[A-Z0-9]+(?:-[A-Z0-9]+)*$`
- `KMIPKIT-TEST-*`: `^KMIPKIT-TEST-[A-Z0-9]+(?:-[A-Z0-9]+)*$`
- `KMIPKIT-DISC-*`: `^KMIPKIT-DISC-[0-9]{3}$`
- `KMIPKIT-DEC-*`: `^KMIPKIT-DEC-[0-9]{3}$`
- `KMIPKIT-POLICY-*`: `^KMIPKIT-POLICY-[A-Z0-9]+(?:-[A-Z0-9]+)*$`

IDs are stable across ordering changes and are never reused after removal; removed records become deprecated references with a reason.

## Parsing limits and strictness

The validator first caps raw input at 16 MiB, validates UTF-8 and Unicode scalar values (rejecting unpaired escaped surrogates), and performs a bounded token preflight that rejects nesting deeper than 32, more than 100,000 entries across top-level record arrays, more than 1,000,000 JSON value tokens or object members globally, any string field over 65,536 UTF-8 bytes, and invalid JSON token structure before constructing the object graph. The preflight recognizes the exact top-level field allowlist and rejects any unknown top-level key before object construction. Only then may the standard-library JSON decoder build the bounded catalog; an object-pairs hook rejects duplicate keys and semantic validation rejects unknown nested fields and control characters in IDs, source sections, and citations. Allocation bounds do not rely on post-parse checks.

## Path and URL handling

`SourceDocument.local_path` must exactly match one of the four paths in the pinned `SOURCES.md` manifest. Canonical URLs must exactly match the corresponding four OASIS URLs. Catalog values are never used to construct arbitrary links or network requests. Fixture presence is determined from the pinned Git tree listing; validators never open a path supplied by a catalog record. Reject symlink and Windows reparse-point entries under the source tree.

A local fixture path is optional and must be a repository-relative POSIX path beneath `specification/oasis/kmip-2.1/`. Reject drive-qualified, absolute, UNC, `..`, backslash, symlink, or reparse-point paths. Do not open fixture paths; use the allowlisted pinned Git tree listing only. A raw OASIS fixture `href` is retained for traceability only and is never followed; absent fixture paths remain null with an explicit unavailable status.

## Source completeness audit

The offline source auditor reads only the four checksum-pinned Git blobs named by the allowlisted source manifest; it never opens a catalog-provided path and the parser has no network capability. Before parsing, a separate immutable-source gate compares the complete `specification/oasis/` tree to the exact pull-request base commit SHA and fails closed if that SHA is absent. Decode the explicit `meta` charset using a fixed codec allowlist (UTF-8 or Windows-1252); never use a host locale, fetch a `<base>` URL, or load external entities/resources. HTML input is limited to 8 MiB per document and 16 MiB total, nesting to 64 levels, parser events to 1,000,000 nodes, and extracted text to 16 MiB. Candidate detection uses the case-insensitive whole-word set `MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, and `OPTIONAL`, after concatenating inline text. Candidate units are paragraphs outside larger semantic blocks, list items including descendant text, table rows with descendant cells concatenated in source order, and definition-list terms/descriptions. Descendant paragraphs and cells are not separate candidates when an enclosing list item, row, or definition item is the source unit. Each candidate has its nearest source section, block kind, and depth-first ordinal; mutable rendered line numbers are not used. The ledger accounts for every emitted candidate exactly once with linked requirement IDs or a reviewed disposition. An independent section-by-section review checks extraction against every source section/table and records completion evidence; extractor/ledger equality alone does not prove source completeness.

## Validation behavior

Validation is offline and read-only with respect to the catalog. CI uses the pull-request event's exact base commit SHA, checks out/fetches that commit, and runs with read-only repository permissions and no secrets. The immutable-source comparison includes additions, removals, renames, modes, symlinks, and contents and runs before HTML parsing. Profile, discrepancy, decision, and policy records have exact field allowlists and checked state vocabularies; deviations and resolved discrepancies may refer only to accepted decisions with approver evidence. Validation exits nonzero for malformed JSON/HTML, limit violations, invalid record shape or field value, duplicate/unresolved IDs, candidate/ledger set differences, changed source checksums or base pins, count mismatches, unclassified source entries, unsupported requirement strengths, missing negative-test flags for prohibitions, deviations from SHOULD/SHOULD NOT/RECOMMENDED without accepted decisions, unsafe paths, or unassigned items omitted from the report. Open discrepancies are reported but do not fail the inventory solely because they remain unresolved; any implementation specification touching them is gated until a reviewed resolution exists.

## Safe report generation

Every catalog string is treated as untrusted. The Markdown renderer has separate encoders for table-cell text and link labels, escaping table delimiters, backticks, brackets, HTML metacharacters, backslashes, and line breaks for their specific contexts; all C0/C1 controls, including terminal escape, are emitted as visible escapes in report text, while identifiers and citations reject controls. Link destinations are only the four exact canonical URLs from the source allowlist. Catalog text, source sections, and raw `href` values never enter link destinations or fragments. Test cases cover parentheses, backslashes, line breaks, image/link syntax, HTML, and control-character payloads.

## Pinned-source immutability gate

CI compares every path under `specification/oasis/`—including source metadata and checksum manifests—with the protected pull-request base revision. Any difference fails, even when changed hashes are internally consistent. The feature validator separately checks the exact pinned hashes; checksum agreement alone does not authorize changing a pinned source.

## Report generation

The Markdown report is generated from the catalog and source-candidate audit. Repeated generation over identical inputs and generation after permutations of input collections yield byte-identical output. It includes totals by element kind, operation direction, requirement level, scope and direction, profile applicability/claim state, test-fixture availability, discrepancies, policies, source-ledger dispositions, unassigned requirements, and unassigned protocol elements.
