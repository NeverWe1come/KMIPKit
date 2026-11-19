# KMIPKIT-0092 Interop API-Route Exhaustiveness Implementation Plan

> **For agentic workers:** implement each task with `superpowers:subagent-driven-development`; use TDD and a task-scoped review before moving to the next task.

**Goal:** Prove the catalog covers the normative KMIP 2.0/2.1 surface, then make shared CI require every case through every public request-construction and execution route and supported encoding/transport combination in Rust, C, Java, and Python.

**Architecture:** Audit the catalog against pinned OASIS inventories before treating it as complete. Add an authored route-capability manifest tied to `specification/api/public-api.json` and installed artifacts. A deterministic generator expands normative cases into route/encoding/transport cells; validators emit evidence for each cell and a shared checker/aggregator rejects missing, duplicate, falsely labelled, or incomparable results.

**Tech Stack:** Python 3.12 shared tooling, JSON Schema 2020-12, GitHub Actions, the existing KMIP 2.0/2.1 test catalog and installed language consumers.

**Spec:** `specs/092-interop-api-route-exhaustiveness/spec.md`

## Global Constraints

- Test KMIP 2.0 and KMIP 2.1 standard cases only.
- Consumer code links installed Rust, C, Java, and Python artifacts built from an exact recorded release commit.
- A declared route must invoke the public API named by the route manifest.
- Keep operation-specific request, response, state, and cleanup assertions.
- Preserve batch order and compare numeric KMIP Result Status/Reason values.
- Keep normalized request/response trees ephemeral; redact secrets and nondeterministic fields.
- Claim byte equality only for deterministic inputs using the same encoding.
- Keep vendor results pending until deployed KMS endpoints are available.

---

### Task 1: Audit complete normative inventories

**Files:**

- Modify: `interop/shared/catalog/generate_test_catalog.py`
- Modify: `interop/shared/tests/test_executable_contract.py`
- Create or modify: source-backed inventory tests under `interop/shared/tests/`
- Read only: `specification/oasis/kmip-2.0/upstream/` and `specification/oasis/kmip-2.1/upstream/`
- Generate: `interop/shared/catalog/kmipkit-1.0-test-catalog.json`

- [ ] Add failing comparisons for every OASIS operation, managed object,
  attribute, TTLV data type, every assigned enumeration value, every mask
  flag, request/response field branch, and defined negative/boundary class.
- [ ] Run focused tests and verify failures point to catalog omissions.
- [ ] Add missing cases and exact OASIS citations through the catalog generator;
  never modify pinned upstream files or generated output by hand.
- [ ] Regenerate the catalog and prove deterministic output plus 100 percent
  version-specific inventory traceability.
- [ ] Commit red/green/refactor evidence.

### Task 2: Define and validate API route capabilities

**Files:**

- Create: `interop/shared/coverage/route-capabilities.schema.json`
- Create: `interop/shared/coverage/route-capabilities.json`
- Create: `interop/shared/tests/test_route_capabilities.py`
- Modify: `specification/api/public-api.json` source/generator only if the existing generated manifest cannot identify request constructors and route differences.

**Interfaces:**

- The manifest identifies each route by stable ID, language, exact installed public API entrypoint, operations/case categories it can construct, supported encodings/transports, and required artifact commit.
- Local generic TTLV construction is a distinct route with no network transport.
- Route declarations are checked against the public API manifest and installed consumer artifact.

- [ ] Add failing schema/contract cases for unknown languages/routes, duplicate IDs, missing API symbols, impossible encoding/transport pairs, and a public route missing from the manifest.
- [ ] Run the focused test and confirm each case fails for its intended reason.
- [ ] Add the schema and capability manifest for the four current public packages. Record only routes verified from the installed artifacts.
- [ ] Validate all four installed packages against the manifest and run the focused tests.
- [ ] Commit the red/green/refactor evidence.

### Task 3: Generate the exhaustive case and route matrix

**Files:**

- Create: `interop/shared/coverage/generate_matrix.py`
- Create: `interop/shared/coverage/coverage-matrix.schema.json`
- Create: `interop/shared/tests/test_coverage_matrix.py`
- Generate: `interop/shared/coverage/coverage-matrix.json`

**Interfaces:**

- Input: `interop/shared/catalog/kmipkit-1.0-test-catalog.json` plus `route-capabilities.json`.
- Output rows identify `case_id`, protocol version, language,
  `construction_route_id`, optional `execution_route_id`, encoding, transport,
  and whether the row is a network exchange or local model check.
- A case is excluded from a route only when the route-capability manifest gives a concrete normative or public-API reason.

- [ ] Add failing tests for omitted applicable versions, missing route products, duplicate/conflicting rows, unverified limitations, and local-TTLV rows incorrectly assigned network transports.
- [ ] Run the focused tests and inspect their expected failures.
- [ ] Implement deterministic matrix expansion with stable sorting and content hashes.
- [ ] Regenerate the matrix and verify row counts by language, version, route, encoding, and transport.
- [ ] Commit the red/green/refactor evidence.

### Task 4: Preserve construction-route evidence and compare runs

**Files:**

- Modify: `interop/shared/schemas/validation-results.schema.json`
- Modify: `interop/shared/aggregator/` result model, grouping, and renderers
- Modify: `interop/shared/tests/` schema, aggregator, parity, and renderer contracts

**Interfaces:**

- Each run carries stable `construction_route_id` and, where public send paths
  differ, `execution_route_id`, in addition to language, protocol version,
  transport, encoding, server fingerprint, catalog digest, release commit, and
  artifact digest.
- Local model runs use a local execution mode and do not invent a network transport.
- Aggregation keeps repeated runs as distinct inputs and does not declare parity for mismatched or missing axes.

- [ ] Add failing tests showing same-language runs through two constructors are both retained and comparable only when all shared dimensions match.
- [ ] Add failures for missing route IDs, mismatched artifact commits, encoding/transport differences, and duplicate conflicting case results.
- [ ] Extend schema, aggregator, and JSON/Markdown/HTML reports without embedding normalized trees or secret data.
- [ ] Run shared schema, aggregation, and renderer suites; verify output dimensions and redaction.
- [ ] Commit the red/green/refactor evidence.

### Task 5: Gate matrix completeness in CI and publish the contract

**Files:**

- Modify: `.github/workflows/interop-common.yml` and relevant CI contracts
- Modify: `interop/shared/README.md` and `specs/092-interop-api-route-exhaustiveness/traceability.md`
- Modify: `specs/092-interop-api-route-exhaustiveness/tasks.md`

- [ ] Add CI checks for generated-matrix freshness, all language capability manifests, absence of missing/duplicate cells, and truthful API limitation reasons.
- [ ] Add a negative CI fixture where one case-route cell is removed and require the gate to fail.
- [ ] Run the full shared test suite, schema validation, catalog generation, and relevant script contracts.
- [ ] Update traceability and document that language-level missing recipes remain pending until their separate PRs satisfy the generated matrix.
- [ ] Commit, push only the feature branch, open a `[WIP]` draft PR, and keep it WIP until the common exact-head CI is green.

## Follow-up language PRs

The common contract does not complete a language validator. Separate Rust, C,
Java, and Python follow-up branches must implement their generated matrix cells,
compare every supported request construction route, and provide controlled
request/response fixtures. Their existing PRs that were merged or opened under
the narrower contracts are not evidence that this new scope is satisfied.
