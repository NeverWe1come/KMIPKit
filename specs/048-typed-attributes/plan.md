# Implementation Plan: KMIPKIT-0048 Typed KMIP Object Attributes

**Branch**: `feature/KMIPKIT-0048-typed-attributes` | **Date**: 2026-11-05 | **Spec**: [spec.md](spec.md)

## Summary

Provide typed Rust definitions, construction, and reading for each direct KMIP 2.1 Object Attribute while retaining the generic TTLV path and exact source multiplicity. Derive tags and value categories from the reviewed normative catalog, preserve open raw Enumeration values, and use the existing TTLV Structure model for complex attribute payloads. The feature is stacked on the 0047 branch as directly authorized in `approval-record.md`; final merge destination remains `release/1.0.0`.

## Technical Context

**Language/Version**: Rust 2024, MSRV 1.94; Python 3 for deterministic generation/tests  
**Primary Dependencies**: Existing `kmipkit-protocol`, `kmipkit-ttlv`, API manifest and normative catalog generators  
**Storage**: N/A  
**Testing**: Rust unit/integration tests, Python generator/manifest tests, API export audit  
**Target Platform**: Existing Rust workspace platforms; binding parity deferred to 0053  
**Project Type**: Multi-crate Rust library with bindings  
**Performance Goals**: Typed reads must borrow existing Items without copying; insertion may allocate only the owned Item/value required by TTLV  
**Constraints**: No wire-format change; preserve generic APIs, unknown values/tags, child order, duplicates allowed by source; do not expose secret values in diagnostics  
**Scale/Scope**: 62 direct tagged/special attributes from the 63 catalog records; Certificate Attributes has no direct TTLV tag and is excluded as recorded in the spec

## Normative Design Decisions

1. Extend the existing `tools/normative_catalog/generate_attribute_types.py` so one validated catalog input produces both the TTLV type-enforcement table and typed marker output; do not add a second parser for the same catalog semantics.
2. A typed attribute marker identifies an immutable catalog tag and its source-defined TTLV value category.
3. The typed owned value and borrowed view cover Boolean, Byte String, Date-Time, Enumeration, Integer, Interval, Structure, Text String, RNG Parameters, and Unique Identifier's source-permitted alternatives.
4. Enumeration wrappers preserve all raw 32-bit values, including values not yet named in 0047.
5. `AttributeSet` typed queries return every matching occurrence in wire order. Existing `try_new` / `try_push` multiplicity validation remains authoritative.
6. Typed Current/New wrappers contain exactly one direct Item. Existing constructors stay source-compatible.
7. Structure payloads remain generic and lossless in this feature; specialized object/key builders belong to 0049.
8. Manifest reconciliation is explicit and deterministic. No generated Rust or documentation output is hand-edited.

## Constitution Check

- Approved bounded spec and exact OASIS references: pass; see `spec.md` and `gap-matrix.md`.
- Red/Green/Refactor with distinct evidence commits: required.
- Unknown values, generic access, secret redaction, no unsafe code: preserved.
- FFI parity is not claimed here and remains a tracked dependency of 0053.
- Dedicated worktree/branch and stacked PR from 0047: pass; direct maintainer plan authorization is recorded.

## Project Structure

```text
crates/kmipkit-protocol/src/attribute.rs        # typed generic wrappers and collection access
crates/kmipkit-protocol/src/typed_attributes.rs # generated marker/value mapping integration
crates/kmipkit-protocol/tests/unit/             # exhaustive encoding, repetition, and wrapper tests
tools/normative_catalog/                        # deterministic typed-definition generation
tools/api_manifest/                             # reviewed public-route mapping/reconciliation
specs/048-typed-attributes/                     # spec, source matrix, traceability and evidence
specs/048-typed-attributes/contracts/            # public Rust API contract
docs/user-guide/en/ and docs/user-guide/es/     # Rust typed attribute examples
```

## Verification Strategy

Run red/green focused protocol and generator checks; test every one of the 62 mappings against tag/category, test all value kinds and open values, exercise repeated attributes in source order and generic fallback, then run formatting, Clippy, relevant workspace tests, manifest generation/auditing and coverage checks. Binding consumers are intentionally verified in 0053, not asserted by this PR.
