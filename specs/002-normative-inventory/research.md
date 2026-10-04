# Research: KMIP 2.1 Normative Inventory

## Decision: Use the four pinned local OASIS work products as the complete evidence set

**Decision**: The initial inventory is based only on the Specification v2.1 OASIS Standard, Profiles v2.1 OASIS Standard, Usage Guide v2.1 Committee Note 01, and Test Cases v2.1 Committee Note 01 recorded in `specification/oasis/kmip-2.1/SOURCES.md` and `CHECKSUMS.sha256`.

**Rationale**: `docs/compliance/document-hierarchy.md` establishes the authority order; `docs/adr/0009-conformance-and-oasis-sources.md` requires immutable, pinned evidence and reviewed errata. No approved erratum or normative machine-readable source is present in the pinned source inventory. The baseline checksums were verified before this inventory work.

**Alternatives considered**: Fetching the current OASIS pages or fixtures during validation was rejected because it would make reviews mutable and non-reproducible, and would violate ADR-0004's prohibition on build-time scraping/downloads.

## Decision: Treat base clauses, profile clauses, tests, and guide text according to the repository hierarchy

**Decision**: Normative client requirements are derived from applicable Specification and Profiles clauses. Test Cases and profile test IDs provide evidence, not independent requirements. The Usage Guide remains informative unless a normative source incorporates it.

**Rationale**: This is the established policy in `docs/compliance/document-hierarchy.md`, `docs/compliance/conformance.md`, and ADR-0009. It prevents test examples and explanatory text from silently expanding protocol obligations.

**Alternatives considered**: Treating each test description or guide example as a requirement was rejected because the hierarchy explicitly forbids it.

Specification §1.2 and Profiles §1.2 explicitly incorporate RFC 2119 for the listed keywords. Preserve each exact keyword and normalize its strength from that reference: REQUIRED/MUST/SHALL are mandatory; MUST NOT/SHALL NOT are prohibitions; RECOMMENDED/SHOULD are recommendations; SHOULD NOT is discouraged; MAY/OPTIONAL has permission-or-optional strength. RFC 2119 defines keyword semantics here and is not entered as an additional KMIP client requirement or a fifth OASIS work product.

## Decision: Record all client profiles, but separate applicability and conformance claims

**Decision**: Inventory all client profiles and relevant clauses/tests, mark their 1.0 applicability based on the approved product boundary, and report Specification §14.1 as a profile-selection prerequisite. Do not claim a profile from inventory presence alone.

**Rationale**: KMIP Specification §14.1 requires a client to conform to one or more client profiles. KMIPKit's current definition names no completed profile claim; the Baseline and HTTPS profiles align with the 1.0 KMIP 2.1, TTLV, and HTTP/1.1 scope, while specialized profiles are conditional. A profile can be claimed only after applicable clauses and official evidence pass.

**Alternatives considered**: Assuming profile compliance from operation coverage alone was rejected by the constitution and ADR-0009. Omitting profiles was rejected because the 1.0 client conformance rule and profile-specific requirements would be hidden.

## Decision: Preserve source inconsistencies as discrepancy records

**Decision**: Record the KMIP §11.5 Batch Error Continuation wording conflict and the profile cross-reference/case-label defects with both source locations and alternatives. Do not choose behavior or repair references inside the derived catalog.

**Rationale**: The source hierarchy explicitly prohibits silent interpretation. The selected feature is an inventory, so it can satisfy its scope by exposing the conflict and assigning affected behavior to a resolution gate.

**Alternatives considered**: Selecting one sentence as authoritative or correcting the published HTML link in derived data was rejected as an unreviewed interpretation.

## Decision: Store the reviewed catalog as JSON and validate it offline

**Decision**: Use one versioned JSON data file plus a documented field contract. A standard-library validator checks syntax, stable IDs, references, source checksums, disposition fields, and aggregate counts; a deterministic report generator emits Markdown. The structural validator consumes the reviewed catalog; a separate source-audit command reads only checksum-pinned local HTML and never parses remote sources.

**Rationale**: JSON is deterministic, widely readable across planned languages, and has standard-library support in the Python validation environment. A separate format contract makes the record rules reviewable. Keeping remote extraction out of CI ensures OASIS sources remain pinned evidence; the local audit verifies the reviewed snapshot without network access.

**Alternatives considered**: YAML was rejected to avoid parser/version ambiguity; direct HTML scraping at build time was rejected by ADR-0004; duplicating separate catalogs per language was rejected because it would drift.

## Decision: Keep unavailable XML fixtures visible as evidence gaps

