# Implementation Evidence

## T012 — Shared Rust/C parity fixture corpus

The source fixture corpus at `tests/fixtures/api/shared-capability-parity.json` defines six language-neutral cases for API-level discovery, per-configuration registry isolation, resource-limit error/redaction behavior, ordered mixed batch outcomes with a Pending correlation value, generic TTLV open-value/order preservation, and extension-registry recognition/preservation. Every case records stable 0043 and owning-feature requirement IDs, a literal input setup, expected observable output, error category when applicable, cleanup expectations, and Rust/C consuming test names. Extension cases resolve to the existing shared extension fixture source.

| Phase | Commit | Command | Result |
|---|---|---|---|
| Red | `835b4e8c` | `python -B -m unittest tools.api_manifest.tests.test_parity.PublicApiParityTests.test_shared_capability_parity_fixtures_cover_rust_and_c_contracts -v` | Expected failure: the shared API parity corpus did not exist. |
| Green | `6774abcc` | Same focused command | PASS: fixture corpus and the required capability, requirement, output, error, cleanup, consumer, and extension-source references were present and valid. |
| Refactor | `3c548559` | `python -B -m unittest tools.api_manifest.tests.test_parity -v`; `git diff --check` | PASS: all 8 parity tests passed; no whitespace errors. Reused one helper for capability-case lookup without changing fixture behavior. |

The Rust and C consumer test names are the planned consumers for subsequent binding tasks; this task establishes the shared input and expected-output source. Runtime C/JNI/Python parity and manifest-backed evidence remain gated by their respective implementation tasks.
