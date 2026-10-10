# Quickstart: Review and Verify the Public API Contract

This guide is for maintainers implementing approved KMIPKIT-0043. It describes local review checks, not product API usage.

## 1. Inspect scope and dependencies

1. Read `spec.md`, `contracts/manifest-v2.md`, and `contracts/parity-matrix.md`.
2. Check `contracts/operation-dependencies.md` before adding an operation mapping.
3. If an operation is blocked, first obtain an approved in-scope operation specification; do not infer a future specification number or signature from another worktree.
4. Confirm the manifest's eight logical public package entries against package metadata. Audit every exported Rust/C/Java/Python declaration; record an exact reviewed exclusion for any deliberately unsupported symbol. Only ADR-0003/ADR-0014's bounded Rust transport contract may use the `rust-only-adr` parity scope.

## 2. Validate the manifest

From the repository root, use the existing tool documented in `docs/development/api-manifest.md`:

```text
python tools/api_manifest/generate.py --check
python -m unittest discover -s tools/api_manifest/tests -v
```

Before release packaging, also run:

```text
python tools/api_manifest/generate.py --check --require-complete
```

During manifest implementation, validate the v2 schema, exact package/export inventory, catalog cross-check, approval sources, and parity scopes before regenerating files. Do not add a network fetch or manually edit generated output.

## 3. Verify one capability in each language

For an approved shared capability or operation:

1. Follow its canonical Rust declaration and stable requirement IDs.
2. Compare its C ABI mapping, including fixed-width values, ownership, release function, and errors.
3. Exercise the same parity fixture through Java 17/JNI and Python 3.12/CFFI.
4. Verify ordering, unknown values, defaults, failures, redaction, and cleanup—not only method-name presence.

## 4. Verify native packaging

Build and load the Java/Python consumer packages locally, with network access unavailable to the test process. Confirm the native library and ABI match and that a failure exposes an actionable redacted diagnostic.

The full implementation PR must also follow repository-required Red/Green/Refactor commits, traceability, platform checks, coverage gates, generated-file check, and security review.
