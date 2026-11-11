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
| Refactor | `2ff6529eeceeb6b465674714b5b2d454b9b5f9a3` | Added bilingual guide examples, 50 exact Rust API declarations, 18 scoped trait exclusions, generated outputs, traceability, and manifest tests. Independent final review found no remaining issues after one invalid source reference was removed and a duplicate-singleton builder test was added. |

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
is [PR #181](https://github.com/NeverWe1come/KMIPKit/pull/181), targeting the
KMIPKIT-0097 branch so work can continue down the stack before that parent is
merged. It remains a draft with `[WIP]` in its title while hosted CI runs. The
PR currently covers Rust only; C, Java, and Python parity and the full
KMIPKIT-0093 journeys remain outstanding.

Hosted CI was still in progress at the time this evidence was updated. The
current exact head and check results are maintained by GitHub on PR #181; this
evidence does not claim hosted CI is green. The local verification above is
scoped to the recorded commands and should not be read as release-wide API
manifest completeness: the export audit reports the noted unrelated baseline
gaps.

## CI-Driven Refactor

The SonarCloud check for PR #181 head `f27e24a2d3e8f494f500ef791492dd784b2df410`
reported 6.8444% duplicated new code (77 duplicated lines in
`crates/kmipkit-client/src/fluent.rs`, above the 3% gate). The duplication
spanned positioned-attribute insertion, vendor-attribute construction, and
Message Extension tag validation shared by the Create and Split Key builders.
The refactor extracts those operations into shared helpers without changing
the public builder contract. An independent code review found no actionable
issues.

| Command | Result |
|---|---|
| `cargo test -p kmipkit-client --all-features fluent_create_split_key_tests -- --nocapture` | Passed, 10 tests. |
| `cargo test -p kmipkit-client --all-features fluent_create_key_pair_tests -- --nocapture` | Passed, 20 tests. |
| `cargo fmt --all -- --check` | Passed. |
| `cargo clippy -p kmipkit-client --all-features --all-targets -- -D warnings` | Passed. |
| `git diff --check` | Passed. |

Hosted SonarCloud and CI results for the refactor commit remain pending. The
PR must remain `[WIP]` until every required check passes on its current head.
