# KMIPKIT-0050 Implementation Evidence

## Branch and scope

- Branch: `feature/KMIPKIT-0050-parameters-auth-client`.
- Base: `feature/KMIPKIT-0049-managed-objects` (PR #116), as recorded in
  `approval-record.md`.
- At the initial 2026-11-07 audit snapshot, the latest synchronized predecessor
  was `28d209fd0` (PR #116), included through normal merge `c5e3051ef`.
- Scope: typed Cryptographic Parameters, request-scoped KMIP Authentication,
  direct Rust client methods, public API reconciliation, examples, and tests.

## Red, Green, Refactor commits

| Work group | Red | Green | Refactor |
|---|---|---|---|
| Cryptographic Parameters model | `c4028f6e6`, `1b5381e3c` | `823eb040c` | `d0d7d396b` (subsequent API cleanup) |
| Operation parameter setters | `50f0c045f` | `cc4cca605` | `d0d7d396b` |
| Request Header Authentication | `72bb37335`, `311374bb3` | `45dc27944` | `f000939e5` |
| Complete direct Client API | `3c5927b3e` | `c3faccac9` | `d70a1071b` |
| Direct method transport parity | `3b8c5a025` | `f0a98a151` | `d0d7d396b`, `f34495eb3` |

`f34495eb3` extends the fake-transport matrix to exercise the default forms
of the direct methods as well as their `_with_options` forms, and checks that
the Cryptographic Parameters debug views do not reveal `P Source`.

## Verification

Successful Windows checks on this branch:

- `cargo fmt --all --check`.
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- `cargo test --workspace --all-features`.
- `cargo test --offline -p kmipkit-client --lib authentication_owner` passed
  both ADR-0012 candidate-callsite lifecycle tests: ownership survives a
  successful exchange and is zeroized afterward; a partial-write failure
  zeroizes the owner, reports the delivery outcome, and does not retry.
- After synchronizing to predecessor head `28d209fd0`, the same focused command
  passed again (2 tests); `cargo fmt --all --check` and `git diff --check` also
  passed.
- `cargo llvm-cov --workspace --all-features --locked --json --output-path coverage-raw-local.json`
  (full Windows workspace test and coverage instrumentation completed).
- `python scripts/coverage_gate.py normalize --workspace . --input coverage-raw-local.json --output coverage-local.json`.
- `cargo test -p kmipkit-protocol --all-features --test cryptographic_parameters_public`
  (9 tests passed).
- Focused direct-method parity and option-forwarding tests passed.
- `python -m unittest tools.api_manifest.tests.test_reconcile_0050_parameters_auth_client`
  (3 tests passed).
- `python tools/api_manifest/generate.py --check` (12 generated outputs current).
- `python tools/api_manifest/audit_rust_exports.py --check` (complete, matches
  manifest, no missing or unobserved Rust paths, zero issues).
- `python -B -m unittest scripts.tests.test_requirement_traceability -v`
  (2 tests passed).
- `python -B -m unittest discover -s tools/normative_catalog/tests -p 'test_feature_traceability.py' -v`
  (30 tests passed; one symlink-privilege-dependent test skipped by Windows).
- `python scripts/test_user_guide_examples.py` (22 Rust guide examples compiled).
- Manifest reconciler dry run: zero declarations and exclusions left to add.

Windows line coverage calculated from the normalized report using the exact
eligible-source inventory in `scripts/coverage_gate.py`. The repository's
documented generated-source exclusions were applied; this is a Windows-only
measurement, not the three-platform CI aggregate.

| Scope | Covered / executable lines | Result |
|---|---:|---:|
| Changed executable Rust lines | 305 / 318 (95.91%) | Meets 95% |
| `kmipkit-ttlv` | 735 / 741 (99.19%) | Meets 95% |
| `kmipkit-protocol` | 21,442 / 22,306 (96.13%) | Meets 95% |
| `kmipkit-transport` | 3,718 / 3,907 (95.16%) | Meets 85% |
| `kmipkit-ffi` | 12,505 / 14,258 (87.71%) | Meets 85% |
| Workspace total | 46,597 / 49,625 (93.90%) | Meets 90% |

The unfiltered `cargo llvm-cov --text` diagnostic previously showed 89.98% for
`kmipkit-protocol`; that denominator included generated files excluded by the
repository's exact documented coverage inventory. With the same exclusions
the coverage gate uses, the local Windows measurements exceed every Rust
threshold, including protocol and changed executable lines. The separate
instrumented C-consumer report and the Linux/macOS platform reports have not
been produced locally, so this does not establish that the complete hosted
coverage job passes.

Only Windows has been exercised locally. Linux and macOS checks and hosted CI
remain unverified; the repository's current GitHub billing issue may prevent
hosted CI from running. The Windows `aws-lc-sys` build emitted non-fatal
environment warnings about NASM and an optional compiler probe.

## Reviews and publication

- Independent QA review found no API-manifest, documentation, or parity
  discrepancy; its coverage concern was resolved by recalculating against the
  gate's documented generated-source exclusions.
- Independent security review and Codex Security diff scan found no confirmed
  security defect. The scan reviewed 15 changed files and recorded zero
  findings (scan `ab320995-27a7-4417-8bdc-17bed9db42a6`). The reviewer noted an
  unconfirmed resource concern: caller-supplied `Authentication` is deep-cloned
  before encoder limits apply; no remote path or exploitability was established.
- Draft pull request: [#118](https://github.com/NeverWe1come/KMIPKit/pull/118),
  base `feature/KMIPKIT-0049-managed-objects`, head
  `feature/KMIPKIT-0050-parameters-auth-client`. The original PR-open head was
  `4f23fba09f791dbbd7f510188151a3683fef1dfb` (historical).
- ADR-0012 Decision 6 still requires hosted CI to run and pass these exact
  candidate-callsite tests before the production route can be merged, enabled,
  or released. CI was not queried under the maintainer's instruction during
  the billing issue, so this remains an integration gate despite the local
  pass.
- This evidence does not claim profile conformance, interoperability, human
  security approval, or release readiness.

## Current PR metadata and verification basis (2026-11-07)

- At the initial audit snapshot before either evidence refresh, the #118
  worktree was clean at `ecbd9c978c625ba3648e49f74149287b54ae6c09`, matching
  its local `origin` ref and `gh pr view 118` head. GitHub reported PR #118
  `OPEN`, draft, with base ref OID
  `28d209fd0a159376a8b7ca2b58c71704b37cc9a2` and head ref OID
  `ecbd9c978c625ba3648e49f74149287b54ae6c09`.
- PR #116's documentation-only evidence refresh was pushed as
  `77b3d91ca2dd5597e2eb3a8e89bef957115bd537`. This branch incorporates that
  parent through merge commit `79277e78f7484196ae2dd97c4d5d30ee4745a298`
  (parents: initial #118 head `ecbd9c9…` and refreshed #116 head `77b3d91…`).
  At this pre-push snapshot, the worktree is at that merge commit; GitHub still
  reports the not-yet-pushed #118 head `ecbd9c9…` and base ref OID `28d209fd…`.
- The full workspace verification listed above was recorded on the original
  #118 source snapshot `4f23fba09f791dbbd7f510188151a3683fef1dfb`. The
  0050-owned implementation and test paths are unchanged from that snapshot.
  After predecessor synchronization to `28d209fd0`, the focused ADR-0012
  candidate-callsite tests, formatting, and diff checks were rerun; the full
  workspace suite and coverage were not rerun on the combined stack.
- Compared with the original #118 source snapshot, the additional source/test
  paths are predecessor-owned: `crates/kmipkit-protocol/src/profile.rs`,
  `crates/kmipkit-protocol/tests/credential_contract.rs`,
  `crates/kmipkit-protocol/tests/support/profile_context_unit.rs`,
  `crates/kmipkit-protocol/tests/support/profile_unit.rs`, and
  `crates/kmipkit-protocol/tests/unit/get_attribute_list_tests.rs`. They are
  covered by the #049 workspace verification recorded against source snapshot
  `0a5f661c9a7964825dab597d4472f8edee2cca69`; comparison from that snapshot to
  #049's audited head `28d209fd0` contains documentation/evidence changes only.
- Since the previous #118 audit head `c30f3d4b1f013c40c3886a7aff2b32de4a128400`,
  the stack/evidence updates changed only documentation files, including the
  merge above. This refresh also changes documentation only; no #050
  implementation, test, or generated path is changed by it.
- Remote CI checks were not queried under the maintainer's billing instruction.
