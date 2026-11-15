# Implementation Plan: KMIP 2.1 Normative Inventory

**Branch**: `feature/KMIPKIT-0002-normative-inventory` | **Date**: 2026-10-04 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/002-normative-inventory/spec.md`

## Summary

Create a complete, reviewed KMIP 2.1 client-scope catalog and coverage report. Use the four checksum-pinned OASIS work products as read-only evidence, record protocol elements and normative requirements in a stable machine-readable catalog, and validate completeness through a deterministic offline tool. Preserve profile distinctions, source defects, missing fixtures, project-policy distinctions, and the 1.0/1.1 direction boundary.

## Technical Context

**Language/Version**: Python 3.12+ for offline catalog validation and reporting; catalog data is JSON.

**Primary Dependencies**: Python standard library only; no runtime or network dependencies.

**Storage**: Checked-in JSON catalog, documented data contract, and generated Markdown coverage report.

**Testing**: Python `unittest`, deterministic output comparison, negative data-contract/semantic validation tests, and the repository's existing CI checks once available on the release branch.

**Target Platform**: Windows, Linux, and macOS developer/CI environments; no platform-specific behavior.

**Project Type**: Internal standards inventory and validation tooling.

**Performance Goals**: Validate and render the complete catalog in under 10 seconds on a standard developer workstation.

**Constraints**: No changes under `specification/oasis/`; no downloading or parsing remote documents; exact source identifiers and checksums must match `SOURCES.md` and `CHECKSUMS.sha256`; HTML decoding follows its pinned declared charset, independent of host locale; all build-time validation is offline and deterministic; source paths are allowlisted and fixture presence is read from Git tree metadata; JSON preflight and HTML event parsing are bounded; report fields use context-specific escaping; CI compares pinned sources against the exact PR base SHA before parsing, with read-only permissions and no secrets; source conflicts remain unresolved records.

**Scale/Scope**: 57 client-to-server and 5 server-to-client operations; 11 data types; 9 managed object types; 23 object structures across §§2.1–3.12; 63 attributes; 7 attribute structures; 41 operation structures; 64 enumeration definitions; 3 bitmasks; 374 single-value tag rows (354 non-reserved named values and 20 reserved values) plus five range rows; all applicable normative client clauses and every client profile/test reference in the pinned source set.

## Constitution Check

- **I. Specification and traceability**: This feature's bounded spec gives exact KMIP source sections and stable-record requirements. The catalog will maintain stable requirement IDs and explicit assignments.
- **II. Test first and evidence based conformance**: Catalog parser/validator/report contracts receive failing tests before production logic. Test IDs are evidence links; this feature makes no conformance claim.
- **III. One core, explicit language boundaries**: No runtime protocol or language API changes are in scope. Inventory tooling does not duplicate protocol semantics.
- **IV. Secure defaults and lossless protocol handling**: No secrets or network data are processed. OASIS data is treated as immutable local input; unknown-value preservation remains labeled as project policy.
- **V. Human governed, reviewable changes**: Work is isolated on the feature branch and presented as a draft PR. Protocol implementation is not part of this spec PR; it requires the reviewed specification to be approved and merged before its own implementation work begins.

**Gate result**: Design artifacts satisfy the pre-implementation planning gate. The specification remains Draft until human review; no catalog/tool implementation may begin before approval and merge. Runtime/protocol implementation is outside this spec PR.

## Design Decisions

- Use one versioned JSON catalog as the human-reviewed source of truth, with a separate field contract. Keep generated Markdown reports out of the authoring path.
- Give source documents, source clauses, protocol elements, tag ranges, requirements, policies, profiles, tests, discrepancies, decisions, and coverage assignments separate record kinds and stable IDs; connect them with explicit IDs instead of embedding repeated source facts.
- Use requirement strength and applicability as separate fields. Represent applicability as `client_1_0`, `client_1_1`, `profile_conditional`, `server_only`, or `out_of_scope`, with a rationale and exact source reference.
- Record the client profile requirement in Specification §14.1 and each candidate/selected/evidence state separately. The catalog itself makes no conformance claim.
- Include both the source Test Cases and Profiles test identifiers, but do not infer an operation mapping when the pinned HTML does not establish one and the fixture is absent.
- Generate an unassigned-coverage report from catalog relationships. Validation fails on malformed identifiers, unknown references, duplicate stable IDs, broken exact-source pointers, count mismatches, or missing disposition fields.
- Keep the catalog validator separate from the source auditor. The source auditor uses only allowlisted local Git blobs, applies a documented whole-word normative-token set and HTML block-boundary algorithm, and compares candidates with the checked-in clause ledger. A reviewer independently checks every Specification and Profiles section/table against the extracted candidates because set equality alone cannot reveal extractor omissions. CI obtains the exact base SHA from the pull-request event, rejects any OASIS-tree diff before parsing, and runs with read-only permissions and no secrets. No remote content is fetched or parsed.

## Project Structure

### Documentation (this feature)

```text
specs/002-normative-inventory/
├── spec.md
├── plan.md
├── research.md
├── data-model.md
├── quickstart.md
├── contracts/
│   └── catalog-format.md
├── checklists/
│   ├── requirements.md
│   └── catalog-readiness.md
└── tasks.md
```

### Source Code (repository root)

```text
specification/catalog/
├── README.md
├── kmip-2.1.json
└── coverage-report.md             # generated from the reviewed catalog

tools/normative_catalog/
├── validate.py                    # offline catalog format/relationship checks
├── audit_sources.py               # pinned local HTML candidate/ledger comparison
├── check_immutable_sources.py     # compare OASIS tree with protected PR base
├── report.py                      # deterministic Markdown report generation
└── tests/
    ├── test_validate.py
    ├── test_audit_sources.py
    ├── test_immutable_sources.py
    └── test_report.py
```

**Structure Decision**: The catalog is a specification artifact under `specification/catalog/`, consistent with ADR-0004 and ADR-0009. Small standard-library Python tools under `tools/normative_catalog/` validate the checked-in data and regenerate the report. No production Rust crate, generated protocol constants, bindings, or OASIS files are changed in this feature.

## Complexity Tracking

No constitution violations or architectural boundary changes are proposed.
