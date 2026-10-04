# Normative Inventory Data Model

## SourceDocument

| Field | Rule |
|---|---|
| `source_id` | Stable slug for one pinned work product. |
| `title`, `version`, `stage`, `publication_date` | Exact identity from the local source inventory. |
| `local_path`, `canonical_url`, `sha256` | Exact allowlisted repository path, publisher URL, and lowercase SHA-256 from `SOURCES.md` and `CHECKSUMS.sha256`. Paths/URLs are metadata; no arbitrary URL is fetched. |
| `authority_class` | `primary_normative`, `profile_normative`, `test_evidence`, or `informative`. |

## SourceClause

A clause-ledger entry is one source unit containing a detected normative-text candidate in the pinned Specification or Profiles HTML. It records `clause_id`, `source_id`, section, stable structural locator (source section plus depth-first ordinal and block kind), exact detected source keywords, normalized strength, manual review disposition, linked requirement IDs, and an exclusion rationale when the unit does not create an applicable client requirement. Candidate keywords are the case-insensitive whole-word set `MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, and `OPTIONAL`; inline text is concatenated before matching. `REQUIRED`, `MUST`, and `SHALL` map to mandatory strength; `MUST NOT` and `SHALL NOT` map to prohibited; `RECOMMENDED` and `SHOULD` map to recommended; `SHOULD NOT` maps to discouraged; and `MAY`/`OPTIONAL` map to canonical permission-or-optional strength while their source keywords remain distinct. Candidate units are paragraphs outside larger semantic blocks, list items including descendant text, table rows with descendant cells concatenated in source order, and definition-list terms/descriptions. Descendant paragraphs or cells are not separately counted when their enclosing list item, row, or definition item is a candidate. A reviewer separately checks every section and table against the extracted locator set; ledger/extractor equality alone is not accepted as proof of completeness.

Dispositions are `requirement`, `profile_conditional`, `server_only`, `later_1_1`, `non_applicable`, `informative_context`, or `source_discrepancy`. Every candidate must have one reviewed disposition; normative obligations must link to one or more requirements.

## ProtocolElement

| Field | Rule |
|---|---|
| `element_id` | Stable ID, unique across the catalog. |
| `kind` | `operation`, `message_field`, `structure_member`, `credential`, `data_type`, `object_type`, `object_structure`, `attribute`, `attribute_structure`, `operation_structure`, `tag`, `enumeration`, `enumeration_value`, `bitmask`, `bitmask_value`, `option`, `result`, or `extension_rule`. |
| `name`, `source_refs` | Canonical KMIP name and one or more exact document/section references. |
| `wire_value`, `allocation` | Optional string-preserving wire value; tag allocation is `assigned`, `reserved`, or `unused`. Values inside extension ranges are not falsely assigned as named OASIS tags. |
| `direction`, `scope_state` | Direction classification and 1.0/1.1/profile/out-of-scope disposition, each with a reason where it is not directly stated by the source. |
| `payload_tables`, `asynchronous_response` | Operation-only traceability. `payload_tables` retains each source table's role, number, and printed caption exactly; an empty array means no operation-specific payload table was found. `asynchronous_response` classifies the special Cancel and Poll response behavior; it does not replace the shared message and result model. |
| `source_encoding`, `source_requiredness` | `source_encoding` records the exact source encoding for an object-structure, attribute, operation-structure, option, result, credential, or message-field root, or a structure member. `source_requiredness` is copied from a `REQUIRED` column when present; it may be omitted when the source table has no such column and may be an empty string for a blank source cell. These fields do not replace the canonical element kind or imply a project policy. |
| `source_name` | On an attribute record, preserves the exact Item/Object label used for the attribute in its source type table when it differs from the canonical attribute heading. |
| `source_comment` | On message fields, preserves the exact non-empty `Comment` cell from the §8 message structure tables; it is source context, not an independently interpreted requirement. |
| `parent_element_ids`, `requirement_ids`, `profile_ids`, `test_case_ids` | Explicit references; every referenced ID must resolve. Shared fields and nested members carry their own direction and scope instead of inheriting an inaccurate parent classification. |
| `feature_spec`, `implementation_refs`, `verification_refs` | Explicit coverage assignment. `null`/empty arrays mean unassigned and must appear in the generated report. |

## TagRange

Each of the five range rows in Specification §11.56 is a separate record with stable `range_id`, its exact value/range text, allocation kind (`unused`, `reserved`, or `extension`), source-order index, and source reference. A range record is not a singleton tag value and cannot be represented as an assigned tag.

## Requirement

| Field | Rule |
|---|---|
| `requirement_id` | Stable identifier such as `KMIPKIT-REQ-SPEC-8.3-001`; never recycled. |
| `source_clause_ids`, `source_refs` | One or more exact ledger IDs and document sections; table/paragraph anchors distinguish clauses. |
| `source_keyword`, `normative_strength` | Preserve the exact keyword (`MUST`, `MUST NOT`, `REQUIRED`, `SHALL`, `SHALL NOT`, `SHOULD`, `SHOULD NOT`, `RECOMMENDED`, `MAY`, or `OPTIONAL`) and store its canonical strength (`mandatory`, `prohibited`, `recommended`, `discouraged`, or `permission_or_optional`) using the mapping above. |
| `subject`, `summary` | Exact subject plus a concise, non-copyrighted paraphrase of the obligation or permission. |
| `role`, `direction`, `condition` | Client/server/both and message direction, plus profile or protocol condition when applicable. |
| `scope_state` | `client_1_0`, `client_1_1`, `profile_conditional`, `server_only`, or `out_of_scope`, with rationale. |
| `element_ids`, `profile_ids`, `test_case_ids` | Related records; absent test evidence has an explicit gap record rather than an invented ID. |
| `feature_spec`, `implementation_refs`, `verification_refs` | Coverage assignments; empty arrays mean unassigned and must appear in the generated report. |
| `decision_id`, `status`, `review_note` | Accepted deviation/interpretation when applicable and explicit lifecycle state. A deviation from SHOULD, SHOULD NOT, or RECOMMENDED requires an accepted decision. |

## Profile

Each profile has `profile_id`, `name`, `role`, `source_refs`, `source_clause_ids`, `dependency_profile_ids`, `transport_requirements`, `encoding_requirements`, `applicability`, `claim_state`, `requirement_ids`, `element_ids`, and `test_case_ids`. Role is `client`, `server`, or `both`. `applicability` is independently one of `client_1_0`, `client_1_1`, `server_only`, `conditional`, or `out_of_scope`. `claim_state` is one of `not_claimed`, `candidate`, `selected`, `evidence_incomplete`, or `evidence_complete`; `evidence_complete` requires resolved clause, requirement, element, and official Test Cases links. Only a separate reviewed release process may publish a claim. All IDs resolve to the corresponding catalog records.

## TestCase

Each case records its exact official ID, source document and section, evidence category, mandatory/optional status if explicitly stated, linked profiles/requirements/elements, raw source `href`, local fixture path if present, fixture availability, and mapping confidence. Raw `href` is never opened or followed. Fixture presence is determined from the pinned Git tree listing; validators never open catalog-provided fixture paths. Reject drive-qualified, absolute, UNC, `..`, backslash, symlink, and Windows reparse-point entries. Missing fixtures are `unavailable`; a title-only match is not promoted to an operation link without explicit evidence.

## SourceDiscrepancy

Each discrepancy has `discrepancy_id`, `summary`, `source_refs`, `source_authority`, `normative_status`, `alternatives`, `affected_requirement_ids`, `affected_element_ids`, `affected_profile_ids`, `affected_policy_ids`, `downstream_impact`, `state`, `decision_id`, and `erratum_source_refs`. Authority is derived from the strongest cited source. States are `open`, `resolved_by_erratum`, or `resolved_by_approved_decision`. Open records have no selected interpretation or decision; the latter state links to an accepted decision that names the discrepancy. Erratum resolution requires an erratum document in the pinned source manifest; no such document is currently in the four-source inventory, so that state cannot be asserted yet.

## Decision

A decision record has `decision_id`, `source_refs`, `requirement_ids`, `discrepancy_ids`, `policy_ids`, `interpretation`, `approver`, `approval_evidence`, `approved_at`, `consequence`, and `status`. Every linked deviation or resolved discrepancy must also link back to that decision. Only an accepted decision with approver, evidence, and ISO date is a decision record. Unapproved candidates cannot authorize a SHOULD, SHOULD NOT, or RECOMMENDED deviation or source interpretation.

## ProjectPolicy

A project-policy record has `policy_id`, `summary`, `provenance` (`AGENTS.md`, constitution, ADR, or approved product decision), `provenance_ref`, `affected_element_kinds`, and separate `requirement_ids` where the policy implements or constrains an OASIS requirement. File provenance uses an exact repository path and Markdown heading; ADRs must be accepted. Approved product decisions name an accepted decision linked back to the policy. Project policy without an OASIS citation is not emitted as an OASIS requirement. Unknown tags, enum values, bitmask bits, vendor values, and extensions remain distinct from assigned/reserved/unused registry states. The interaction between preserving a received reserved tag and the OASIS prohibition on using it is an open policy/source discrepancy until formally resolved.

## Coverage assignment

Coverage links connect requirement or protocol-element IDs to feature specifications, implementation locations, and verification references. Absence of any link is explicit unassigned coverage; generated reports group these absences without claiming implementation progress.

## Catalog and report invariants

- Every record ID is unique and every relationship points to an existing record.
- Every normative requirement links to at least one source-clause ledger entry and exact source section.
- The source auditor's candidate ID set and checked-in source-clause ledger ID set match exactly.
- All official source case IDs and raw fixture `href` values remain unchanged, even if headings or links appear malformed; display labels and local fixture paths are separate fields.
- Source paths are exact allowlisted repository-relative POSIX paths. Fixture presence is checked only against pinned Git tree metadata; catalog-supplied paths are never opened or filesystem-resolved. Reject absolute, traversal-based, symlink, and reparse-point paths.
- Counts derive from catalog records and source table categories, not from report-only constants.
- Every source count discrepancy, unassigned coverage row, missing fixture, and open source discrepancy appears in the generated report.
- Operation table captions and numbers are copied as printed, including apparent source-label defects; missing operation-specific response tables are represented explicitly by the absence of a response entry rather than by an inferred table.
- JSON is UTF-8, deterministically ordered, newline-terminated, and does not contain copied normative paragraphs.
