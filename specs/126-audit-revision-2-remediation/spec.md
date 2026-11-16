# Specification: Remediation of Second Independent Audit Findings (KMIPKIT-0126)

## 1. Overview and scope

Addresses all 7 findings identified in the second independent audit (`auditoria_kmipkit_1_0_0_segunda_revision.md`), reconciling normative versioned catalogs, code generators, protocol version gating, client batch response decoding, and interop test harnesses with the pinned OASIS KMIP 2.0, 2.1, and 3.0 specifications.

## 2. Normative and project references

- OASIS KMIP Specification v2.0 §11.55 (Tags), §11.33 (Operations).
- OASIS KMIP Specification v2.1 §11.56 (Tags), §11.34 (Operations), §3 (Attributes).
- OASIS KMIP Specification v3.0 §11.56 (Tags, Table 603 Split Key Polynomial), §11.11 (Credential Type), §11.12 (Cryptographic Algorithm), §11.17 (Digital Signature Algorithm), §11.25 (Item Type), §11.46 (Query Function), §11.47 (Recommended Curve), §11.50 (Revocation Reason Code), §11.60 (Unique Identifier), §12.1 (Cryptographic Usage Mask).
- AGENTS.md §1 (Authority and source order), §5 (OASIS requirements and conformance), §6 (TDD and testing), §7 (Rust and code quality), §9 (Public API and compatibility).
- ADR-0002 (TTLV encoding), ADR-0017 (facade dependencies), ADR-0033 (catalog-driven generation).

## 3. Requirements

- **KMIPKIT-0126-FR-001 (Catalog v2.1 tags & attribute policy)**: Replace corrupted `v2_1_tags` in `kmip-versioned-tags.json` with the exact 36 KMIP 2.1 tags from `kmip-2.1.json` (§11.56 Table 487). Ensure the 7 object attributes (`0x42016A`, `0x42016B`, `0x42016C`, `0x42016D`, `0x42016E`, `0x42016F`, `0x420172`) have `"is_object_attribute": true`. Derive `v21_tags` in `generate_attribute_policy.py` from `v2_1_tags` so `Rotate Interval`, `Rotate Automatic`, `Rotate Offset`, and `Rotate Date` have `min_version: (2, 1)`.
- **KMIPKIT-0126-FR-002 (Catalog KMIP 3.0 typed attributes & generator parity)**: Update `generate_attribute_types.py` to accept `versioned_tags` and emit all 99 typed attributes (62 KMIP 2.1 + 32 KMIP 3.0 new + 5 promoted). Update all 6 generated files, tests, and API parity assertions across Rust, C, Java, and Python.
- **KMIPKIT-0126-FR-003 (Split Key Polynomial & wire names fail-closed)**: Add `"enumeration_family": "Split Key Polynomial"` to `0x4201B6` in `kmip-versioned-tags.json` and add `Split Key Polynomial` (`Polynomial-283` `0x00000001`, `Polynomial-285` `0x00000002`) to `kmip-versioned-values.json` (§11.56 Table 603). Enforce fail-closed validation in `generate_wire_names.py` for any Enumeration entry without an enumeration family. Regenerate wire names, named values, and public API artifacts.
- **KMIPKIT-0126-FR-004 (OASIS KMIP 3.0 enumeration deltas)**: Incorporate verified OASIS KMIP 3.0 enumeration deltas from `kmip-spec-v3.0.html` into `kmip-versioned-values.json` (`Credential Type`, `Item Type`, `Query Function`, `Recommended Curve`, `Revocation Reason Code`, `Unique Identifier`). Strictly forbid inventing non-existent codes in `Digital Signature Algorithm` (§11.17 Table 558) or `Cryptographic Usage Mask` (§12.1 Table 616).
- **KMIPKIT-0126-FR-005 (KMIP 2.0 VersionGate attribute & operation tags)**: Correct the hex tags for `Digest` (`0x0042_0034`), `Key Value Present` (`0x0042_00BB`), and `Key Format Type` (`0x0042_0042`) in `crates/kmipkit-protocol/src/version_gate.rs`, unit tests `crates/kmipkit-protocol/tests/unit/kmip20_tests.rs`, and `specs/070-kmip-2-0-interoperability/spec.md`. Fix the operation names for `0x36..=0x3B` in spec 070 and unit tests.
- **KMIPKIT-0126-FR-006 (KMIP 3.0 typed response readers in ClientBatchItemResult)**: Add per-operation typed response readers in `ClientBatchItemResult` / `BatchItemResult` for all 7 KMIP 3.0 operations (`CreateGroup`, `CreateUser`, `CreateCredential`, `Obliterate`, `Deactivate`, `Encapsulate`, `Decapsulate`) across Rust, C, Java, and Python. Update `docs/architecture/public-api.md` and `specification/api/public-api.json`.
- **KMIPKIT-0126-FR-007 (Java Interop Validator live routes & library pin)**: Update `library-pin.json` in `interop/java-kmip-validator/` to the current `release/1.0.0` commit, unblock KMIP 2.0 live routes in `Main.java`, and update provenance and contract tests.
