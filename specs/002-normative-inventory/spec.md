# Feature Specification: KMIP 2.1 Normative Inventory

**Feature Branch**: `feature/KMIPKIT-0002-normative-inventory`

**Created**: 2026-10-04

**Status**: Approved

**Approval evidence**: The maintainer authorized autonomous implementation of the complete execution plan on 2026-10-04.

**Input**: User description: Build a complete, traceable KMIP 2.1 inventory for the 1.0 client scope, covering operations, protocol elements, normative client requirements, profile applicability, and official test evidence. Preserve unresolved source conflicts and report unassigned coverage.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Review client protocol coverage (Priority: P1)

As a KMIPKit maintainer, I need a single inventory of the KMIP 2.1 elements used by the 1.0 client so that no client-initiated operation, shared message rule, value, object, attribute, option, or result flow is omitted from later specifications.

**Why this priority**: Later protocol design depends on complete, direction-aware scope boundaries.

**Independent Test**: Compare the inventory against the pinned Specification headings and value tables; all enumerated client and server operation headings and protocol element totals must reconcile, with direction and 1.0 disposition recorded.

**Acceptance Scenarios**:

1. **Given** the pinned KMIP 2.1 Specification, **When** a maintainer reviews the operation inventory, **Then** each of its 57 client-to-server and 5 server-to-client operation definitions has an exact section, direction, scope disposition, request/response references where applicable, and linked test evidence when the pinned sources provide it.
2. **Given** the named protocol-element tables in the pinned Specification, **When** a maintainer reviews the catalog coverage report, **Then** it reconciles the 11 data types, 9 managed object types, 23 object structures across §§2.1–3.12, 63 attributes, 7 attribute structures, 41 operation structures, 64 enumeration definitions, 3 bitmasks, and 374 single-value tag rows, comprising 354 non-reserved named values and 20 reserved values, while separately identifying the five range rows and all reserved, unused, extension, and server-direction values.
3. **Given** a protocol element defined by KMIP 2.1 but outside the 1.0 client direction, **When** the inventory classifies it, **Then** it remains represented and is marked for 1.1 or as outside 1.0 with a cited reason rather than being silently dropped.

---

### User Story 2 - Trace normative requirements and test evidence (Priority: P1)

As an implementer or reviewer, I need stable requirement records connecting each applicable normative client clause to its source, planned capability, and test evidence so that implementation and conformance claims can be audited.

**Why this priority**: KMIPKit requires complete normative traceability before implementation and release claims.

**Independent Test**: For every normative source clause in the reviewed clause ledger, follow its stable ID to its exact source section, client/direction classification, scope state, and linked test evidence or explicit evidence gap.

**Acceptance Scenarios**:

1. **Given** a normative KMIP client clause, **When** a reviewer opens its requirement record, **Then** the record contains a stable ID, source document/version/stage/date/section, a concise paraphrase, normative level, role and direction, 1.0 applicability, linked protocol elements, and test-case IDs or an explicit reason evidence is unavailable.
2. **Given** a MUST/SHALL NOT prohibition, **When** the inventory is reviewed, **Then** it is marked as requiring negative verification; any deviation from SHOULD, SHOULD NOT, or RECOMMENDED has an accepted decision; MAY and OPTIONAL capabilities remain represented.
3. **Given** a pinned OASIS test-case ID, **When** it is recorded, **Then** it is identified as conformance evidence rather than an independent normative requirement, and any absent fixture is reported without inventing its contents.

---

### User Story 3 - Identify profile boundaries and unresolved source issues (Priority: P2)

As a maintainer, I need an explicit profile matrix and source discrepancy report so that 1.0 scope, later work, and interpretation blockers are visible before protocol families are specified.

**Why this priority**: Profile clauses can add requirements, while source defects and contradictory wording must not silently become implementation behavior.

**Independent Test**: Review the profile and discrepancy report against the pinned Profiles document; each client profile and each identified inconsistency has exact references, classification, impact, and disposition.

**Acceptance Scenarios**:

