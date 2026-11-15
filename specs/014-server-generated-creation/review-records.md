# Independent Review Record: KMIPKIT-0014

Date: 2026-10-09  
Pull request: [#61](https://github.com/NeverWe1come/KMIPKit/pull/61)  
Current reviewed implementation head: `db78f73ee6f4a6ee97695956af4b913dfbb2f06f`

## Convergence

- Protocol/model convergence review: no remaining buildable gaps after T029.
- Client/lifecycle convergence review: no remaining buildable gaps after T029.
- Scope included Create, Create Key Pair, Create Split Key, Table 191 value consistency, generic attribute preservation, generated attribute type metadata, traceability, and drift checking.
- Review mode was static inspection; the convergence agents did not execute tests or inspect Red/Green/Refactor commit history.

## Independent QA

- Initial review of `d64d8c3..63ce7db` reported one P2 acceptance gap: no measured 95% line coverage for the new catalog attribute type generator.
- Follow-up review of `63ce7db..4061c7b` accepted the correction. CI now runs the generator's focused tests with pinned coverage.py and enforces `--fail-under=95`; the workflow contract, developer guide, traceability, and T030 document the gate.
- Local coverage evidence on the reviewed code: 117/117 statements, 100%; focused workflow contracts passed. The reviewer did not execute tests.
- QA found no additional functional or cross-platform gap. The workflow contract checks command presence and the existing exact-pinned dependency contract; it does not assert the relative YAML order or a fixed coverage.py patch version.
- Final review of `4061c7b..e2bdaf5` accepted the test-only path normalization that compares against `root.resolve(strict=True)`, covering Windows and macOS canonical path differences without changing generator behavior.
- Follow-up review of `661a587..c6e445d` accepted the deterministic connection-worker and response-worker spawn-failure regressions and inline fallbacks. It identified a race in T032's metric assertion because the detached connection worker can decrement the counter after the response becomes readable.
- QA follow-up of `c6e445d..41fd777` confirmed that the test now waits for an explicit connection-worker completion message before inspecting the metric. The race is resolved; no further actionable QA findings. The reviewer also noted that the existing socket-clone error path retries the clone inside `serve_tcp_connection` and still cannot recover if descriptor exhaustion persists. This is outside the injected thread-spawn behavior covered by T031.

## Independent security review

- Review of `d64d8c3..63ce7db`: no reportable security findings in attribute type generation/validation, errors, tests, or CI drift checking.
- Follow-up review of `63ce7db..4061c7b`: no reportable security findings in the coverage workflow, tests, or documentation.
- Review of `4061c7b..e2bdaf5`: the path-normalization test correction changes no production code, dependencies, or CI behavior and introduces no security risk.
- Review of `661a587..41fd777`: no security findings. The injectable spawner is private to the DNS fixture; production uses the fixed thread spawner. The fallback retains the accepted socket or response, serializes writes through the existing mutex, and the tests remain loopback-only. This was static review only.
- Reviews were static only; the security reviewer did not execute tests, fuzzing, or dynamic analysis.
- These agent reviews do not replace the qualified human security review required before 1.0.0.

## Final CI evidence

- Run [37866222386](https://github.com/NeverWe1come/KMIPKit/actions/runs/37866222386) completed successfully for head `e2bdaf5e55bfe4a549240e6e6da66c4595872421`. Core, coverage, adapter coverage, aggregate thresholds, language bindings, script contracts, FFI sanitizer, fuzz, normative inventory, and dependency policy passed; scheduled-only jobs were skipped.
- The first Coverage/macOS attempt failed in the existing `kmipkit-test-support` DNS fixture at `dns_tests.rs:250` with `ConnectionReset` during the first response-length read. The client had not yet sent the short frame used to test intentional fixture closure. The same job passed when rerun on the same head; the test also passed in Core/macOS stable and Rust 1.94 and the preceding coverage run. `crates/kmipkit-test-support` was unchanged between the preceding and failed heads. Evidence favors an intermittent fixture/environment failure, but does not establish its root cause. No error was suppressed and no source change was made for it.
- A subsequent Coverage/macOS attempt, run [37868050267](https://github.com/NeverWe1come/KMIPKit/actions/runs/37868050267) on documentation head `f0a0f5c2a65ce254c68379312f6defd3c4b45e21`, failed in the same DNS fixture at the first response-length read, this time with `UnexpectedEof`. This second failure increased the evidence that accepted TCP work could be lost under resource pressure, but its logs do not prove that thread creation failed in that run. T031/T032 now deterministically inject failures at both worker creation points and verify inline delivery; fresh cross-platform CI on `41fd777` is pending.
- Run [37870208992](https://github.com/NeverWe1come/KMIPKit/actions/runs/37870208992) passed for implementation head `db78f73ee6f4a6ee97695956af4b913dfbb2f06f`. All six Core matrix legs, all three platform coverage jobs, adapter coverage, aggregate thresholds, language bindings, script contracts, FFI sanitizer, fuzz smoke, normative inventory, and dependency policy passed; scheduled-only jobs were skipped. This run includes the deterministic T031/T032 regressions and current review records.
- Adapter coverage verified `tools/normative_catalog/generate_attribute_types.py` at 117/117 statements (100%) against the enforced 95% threshold.
