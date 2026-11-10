# Implementation plan: KMIP 2.0/2.1 catalog and result matrix

1. Parse the immutable OASIS 2.0/2.1 specification HTML through headings and
   the reviewed normative catalog. Verify source hashes, operation inventory,
   version-specific sections, attributes, object types, enumerations, masks,
   and negative result reasons.
2. Add failing shared tests for version applicability/references, per-case
   allowed reasons, stable server configuration fingerprints, multi-run
   preservation, comparability, and JSON/Markdown/HTML rendering.
3. Update the catalog generator and schemas to derive per-case applicability
   and source references from the pinned source inventories. Add explicit
   result-reason metadata for bounded negative cases and the stable
   `server_configuration_fingerprint` connection field.
4. Rework the aggregator to preserve each run, separate protocol-version
   matrices, compare only the shared identity/configuration/catalog/library
   evidence, and retain language, artifact, transport, encoding, and API route
   as visible dimensions.
5. Update shared usage documentation, requirement traceability, and CI. Run
   regeneration checks and all shared tests before pushing the WIP branch.

## Files and responsibilities

- `interop/shared/catalog/generate_test_catalog.py`: source hash checking,
  heading/version maps, versioned case references, and negative result reasons.
- `interop/shared/catalog/kmipkit-1.0-test-catalog.json`: deterministic
  generated executable inventory.
- `interop/shared/schemas/test-catalog.schema.json` and
  `server-connection.schema.json`: typed version/reason and fingerprint
  contracts.
- `interop/shared/profiles/connection.example.json`: stable KMS configuration
  identity example.
- `interop/shared/tools/aggregate_matrix.py`: preserve run variants and render
  grouped JSON, Markdown, and HTML matrices.
- `interop/shared/tests/test_executable_contract.py` and
  `test_schemas_and_profiles.py`: executable contract/regression coverage.
- `interop/README.md`: correct version gates and explain the new fingerprint
  and matrix dimensions.
- `.github/workflows/interop-common.yml`: ensure regeneration and shared
  tests are CI gates.
- `specs/088-interop-contract-version-matrix/`: scope, tasks, and traceability.

## Global constraints

- Work on `feature/KMIPKIT-0088-interop-contract-matrix`, based on release
  commit `fdafef8a3eb0c7b4b18d6433b78fde4151493ceb`.
- Use the checked-in OASIS 2.0 and 2.1 HTML only; verify their recorded SHA-256.
- Do not edit language validators, upstream OASIS files, or generated catalog
  output by hand.
- Keep the PR draft with `[WIP]`; never approve or merge.
- Preserve vendor-neutral KMIP 2.0/2.1 scope and report no live-server claims.

## Task sequence

### Task 1: executable source inventory and schema expectations

1. Add failing tests for the six 2.1-only operations, versioned refs, the
   Vendor Attribute section remap, two 2.1-only Validation enumerations, and
   the negative Encrypt result-reason set.
2. Run the focused tests and confirm the current generated catalog fails those
   assertions.
3. Add schema fields for `requires.expected_result_reasons` and
   `server_configuration_fingerprint`; test invalid values and missing fields.
4. Run focused schema tests and keep the test commit red until the generator
   and example configuration are updated.

### Task 2: source-backed catalog generation

1. Implement a standard-library HTML heading reader that verifies both pinned
   source hashes and indexes normative sections by number and heading name.
2. Assign operation versions by exact operation-heading presence. Assign
   attribute/object/enum/mask versions by corresponding normative section
   presence and name mapping. Fail generation when a source element cannot be
   reconciled.
3. Generate exact `normative_refs` per applicable version and bounded allowed
   result reasons for the negative cases.
4. Regenerate the shared JSON catalog and run the failing tests; all should
   pass with a deterministic `--check` result.

### Task 3: multi-run result matrices

1. Add tests with multiple runs for one language/server using different API,
   transport, and encoding values; assert no run is overwritten.
2. Add tests that variant-only differences remain comparable, while mismatched
   server version, explicit configuration fingerprint, catalog hash, or
   library commit remain incomparable.
3. Refactor aggregation into per-server/per-protocol-version matrices whose
   result columns identify the run ID and all execution axes.
4. Update Markdown and HTML to render every run column, library correctness,
   server compatibility, parity differences, and not-run coverage; add
   redaction/rendering tests.

### Task 4: traceability and CI

1. Update the shared README and spec traceability for all new requirements.
2. Ensure the common workflow regenerates the catalog, checks canonical
   vectors, and runs all shared tests.
3. Run `python3 interop/shared/catalog/generate_test_catalog.py --write`, then
   `--check`; run all `interop/shared/tests` under Python 3.12 with pinned
   `jsonschema`.
4. Run whitespace/diff checks, verify no language or upstream files changed,
   and commit Red/Green/Refactor phases with DCO sign-off.
5. Push the branch, verify all GitHub checks, and send the commit/check results
   for independent review. Do not open a PR until parent review authorization.
