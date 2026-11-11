# KMIPKIT-0099 Implementation Evidence

## Scope

This delivery adds a Rust-only Create Split Key builder over the existing
typed request and client exchange. It does not claim C, Java, or Python parity
or completion of the KMIPKIT-0093 end-to-end journeys.

## Red / Green / Refactor

| Stage | Commit or artifact | Evidence |
|---|---|---|
| Red | `69331acb3` | Focused `cargo test -p kmipkit-client --lib fluent_create_split_key_tests -- --nocapture` failed to compile because `CreateSplitKeyBuilder`, `RequestBuilder::create_split_key`, and the four required-field errors did not exist. |
| Green | `b7fe8ffca3dee514383937fa08e9d3f57e64a178` | Added the typed builder, local validation, ordered expert attributes, batch conversion, options and one-exchange execution. Focused suite initially: 9 passed. Existing Create Split Key execution: 13 passed. Existing Create Key Pair fluent suite: 20 passed. Implementer Clippy check passed. Independent static review: no actionable findings. |
| Green boundary follow-up | `a39e1ce76` | Added narrow, exact AST allowlist and regression tests for the explicit expert values this API requires. Full client suite passed (819 unit tests plus integration/doctest groups); full Clippy passed. |
| Green cardinality follow-up | `eec20f385` | Added direct builder coverage for duplicate singleton rejection through `validate`, `build_request`, and `execute` (`NotSent`, zero exchanges). The focused test passed; the full client suite then passed with 820 unit tests. |
| Refactor | pending | Bilingual guide, exact API manifest declarations and generated outputs, traceability, and final verification are being completed on this branch. |

## Verification commands

| Command | Result |
|---|---|
| `cargo test -p kmipkit-client --all-features fluent_create_split_key_tests -- --nocapture` | Passed, 10 tests after the cardinality follow-up. |
| `python3 -B -m unittest tools.api_manifest.tests.test_0099_fluent_create_split_key_manifest -v` | Passed, 3 tests. |
| `python3 -B tools/api_manifest/generate.py --write` | Passed after aligning approval status and excerpt evidence to the manifest contract; generated 12 outputs. |
| `python3 -B tools/api_manifest/generate.py --check` | Passed; verified all 12 generated outputs after the final manifest and spec edits. |
| `python3 scripts/test_user_guide_examples.py` | Passed; compiled 30 marked Rust examples, including the new bilingual examples. Existing unrelated dead-code warnings were emitted from `execute.rs`. |
| `cargo test -p kmipkit-client --all-features` | Passed after adding exact signature exceptions and duplicate-singleton coverage: 820 unit tests passed; all integration, trybuild, and doctest groups passed (3 external Cosmian tests remain intentionally ignored). |
| `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings` | Passed after fixing test-only style findings and rechecked after cardinality coverage. |
| `cargo fmt --all --check` | Passed. |
| `git diff --check` | Passed. |
| `python3 -B tools/api_manifest/audit_rust_exports.py --repo-root .` | Exit 0. All 50 KMIPKIT-0099 declarations were observed and all 18 exact standard-trait paths were excluded. The release-wide report still lists 4,938 unrelated missing manifest entries and one pre-existing `notObserved` declaration (`matchesManifest: false`). |

Operation-level exchange assertions use the controlled `ExchangeScript`
transport and verify `NotSent`/`PossiblySent`, one exchange, typed Success,
Failure, Pending, all response identifiers, exact correlation, and no implicit
retry or Poll. They do not simulate a real network server; the TLS fixture
journeys remain covered by a later end-to-end delivery.

## Review and PR

The independent builder review reported no actionable findings. The WIP PR
number, exact head, hosted CI results, and remaining required check evidence
will be added after the Refactor commit and PR creation.
