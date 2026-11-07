# Feature Specification: Standalone Java KMIP 2.1 Validator

**Feature Branch**: `feature/KMIPKIT-0061-interop-java-validator`
**Status**: Approved for implementation by maintainer plan authorization
**Owning identifier**: KMIPKIT-0061

Provide a standalone, independent Java 21 Gradle project (`interop/java-kmip-validator/`)
that executes the exhaustive KMIP 2.1 test catalog against arbitrary KMIP servers
using capability matching, anti-collision isolation, and graphical report generation.

## Purpose

1. Isolated standalone Java Gradle project with independent build.
2. Native parser for server capabilities and connection profiles.
3. Capability matching engine identifying executable vs unsupported (`➖`) cases.
4. AttemptContext anti-collision isolation (`setup` -> `target` -> `teardown`).
5. Decoupled `KmipDriver` interface (mock driver for standalone execution).
6. Native HTML/Markdown/ANSI graphical matrix reporter (`✅` / `❌` / `➖`).
7. Isolated GitHub Actions workflow `.github/workflows/interop-java.yml`.

## Requirements

| ID | Requirement | Normative source / project decision | Verification |
|---|---|---|---|
| KMIPKIT-0061-FR-001 | The Java validator SHALL use an independent build.gradle.kts and build without requiring bindings/java build tasks. | Project build isolation decision | `./gradlew test` in `interop/java-kmip-validator/` |
| KMIPKIT-0061-FR-002 | The Java suite SHALL parse server capabilities JSON and test catalog to classify cases. | Project interoperability architecture decision | Unit test executable in `src/test/java/org/kmipkit/validator/ValidatorTest.java` |
| KMIPKIT-0061-FR-003 | The Java runner SHALL generate unique names per attempt and enforce setup/teardown steps. | Project anti-collision decision | Unit test verification in `ValidatorTest.java` |
| KMIPKIT-0061-FR-004 | The CLI SHALL support `run` and `plan` commands outputting `results.json`, `report.html`, and `report.md`. | Project CLI & visual reporting decision | CLI invocation test in `ValidatorTest.java` |
| KMIPKIT-0061-FR-005 | A dedicated GitHub Actions workflow `.github/workflows/interop-java.yml` SHALL validate Gradle compilation and tests for the Java validator. | Project CI automation decision | Local test check and workflow file presence |
