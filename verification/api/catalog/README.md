# API coverage catalogs

- `kmip-3.0-draft-api.json` maps the seven draft client operation routes to the binding APIs and the pinned KMIP 3.0 source.
- `fluent-operation-inventory.json` inventories versioned operations, fluent API scope, and request model ownership.
- `fluent-operation-model-owners.json` records the Rust request models and their supporting tests.

The canonical public API contract and schema are `bindings/shared/public-api.json` and `bindings/shared/public-api.schema.json`. Normative source catalogs and OASIS documents are under `verification/conformance/`. Generate the fluent operation inventory with `python verification/conformance/generate_fluent_operation_inventory.py --repo-root .` and review any evidence changes before committing them.
