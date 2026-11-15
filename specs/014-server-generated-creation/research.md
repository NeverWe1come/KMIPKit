# Research: Server-Generated Object Creation

## Implementation acceptance evidence

The human maintainer accepted KMIPKIT-0014 by merging [PR #53](https://github.com/NeverWe1come/KMIPKit/pull/53) into `release/1.0.0` on 2026-10-08 (merge commit `a2572084d3d6f42019bed13234598e6d77478796`). The source and catalog corrections were merged in [PR #57](https://github.com/NeverWe1come/KMIPKit/pull/57) on 2026-10-08 (merge commit `55c0c0cc42a6422cee72183a3532fc1e169f297c`). Both commits are ancestors of implementation base `0e50e4b9859532cf7a0832deb2d510a3af4563de`. GitHub metadata reports no review decision or submitted reviews for those PRs; this record treats the maintainer merge as the acceptance event and does not claim a separate review.

## T003–T005 Red/Green/Refactor evidence

- **Red** — Signed-off commit `9fc022c` adds `crates/kmipkit-protocol/tests/unit/attribute_tests.rs`, including `property_roundtrips_ordered_repeated_direct_attribute_items`. Before `AttributeSet` existed, `cargo test -p kmipkit-protocol --lib attribute_tests` failed to compile with unresolved `crate::AttributeSet`, as expected.
- **Green** — Signed-off commit `a8d8171` adds and exports the ordered `AttributeSet`, preserves generic item tags/types/repetition/order, validates the Table 150 Vendor Attribute fields, and redacts Debug output. The focused attribute tests passed (7/7), and `cargo test -p kmipkit-protocol --all-features` passed before refactoring.
- **Refactor** — Signed-off commit `bbce620` extracts the Vendor Identification and Attribute Name checks from the cardinality scan, preserves the behavior, and asserts each payload-free malformed-field category, including duplicate and wrong-type fields.

The normative tag values were checked against the pinned OASIS v2.1 Specification §4.60/Table 150 and the catalog’s §11.56 records: outer `Attribute` 0x420008, `Vendor Identification` 0x42009D, `Attribute Name` 0x42000A, and `Attribute Value` 0x42000B. No normative conflict was found.

Final Phase 1 verification on the implementation branch:

- `cargo fmt --all --check` passed.
- `cargo clippy -p kmipkit-protocol --all-targets --all-features -- -D warnings` passed.
- `cargo test -p kmipkit-protocol --all-features` passed: 78 library unit tests, all protocol integration suites, and 2 doctests. The seven focused attribute tests, including the property roundtrip, passed.
- `cargo llvm-cov -p kmipkit-protocol --lib --all-features --json --summary-only` passed; `attribute.rs` has 109/110 covered lines (99.09%).
- `python -m unittest discover -s tools/normative_catalog/tests -p test_immutable_sources.py -q` passed (15 tests); catalog validation passed with 4 sources, 1,411 clauses, and 4,024 records; `report.py --check` passed; the immutable-source checker passed against base `0e50e4b9859532cf7a0832deb2d510a3af4563de`; `audit_sources.py --check` audited 1,411 candidates successfully.
- The pinned upstream immutability check confirms all OASIS upstream bytes match the base. No T006 or later operation task was started, and no KMIPKIT-0016 or language-binding API was changed.

## Decision 1 — Use the pinned OASIS v2.1 operation tables as the schema

**Decision**: Implement payloads from Specification §§6.1.8–6.1.10 and Tables 186–195. Use §§5.1–5.4 and Tables 157–160 for operation attribute groups and direct §4 Object Attribute items. Use §4.60/Table 150 only for the distinct Vendor Attribute structure. Shared message/result behavior follows the applicable shared sections.

**Rationale**: The pinned HTML is the normative source and the inventory records operation element IDs and applicable requirement IDs. The Usage Guide is informative and cannot fill a normative gap.

**Alternatives considered**: Derive payloads from a server implementation or Usage Guide examples. Rejected because those may omit normative optional, repeated, or error cases.

## Decision 2 — Preserve a typed operation envelope and dynamic attributes

**Decision**: Expose distinct request/response types for each operation and ordered attribute groups containing direct §4 Object Attribute TTLV items as specified by Tables 157–160. Each generic Item retains its tag and typed value; entries may repeat and remain in wire order. Do not wrap ordinary attribute items in Table 150's Attribute Name/Attribute Value structure. Table 150 is the separate §4.60 Vendor Attribute structure and requires Vendor Identification. The client preserves the Common, Private, and Public groups; the server applies the Table 189 union rule. Do not accept caller-defined conversions or arbitrary pre-encoded request bodies.

**Rationale**: Tables 157–160 specify direct §4 attribute items, which already carry their attribute identity in the TTLV tag. The generic Item preserves that tag and the typed value without lossy conversion. The distinct Vendor Attribute structure uses its own Vendor Identification and Attribute Name fields under §4.60/Table 150. This representation does not expand outbound assigned-value or registered-extension policy. Convenience attribute-specific builders may be added before 1.0 if they preserve this wire contract and do not locally merge groups.

**Alternatives considered**: Convert every attribute into a rigid closed enum immediately. Rejected for this operation slice because it duplicates extensible values and risks losing unrecognized attributes. Treat the entire request as an arbitrary TTLV Item. Rejected because it bypasses the closed typed operation boundary from KMIPKIT-0007.

## Decision 3 — Preserve operation result and generic response ownership

**Decision**: Create a typed response view from the validated response batch item and retain a reference to the owning ResponseMessage for fields not modeled by the operation type. A non-success KMIP result has no fabricated success payload.

**Rationale**: This matches existing Discover Versions and async response conventions, preserves unknown fields, and avoids cloning secret-bearing response structures.

**Alternatives considered**: Consume or normalize the full response into the operation model. Rejected because it can discard extensions and field order and increase secret copies.

## Decision 4 — Use one explicit call and preserve asynchronous outcomes

**Decision**: The operation call enters the existing closed typed request writer once. Pending is represented under KMIPKIT-0009; this feature never polls, retries, waits automatically for completion, or claims server completion.

**Rationale**: KMIP asynchronous behavior is part of the existing client contract, and automatic follow-up would hide delivery and server policy from the caller.

## Decision 5 — Add no runtime dependency

**Decision**: Use only workspace TTLV, protocol, client, transport, and zeroization dependencies.

**Rationale**: The payload schemas need no new parser, crypto provider, or transport. Dependency policy stays reproducible.

## Decision 6 — Keep source integrity checks aligned with the immutable boundary

**Decision**: Keep the exact pinned OASIS files under `specification/oasis/kmip-2.1/upstream/` immutable. The source checker may allow modifications only to the KMIP 2.1 `README.md` and `SOURCES.md` paths already tracked in the exact base commit, plus additions under `fixtures/`; it rejects adding, deleting, or renaming either inventory file, every other OASIS path, and changes to fixture files already present in the base commit. Preserve the existing exact-base and catalog fixture validation.

**Rationale**: T001 and T002 require recording the project-owned fixture's provenance and mapping its exact XML path as available. A whole-`specification/oasis/` prohibition blocks those approved project artifacts. Keeping the exact pinned `upstream/` subtree immutable, recording and verifying the fixture SHA-256, and retaining catalog path validation preserves the distinct source-integrity checks.

**Evidence**: `tools/normative_catalog/tests/test_immutable_sources.py` exercises permitted tracked inventory edits and new fixture additions; rejects upstream changes, unrelated OASIS paths, existing fixture changes, and inventory additions/deletions/renames. The first Red/Green/Refactor cycle is preserved in `8a60b42`, `6e02e42`, and `1c061bd`; the follow-up inventory add/delete/rename Red/Green/Refactor cycle is preserved in `97d4d1e`, `2f6469f`, and `43ae31a`. The official XML download and local fixture are each 2,725 bytes and have SHA-256 `882e0f57ff2cc42b214e2ffb40bf9c80105489aab00f07a6ea5e2c1c395474ad`.

## Normative evidence

Catalog-linked and feature-assigned requirements for Create Key Pair are KMIPKIT-REQ-SPEC-6.1.9-001-001, -001-002, -006, and -007. Create Split Key links KMIPKIT-REQ-SPEC-6.1.10-001. Section 2.8/Table 9 describes Prime Field Size in the Split Key object, while §6.1.10/Table 193 marks the Create Split Key request field optional. FR-015 is therefore documented as a KMIPKit client restriction, not as an OASIS request requirement. The pinned KMIP Test Cases HTML includes `TC-CREATE-SD-1-21` for Create/Secret Data (§2.12); its byte-identical linked XML is pinned at `specification/oasis/kmip-2.1/fixtures/TC-CREATE-SD-1-21.xml` with provenance in `SOURCES.md` and catalog availability marked `available`. The catalog retains the Secret Data object association; availability is not evidence that the case passed. Only the Create request/response batch item is used for a derived test; the full two-operation case is not claimed as passing. No direct Create Key Pair or Create Split Key case appears in the pinned work product.
