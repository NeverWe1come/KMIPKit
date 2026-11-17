# Traceability: KMIPKIT-0087

| Requirement | Implementation evidence | Verification |
|---|---|---|
| 0087-FR-001 | Offline planner and preflight validate the shared connection contract and enumerate version-applicable rows. | `test_plan_is_offline_and_accounts_for_each_protocol_version`; workflow validates both plans against the catalog. |
| 0087-FR-002 | `library-pin.json`, `tools/install_artifact.py`, and `tools/verify_artifact.py` build and verify the wheel from the pinned release commit. | Installed-package integration tests and `verify_artifact.py`; CI uses Python 3.12 and Rust 1.94. |
| 0087-FR-003 | Catalog dispatcher covers typed client recipes, local enum/bitmask tests, async workflows, and explicit documented API gaps. | `test_every_advertised_case_has_a_public_request_recipe_or_special_runner`; all emitted typed batch items pass `validation_report()`. |
| 0087-FR-004 | Connection checks and mutation gate run before client creation; credentials are read from environment variables. | Mutation-gate tests, live-run safety test, and controlled mTLS tests. |
| 0087-FR-005 | JSON, Markdown, and HTML results use shared schema 2.0 and separate library/server/policy/parity outcomes. | Shared-schema report test and sanitization assertions. |
| 0087-FR-006 | Public typed and selected high-level APIs, TTLV raw-value round trips, and request response checks are exercised. | Constructor sweep, enum/mask test, and controlled TLS tests. Full cross-language parity is outside this Python-only run and is not claimed. |
| 0087-FR-007 | `.github/workflows/interop-python.yml` installs the pinned wheel and tests both protocol-version plans with controlled TLS fixtures. | `Interop Python Validator` workflow. |

## Explicit Python API boundaries

The pinned public Python API has no server-initiated Request handler,
multipart Init/Update/Final client workflow, Batch ID Placeholder builder,
or generic outbound-TTLV client request. The typed request surface is broader
than the high-level surface; unsupported routes are represented as
`not_run_documented_api_gap` and are not scored as server failures. The
validator reports `parity: not_compared` unless an actual comparison exists.

Normative source: OASIS KMIP 2.0/2.1 Specifications §§2–12 and the matching
OASIS Profiles transport/encoding clauses. Exact operation clauses are carried
by each shared catalog row.
