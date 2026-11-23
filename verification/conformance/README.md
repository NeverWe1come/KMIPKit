# KMIP conformance reference inputs

This directory groups the inputs and tools used to trace KMIPKit behavior to the pinned KMIP specifications. Inputs were imported from `release/1.0.0` at `1f84a00d6f453c2ec09c36ca0622f289521e137e`; the binding API manifest remains the version maintained on the review branch.

- `catalog/`: normative element, tag, value, and profile catalogs, together with the source review and inventory report.
- `requirements/`: CSV matrices linking project requirements to implementation and test evidence.
- `oasis/`: pinned KMIP 2.0, 2.1, and 3.0 documents, source inventories, checksums, and official test vectors. The imported 3.0 source is identified as a working draft in its source inventory.
- Python scripts and `tests/`: catalog validation, source auditing, deterministic generators, reports, and their regression checks.

The public binding contract and its schema live in `bindings/shared/`. Draft API routes and fluent operation inventories live in `verification/api/catalog/`. Dependency exceptions live in `verification/dependencies/dependency-policy-exceptions.json`.

## Checking source integrity

From each `oasis/kmip-*` directory, run `sha256sum --check CHECKSUMS.sha256`. Source documents and official vectors retain their original bytes and notices. Repository paths in project inventories and catalogs change with this layout; identifiers and recorded evidence states retain their imported meaning.

Generators and validators accept `--repo-root` and use the canonical paths above. For example, from the repository root:

```sh
python verification/conformance/validate.py --repo-root .
python verification/conformance/generate_fluent_operation_inventory.py --check --repo-root .
```

The complete catalog validation also checks historical project policy provenance, feature approvals, and evidence paths. Some of those records refer to `AGENTS.md`, `.specify/`, `specs/`, and `docs/` on the full development branch, which are not imported here. Those checks can therefore report missing provenance; copying reference data does not approve a policy, refresh a historical coverage report, or establish full KMIP conformance. Retain these failures until the associated records are reviewed.

The imported public API schema also exposes existing declaration metadata differences in the review manifest. Resolve those differences by reviewing the declarations and their evidence, without automatically promoting their verification state.