1. **Given** the KMIP 2.1 Profiles Standard, **When** a maintainer reviews the profile matrix, **Then** every client profile and its client conformance clauses, required test IDs, encoding/transport constraints, 1.0 applicability, and evidence status are represented; no profile is claimed conformant by this inventory.
2. **Given** the base specification's client conformance requirement to conform to one or more client profiles (§14.1), **When** the inventory reports 1.0 readiness, **Then** the profile-selection and evidence state is explicit and cannot be mistaken for a completed conformance claim.
3. **Given** conflicting or apparently incorrect pinned source statements or references, **When** they are entered in the discrepancy report, **Then** both sides, document status, exact sections, plausible observable interpretations, and downstream effect are preserved without an agent-selected resolution.

### Edge Cases

- A tag-table row describes a range or reserved/unused values rather than one named tag; it must be counted separately from the 374 single-value rows and must still be categorized.
- A shared structure contains both client-to-server and server-to-client fields; direction must be recorded at field or requirement level rather than inferred from the parent section alone.
- A test case is referenced in HTML but its XML fixture is absent from the pinned tree; the ID may be recorded, while fixture-level assertions remain unavailable.
- An OASIS document contains conflicting normative prose or a malformed cross-reference; preserve the source text locations and mark the interpretation unresolved.
- A normative statement is applicable only under a selected profile or conditional feature; record its condition and profile relation.
- A registry value is reserved, vendor-defined, unknown to KMIPKit, or introduced by future extension; distinguish OASIS-prescribed handling from KMIPKit's separate lossless-preservation policy.
- A JSON document contains duplicate keys, excessive nesting, invalid UTF-8, or an oversized string; reject it before rendering any report.
- A catalog path is absolute, traverses outside its allowed root, or resolves through a symlink outside the repository; reject it without opening the target.
- A catalog name or discrepancy contains Markdown delimiters, HTML, line breaks, or control characters; escape or reject it before report output.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The inventory MUST use only the four pinned KMIP 2.1 sources under `specification/oasis/kmip-2.1/`, preserve their source identity and checksums, and leave those upstream files unchanged.
- **FR-002**: The inventory MUST represent every operation in Specification §§6.1–6.2 and record its exact section, direction, 1.0/1.1 disposition, and applicable request/response structures and official test-case references.
- **FR-003**: The inventory MUST cover the named data types, managed object types, object structures, every message field and nested structure member, credential forms, attributes, attribute structures, operation structures, enumeration definitions and every named enumeration value, bitmasks and every defined bit value, options and results, and named unique tag values in the pinned Specification. It MUST reconcile the counts stated in User Story 1 and the exact names and values against their source tables and structures.
- **FR-004**: Every applicable normative client requirement found in the reviewed source-clause ledger MUST have a stable ID and a record containing its exact source document, stage/version/date/section, concise paraphrase, normative level, role/direction, scope state, protocol-element links, profile conditions, verification links, and review status.
- **FR-005**: The requirement inventory MUST preserve the exact source keyword among MUST, MUST NOT, REQUIRED, SHALL, SHALL NOT, SHOULD, SHOULD NOT, RECOMMENDED, MAY, and OPTIONAL, and map REQUIRED/MUST/SHALL to mandatory strength, MUST NOT/SHALL NOT to prohibited strength, RECOMMENDED/SHOULD to recommended strength, SHOULD NOT to discouraged strength, and MAY/OPTIONAL to the canonical permission-or-optional strength while retaining the exact keyword. It MUST flag prohibitions for negative verification, deviations from SHOULD, SHOULD NOT, or RECOMMENDED for an accepted decision, and MAY/OPTIONAL capabilities for representation and use when in scope.
- **FR-006**: The inventory MUST distinguish client-initiated operations and protocol asynchronous result flows in 1.0 from server-initiated operations planned for 1.1, and MUST NOT misclassify async Poll/Cancel behavior as server-initiated work solely because a response is pending.
- **FR-007**: The profile matrix MUST include every client profile in the pinned Profiles Standard, its normative clauses, mandatory and optional test IDs, applicable transports/encodings, client conformance conditions, 1.0 disposition, and evidence gaps. Server-only profiles MUST be identified as such.
- **FR-008**: The inventory MUST record the §14.1 requirement that a KMIP client conforms to one or more client profiles, while keeping profile applicability, selected target profile, evidence completion, and a public conformance claim as distinct states.
- **FR-009**: The test crosswalk MUST identify source test-case IDs and whether the pinned source contains the referenced fixture. Test cases MUST be treated as evidence unless a normative source clause incorporates a requirement.
- **FR-010**: The discrepancy register MUST preserve apparent contradictions, malformed cross-references, inconsistent case labels, absent fixture references, and project-policy conflicts with exact source locations, normative status, alternatives, impact, and a resolution state. It MUST NOT silently resolve them.
- **FR-011**: Project policies MUST have separate stable records and provenance, including preservation of unknown values and extension data, so project policy is not misrepresented as an OASIS requirement. Unknown/future registry values, vendor values, and OASIS-reserved or unused values MUST have distinct classifications.
- **FR-012**: A generated coverage report MUST list every applicable requirement or capability without an assigned feature specification, implementation, or verification reference, and MUST summarize counts by direction, normative level, scope, profile, and evidence state.
- **FR-013**: The checked-in catalog and report MUST be deterministic, reviewable, and self-contained; routine build and CI validation MUST NOT download, scrape, or parse mutable remote OASIS pages. Offline auditing MAY parse only the checksum-pinned local copies.
- **FR-014**: Technical artifacts, requirement identifiers, catalog labels, and paraphrases MUST be written in English and cite exact OASIS sections; the human-facing progress discussion may be Spanish.
- **FR-015**: Source and local fixture paths MUST be repository-relative POSIX paths confined to the pinned OASIS tree; source records MUST use the exact four allowlisted paths in `SOURCES.md`. Validation MUST reject absolute, drive-qualified, UNC, traversal, and symlink/reparse-point paths. Fixture availability MUST be read from pinned Git tree metadata rather than by following catalog-provided paths. Raw fixture `href` values are metadata only and MUST NOT be opened or followed.
- **FR-016**: Catalog parsing MUST reject duplicate JSON object keys, unknown fields, invalid UTF-8 or unpaired escaped surrogates, and resource-bound violations before report generation. A bounded preflight scan MUST enforce limits before constructing the object graph: 16 MiB input, nesting depth 32, 100,000 record-array entries, 1,000,000 JSON value tokens or object members globally, and 65,536 UTF-8 bytes per string field.
- **FR-017**: Stable IDs MUST match anchored record-kind-specific patterns. Report generation MUST use separate context-specific encoders for table text and link labels, reject control characters in identifiers and citations, and visibly escape every C0/C1 control character in all rendered text fields, including terminal escape characters. It MUST emit links only to the four exact canonical OASIS URLs; catalog text MUST NOT be interpolated into link destinations or URL fragments.
- **FR-018**: Catalog and report ordering MUST use documented, locale-independent keys; report bytes MUST be UTF-8 with LF newlines and MUST exclude timestamps, absolute paths, hostnames, and other environment-dependent values. Permuting input record order MUST NOT change output.
- **FR-019**: OASIS allocation (`assigned`, `reserved`, `unused`, `extension range`) MUST be distinct from unknown-to-KMIPKit or vendor-defined values; reserved, unused, or range entries MUST NOT be represented as assigned usable tags.
- **FR-020**: CI MUST compare the entire pinned OASIS Git tree and source manifest with the pull-request event's exact base commit SHA, fail closed when that SHA is unavailable, and reject any added, removed, renamed, mode-changed, symlink, or content-changed path even when checksums are changed in the same PR. The guard MUST run before any source parser, in a pull-request workflow with read-only repository permissions and no secrets.
- **FR-021**: The checked-in source-clause ledger MUST enumerate every normative-text candidate in the pinned Specification and Profiles sources, assign each a requirement or documented disposition, and be compared by an offline source-audit tool against the local pinned Git blobs. Candidate extraction MUST use documented normative keywords and block boundaries after concatenating inline text. An independent section-by-section review MUST check the extractor's coverage against the actual source, including missed blocks and tables, before the ledger is accepted.
- **FR-022**: The source auditor MUST parse only allowlisted local Git blobs with a non-fetching parser, decode using the pinned HTML documents' declared charset (currently Windows-1252) independent of host locale, and enforce an 8 MiB per-document, 16 MiB aggregate input, 64-level nesting, 1,000,000 HTML-node, and 16 MiB extracted-text limit before report generation; it MUST fail closed on unsupported charset, malformed declaration, or exceeded limit.

