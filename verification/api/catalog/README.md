# API coverage catalogs

- `kmip-3.0-draft-api.json` maps the seven draft client operation routes to the binding APIs and the pinned KMIP 3.0 source.
- `fluent-operation-inventory.json` inventories versioned operations, fluent API scope, and request model ownership.
- `fluent-operation-model-owners.json` records the Rust request models and their supporting tests.

The canonical public API contract and schema are `bindings/shared/public-api.json` and `bindings/shared/public-api.schema.json`. Normative source catalogs and OASIS documents are under `verification/conformance/`. Generate the fluent operation inventory with `python verification/conformance/generate_fluent_operation_inventory.py --repo-root .` and review any evidence changes before committing them.

## Fluent API field inventory

Run `python3 -B -m verification.conformance.fluent_field_inventory` from the
repository root to emit the current field inventory as JSON. Use `--check` to
check the pinned source files and the coverage of the extracted tables.

The inventory lists operation payload rows and request/response header and
batch-item rows for KMIP 2.0, 2.1 and the pinned 3.0 draft. It preserves the
source text describing requiredness and conditions. `modelCandidates` points
to existing protocol models. Empty `evidence` means the public routes still
need an explicit assessment. Nested structures, field domains and per-language
consumer evidence are tracked as follow-up work.
