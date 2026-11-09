# Tasks: KMIPKIT-0068 Multi-Language Release Packaging Dry-Run

- [x] `TASK-068-001`: Author unit tests for release packaging metadata, crate validation, and workflow contracts in `tools/release_packaging/tests/test_release_packaging.py` [Red].
- [x] `TASK-068-002`: Implement `tools/release_packaging/dry_run.py` CLI supporting `verify-metadata`, `stage-rust`, `stage-c`, `stage-java`, `stage-python`, and `assemble-manifest` [Green].
- [x] `TASK-068-003`: Complete packaging metadata in `Cargo.toml`, `crates/*/Cargo.toml`, `bindings/java/pom.xml`, `bindings/java/build.gradle.kts`, and `bindings/python/pyproject.toml` [Green].
- [x] `TASK-068-004`: Author `.github/workflows/release-dry-run.yml` with pinned commit SHAs, minimal permissions, and multi-platform staging matrix [Green].
- [x] `TASK-068-005`: Execute local automated tests and offline dry-runs across metadata, crates, C archive, Java bundle, and Python packages [Refactor / Verify].
- [ ] `TASK-068-007`: Add proxy provider catalog CLI and reject unknown or feature-disabled provider selection without mock fallback [Red/Green/Refactor].
- [ ] `TASK-068-008`: Pin proxy container base images by digest, parameterize provider feature builds, and exclude local secrets/build output from Docker context [Red/Green/Refactor].
- [ ] `TASK-068-009`: Add a dedicated proxy packaging workflow to build and smoke-check the universal image and ten provider images on Linux x86-64 [Red/Green/Refactor].
- [ ] `TASK-068-010`: Lint, render all Helm image/provider variants, package the chart, and assemble deterministic inventory/checksums [Red/Green/Refactor].
- [ ] `TASK-068-006`: Open Draft PR on GitHub and record verification evidence in PR summary.
