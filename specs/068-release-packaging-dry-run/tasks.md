# Tasks: KMIPKIT-0068 Multi-Language Release Packaging Dry-Run

- [ ] `TASK-068-001`: Author unit tests for release packaging metadata, crate validation, and workflow contracts in `tools/release_packaging/tests/test_release_packaging.py` [Red].
- [ ] `TASK-068-002`: Implement `tools/release_packaging/dry_run.py` CLI supporting `verify-metadata`, `stage-rust`, `stage-c`, `stage-java`, `stage-python`, and `assemble-manifest` [Green].
- [ ] `TASK-068-003`: Complete packaging metadata in `Cargo.toml`, `crates/*/Cargo.toml`, `bindings/java/pom.xml`, `bindings/java/build.gradle.kts`, and `bindings/python/pyproject.toml` [Green].
- [ ] `TASK-068-004`: Author `.github/workflows/release-dry-run.yml` with pinned commit SHAs, minimal permissions, and multi-platform staging matrix [Green].
- [ ] `TASK-068-005`: Execute local automated tests and offline dry-runs across metadata, crates, C archive, Java bundle, and Python packages [Refactor / Verify].
- [ ] `TASK-068-006`: Open Draft PR on GitHub and record verification evidence in PR summary.