### Key Entities *(include if feature involves data)*

- **Source document**: A pinned OASIS work product with document identity, version, stage, date, source location, and checksum.
- **Requirement**: A stable, reviewable record of a normative clause, its paraphrase, strength, applicability, relationships, and evidence.
- **Protocol element**: An operation, message field or structure member, credential form, data type, object, attribute, enumeration/value, bitmask/bit, option, result, tag allocation, or extension rule defined by KMIP 2.1.
- **Profile**: A named client or server selection of KMIP requirements, operations, values, transports, encodings, and conformance evidence.
- **Test evidence**: An official test-case identifier, source location, fixture availability, and linked requirement/profile records.
- **Source clause**: A stable source locator for a normative-text candidate in a pinned Specification or Profiles section, its detected normative strength, review disposition, and linked requirement IDs.
- **Project policy**: A KMIPKit-only rule with separate provenance and affected values, explicitly distinguished from OASIS requirements.
- **Tag range**: One of the five reserved, unused, or extension ranges in Specification §11.56, represented separately from single-value tags.
- **Decision**: An approved interpretation or deviation with affected sources and consequence.
- **Source discrepancy**: A preserved conflict, suspect cross-reference, malformed case reference, or evidence gap with alternatives and downstream impact.
- **Coverage assignment**: A relationship from a requirement or protocol element to a feature specification, implementation location, and verification reference, including an explicit unassigned state.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All 57 client-to-server and 5 server-to-client operations in the pinned Specification are represented once each with correct source section and direction.
- **SC-002**: Counts for the 11 data types, 9 managed object types, 23 object structures across §§2.1–3.12, 63 attributes, 7 attribute structures, 41 operation structures, 64 enumeration definitions, 3 bitmasks, and 374 single-value tag rows (354 non-reserved named values and 20 reserved values) reconcile exactly with the pinned Specification tables; every message field, nested structure member, credential form, named enumeration value, defined bit value, option, and result matches its source table or structure; all five range rows and other range/reserved/unused entries are reported separately.
- **SC-003**: An independent review of every Specification and Profiles section confirms complete coverage of normative prose and table candidates; 100% of the documented extractor's candidates are in the source-clause ledger and each has a linked requirement or explicit reviewed disposition.
- **SC-004**: Every client profile, every pinned profile/test-case reference, and every discovered source discrepancy has an explicit record and disposition; missing fixtures are visible in the report.
- **SC-005**: The coverage report identifies all unassigned 1.0/1.1 requirements and capabilities; zero item is represented as implemented or verified without a corresponding assignment.
- **SC-006**: Repeated report generation and generation after any permutation of identical input records produce byte-identical UTF-8/LF output with no host-specific data.
- **SC-007**: Duplicate JSON keys, malformed or oversized JSON/HTML inputs, out-of-root or symlink/reparse paths, invalid IDs, control characters, and Markdown/link injection payloads are rejected or safely escaped before report output.
- **SC-008**: A CI change to any pinned OASIS path, mode, symlink, content, or source manifest fails against the exact pull-request base commit even if its checksum is updated.
- **SC-009**: Every requirement/element record cites an exact pinned OASIS section; informative Usage Guide or Test Cases text is never labelled as an independent normative requirement, and all technical inventory artifacts are in English.

## Assumptions

- The four pinned HTML sources listed in `specification/oasis/kmip-2.1/SOURCES.md` are the full source set for this inventory; no approved errata or normative machine-readable source is currently included.
- The KMIP Specification is the primary normative source, Profiles add normative profile requirements, Test Cases provide evidence, and the Usage Guide is informative according to `docs/compliance/document-hierarchy.md`.
- The feature inventories all client profiles and records likely 1.0 applicability, but does not claim profile conformance. A target profile must be selected and evidenced before any conformance claim; the §14.1 requirement remains visible until that is done.
- XML test fixtures referenced by the pinned HTML may be unavailable. The catalog records explicit case identifiers while reporting fixture absence and does not reconstruct unavailable fixture contents.
- The project lossless-preservation contract applies to the representation/model design but remains a KMIPKit policy, not an OASIS normative clause.
- Apparent OASIS conflicts are data in the discrepancy register; later implementation of affected behavior remains gated on an authoritative resolution.
