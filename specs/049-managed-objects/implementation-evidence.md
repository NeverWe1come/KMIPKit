# KMIPKIT-0049 Implementation Evidence

## Source and branch baseline

- Authorized feature chain root: `feature/KMIPKIT-0043-t076-shared-client-mapping` (PR #104).
- Immediate predecessor: `feature/KMIPKIT-0048-typed-attributes`, PR #115, commit `48f8ceac29eea37ac6a65fa81fc9580af33930bd`.
- Current feature branch: `feature/KMIPKIT-0049-managed-objects`; the maintainer authorized the phase stack to start from #104 and to use predecessor branches until eventual integration into `release/1.0.0`.
- Pinned OASIS source: `specification/oasis/kmip-2.1/upstream/kmip-spec-v2.1-os.html`, SHA-256 `8BF9D914C097E98A6509AA1FFCBF03406F738066E940597AEE93D0A5E07ADDCF`.
- Normative catalog: `specification/catalog/kmip-2.1.json`, SHA-256 `0D68D403956FE0BA4057BED346FDE57D1BC8938D975338E02BE2D435EC16FBEE`.
- Source check: `traceability.md` contains 53 unique operative requirement IDs; all 53 exist in the pinned catalog. The 9 structure tag associations were independently checked against catalog `wire_value` entries sourced from KMIP 2.1 §11.56.
- No clarification question remained: KMIP 2.1 tables resolve structure shape, ordering, tags, and conditional requirements; unresolved server behavior and out-of-scope capabilities remain explicit exclusions.

## Red — tests before implementation

The following tests were added before production code. Their compile failures are the expected Red state because KMIPKIT-0049 types and additive request helpers do not yet exist.

| Command | Result | Expected Red evidence |
|---|---|---|
| `cargo test --offline -p kmipkit-protocol --tests --no-run` | Failed as expected | `E0432` for missing managed-object, Key Block/Key Material, wrapping, and transparent key types; no implementation has been added yet. |
| `cargo test --offline -p kmipkit-client --no-run` | Failed as expected | `E0432` for missing typed models and `E0599` for the not-yet-added `RegisterRequest::from_managed_object` and `ImportRequest::from_managed_object`. |
| `cargo fmt --all --check` | Passed | All Red tests and current Rust workspace files are formatted. |

The Red tests cover all nine object tags and fields, the §3.1–§3.3 key structures, the nine transparent material structures, conversion preservation, validation negatives, redaction, and fake-transport Register/Import requests. The exact test output is retained in the task command history; implementation acceptance will be based on Green results, not on these expected compile failures.

## Development commits

- Specification/design: `686394817a34467bf4757fc9c7af7069df422575`.
- Normative order/tag correction: `d83cfec699ddf913258c37479f2bc78c2ca4e1d3`.
- Red: pending.
- Green: pending.
- Refactor: pending.

## Remaining evidence

Implementation tests, generator and export audits, coverage, security-focused diff review, acceptance-criteria review, CI status, and the draft PR identity are pending. No conformance or interoperability claim is made by this evidence record.
