# Specification: KMIPKIT-0068 Multi-Language Release Packaging Dry-Run & Pre-Publication Validation

## 1. Context & Purpose
Phase 5 of the KMIPKit roadmap requires reproducible packages, publication pipelines in dry-run mode, and final artifact verification before the 1.0.0 release gate across Crates.io (Rust), C Distribution, Maven Central (Java 17), and PyPI (Python 3.12).
This specification governs:
1. Version synchronization and metadata completeness across all 8 declared `publicPackages` in `specification/api/public-api.json`.
2. Topological packaging and isolated offline compilation verification for the 5 public Rust crates (`kmipkit-ttlv`, `kmipkit-protocol`, `kmipkit-transport`, `kmipkit-client`, `kmipkit`).
3. Complete archive packaging and consumer linking verification for C distribution (`kmipkit-c`) with CMake and pkg-config.
4. Maven Central bundle staging (JAR, native JAR per platform, sources, javadoc, checksums, GPG signature dry-run) and isolated native loader verification.
5. PyPI wheel and sdist building, metadata inspection (PEP 621), FFI symbol auditing, and isolated venv smoke verification.
6. Unified artifact inventory manifest and `SHA256SUMS` generation.

## 2. Requirements & Traceability
- **KMIPKIT-REQ-PKG-001**: Version Synchronization: All packaging files (`specification/api/public-api.json`, `Cargo.toml`, `bindings/c/package.toml`, `bindings/java/pom.xml`, `bindings/java/build.gradle.kts`, `bindings/python/pyproject.toml`) MUST match `releaseLine` ("1.0.0").
- **KMIPKIT-REQ-PKG-002**: Rust Crates Dry-Run: Public crates MUST be packageable via `cargo package` and verifiable in an isolated directory offline. Non-public crates MUST have `publish = false`.
- **KMIPKIT-REQ-PKG-003**: C Distribution Dry-Run: Packaged archive MUST include `kmipkit.h`, native library, CMake config, and pkg-config file, validated by building an isolated C consumer.
- **KMIPKIT-REQ-PKG-004**: Maven Central Dry-Run: Must produce standard bundle with POM, primary JAR, platform native JAR, sources JAR, javadoc JAR, checksums, and ASC signatures.
- **KMIPKIT-REQ-PKG-005**: PyPI Dry-Run: Must build valid wheel and sdist, audit FFI symbols, and pass isolated offline smoke execution.
- **KMIPKIT-REQ-PKG-006**: Automated CI Pipeline: Workflow `.github/workflows/release-dry-run.yml` with pinned commit SHAs MUST execute these verifications on Linux, macOS, and Windows.
