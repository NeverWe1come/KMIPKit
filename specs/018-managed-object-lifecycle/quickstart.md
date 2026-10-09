# Quickstart: KMIP 2.1 Managed-Object State Transitions

This guide becomes executable validation evidence after the approved implementation lands. It is not evidence that KMIPKIT-0018 has already been implemented.

## Prerequisites

- Rust toolchain 1.94 or stable and the repository's pinned workspace dependencies.
- The checked-in pinned OASIS KMIP 2.1 sources and normative catalog; tests must not download source documents.
- Use fake transport for deterministic request/response behavior. A real KMIP server is not required for these focused tests.

## Focused validation

From the repository root, run the final test targets named by `tasks.md`. The implementation must cover protocol request/response models, client execution, malformed responses, errors, Pending correlation, no retry, and redaction. The expected result is that every focused test passes with no network call.

## Normative catalog validation

```powershell
python -B tools/normative_catalog/validate.py --repo-root .
python -B tools/normative_catalog/report.py --check
python -B tools/normative_catalog/check_immutable_sources.py --base-sha 227e3f9104f14494810598d078c013c389c24f8f --repo-root .
```

Expected outcome: the four operation records and three applicable client requirement records are assigned to KMIPKIT-0018, the generated coverage report is current, and the pinned upstream OASIS copies are unchanged.

## Workspace validation

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo llvm-cov --workspace --all-features
```

Expected outcome: formatting, lint, workspace tests, and the repository coverage gates pass on required CI platforms before the draft PR is marked ready for human review.