**Decision**: Record every case ID visible in the pinned HTML, mark whether its fixture exists in the immutable source tree, and leave fixture-level mapping unresolved where the file is absent.

**Rationale**: The pinned Profiles document links to 93 XML fixtures and Test Cases CN01 has 110 test sections; those XML fixtures are absent from the source tree. The local HTML establishes identifiers and some descriptions but not all case details.

**Alternatives considered**: Reconstructing expected fixture contents from titles or fetching unpinned fixture files was rejected because it could invent or silently change official evidence.

## Source count notes

- Operation definitions: 57 client-to-server (§6.1) and 5 server-to-client (§6.2).
- Table 487: 374 single-value six-hex-digit rows, including 20 values explicitly marked Reserved; 354 rows are non-reserved named values; five range rows for unused, reserved, and extension ranges are tracked separately.
- Other verified counts: 11 data types; 9 managed object types; 12 object structures; 63 attributes; 7 attribute structures; 41 operation structures; 64 enumeration definitions; 3 bitmask definitions.
- Test evidence: 110 Test Cases CN01 sections; 93 profile fixture links (86 mandatory and 7 optional); fixture XML is absent locally.

## Open source issue register

- Specification §11.5 describes Batch Error Continuation `Continue` as continuing after a failure and also says processed items have been undone. The catalog records both statements and defers the behavior interpretation.
- Profiles §6.1 and §6.2 cite `(5.1.3, 5.6.3)` under Baseline conformance even though §5.6.3 is Symmetric Key Lifecycle test material.
- Profiles §5.4.2, §5.13.1, and §5.18.3 contain suspect conflict-scope references; §6.9, §6.13, and §6.14 also contain suspect cross-references.
- Profiles §5.3.1 has conditional XML/JSON content types alongside a binary TTLV body requirement; source interpretation is not inferred for those out-of-scope encodings.
- Profile fixture headings/links disagree for Quantum Safe and cryptographic cases; Test Cases CN01 contains malformed/odd case labels and a malformed fixture link.
- Profile and Test Cases XML fixture contents are unavailable in the pinned tree; exact operation-level mappings cannot be inferred from their titles alone.



## Decision: Use a checked-in source-clause ledger as the completeness denominator

**Decision**: Store a source-clause ledger in the catalog and compare it with a deterministic candidate-locator report generated from the exact pinned local Specification and Profiles HTML. Every candidate gets a requirement link or an explicit reviewed disposition. The source audit reads no remote documents and does not infer normative meaning.

**Rationale**: Aggregate catalog counts alone cannot prove that a normative paragraph or table row was not omitted. Stable structural locators let the inventory identify all local normative-text candidates and preserve an explicit denominator while keeping manual interpretation reviewable.

**Alternatives considered**: Trusting a narrative checklist or hard-coded total alone was rejected because it cannot expose a missing clause. Parsing mutable remote OASIS pages was rejected; only checksum-pinned local copies are inputs.

## Decision: Bound and strictly parse the catalog, confine paths, and escape report values

**Decision**: Reject duplicate JSON keys, unknown fields, invalid UTF-8, unsafe paths, unanchored IDs, excessive nesting/size/count/string lengths, and report-injection values. Allow source paths only from the four-entry manifest and fixture files only beneath the local pinned OASIS directory. Escape all untrusted report text and emit links only from allowlisted source metadata.

**Rationale**: The catalog is checked-in data but remains an input to parsing and Markdown generation. Strict behavior removes parser ambiguity, path traversal, and output injection risks without adding runtime dependencies.

**Alternatives considered**: Relying solely on code review or trusting authored JSON was rejected because CI and future tooling must handle malformed or adversarial changes safely. Adding a JSON Schema library was rejected to keep the tooling standard-library-only; a documented contract and strict validator are used instead.

## Decision: Enforce pinned-source immutability against the protected base

**Decision**: CI rejects any diff under `specification/oasis/` relative to the protected PR base, regardless of checksum edits. Local checksum verification remains an additional integrity check.

**Rationale**: A mutable checksum manifest cannot by itself prove that an upstream file is unchanged. Comparing the full tree to the protected base enforces the repository's immutable-copy rule.

**Alternatives considered**: Checking only that file hashes match the in-branch manifest was rejected because a source and its manifest could be changed together.

- The interaction between KMIPKit's unknown-tag preservation policy and Specification §11.56's prohibition on using reserved tag values is tracked separately: decode-time retention versus re-encoding remains unresolved until an approved interpretation specifies the safe behavior.
