# KMIPKIT-0050 Implementation Evidence

## Branch and scope

- Branch: `feature/KMIPKIT-0050-parameters-auth-client`.
- Base: `feature/KMIPKIT-0049-managed-objects` (PR #116), as recorded in
  `approval-record.md`.
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
- `cargo clippy -p kmipkit-client -p kmipkit-protocol --all-targets --all-features -- -D warnings`.
- `cargo test --workspace --all-features`.
- `cargo llvm-cov --workspace --all-features --text` (full workspace tests and
  coverage instrumentation completed successfully).
- `cargo test -p kmipkit-protocol --all-features --test cryptographic_parameters_public`
  (9 tests passed).
- Focused direct-method parity and option-forwarding tests passed.
- `python -m unittest tools.api_manifest.tests.test_reconcile_0050_parameters_auth_client`
  (3 tests passed).
- `python tools/api_manifest/generate.py --check` (12 generated outputs current).
- `python tools/api_manifest/audit_rust_exports.py --check` (complete, matches
  manifest, no missing or unobserved Rust paths, zero issues).
- `python scripts/test_user_guide_examples.py` (22 Rust guide examples compiled).
- Manifest reconciler dry run: zero declarations and exclusions left to add.

Windows line coverage from the full workspace run:

| Scope | Covered / executable lines | Result |
|---|---:|---:|
| Changed executable Rust lines | 741 / 756 (98.02%) | Meets 95% |
| `kmipkit-ttlv` | 748 / 761 (98.29%) | Meets 95% |
| `kmipkit-protocol` | 23,417 / 26,025 (89.98%) | **Below 95% project gate** |
| `kmipkit-client` | 6,503 / 6,756 (96.26%) | Above 95% |
| `kmipkit-ffi` | 13,019 / 15,047 (86.52%) | Meets 85% |
| Workspace total | 49,280 / 54,543 (90.35%) | Meets 90% |

The protocol crate's aggregate coverage is below the repository's 95% gate.
The changed executable lines in this feature exceed 95%, but this does not
clear the pre-existing protocol-wide deficit. Keep the PR in draft until the
protocol gate is met or the maintainer records an applicable exception.

Only Windows has been exercised locally. Linux and macOS checks and hosted CI
remain unverified; the repository's current GitHub billing issue may prevent
hosted CI from running. The Windows `aws-lc-sys` build emitted non-fatal
environment warnings about NASM and an optional compiler probe.

## Reviews and publication

- Independent security and QA review: pending.
- Draft pull request: pending creation and verification.
- This evidence does not claim profile conformance, interoperability, human
  security approval, or release readiness.
