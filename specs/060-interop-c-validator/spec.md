# Feature Specification: Standalone C KMIP 2.1 Validator

**Feature Branch**: `feature/KMIPKIT-0060-interop-c-validator`
**Status**: Approved for implementation by maintainer plan authorization
**Owning identifier**: KMIPKIT-0060

Provide a standalone, independent C11 project (`interop/c-kmip-validator/`)
that executes the exhaustive KMIP 2.1 test catalog against arbitrary KMIP servers
using capability matching, anti-collision isolation, and graphical report generation.

## Purpose

1. Isolated standalone C11 project with independent CMake build.
2. Native parser for server capabilities and connection profiles.
3. Capability matching engine identifying executable vs unsupported (`➖`) cases.
4. AttemptContext anti-collision isolation (`setup` -> `target` -> `teardown`).
5. Decoupled `kmip_driver_vtable_t` interface (mock driver for standalone execution).
6. Native HTML/Markdown/ANSI graphical matrix reporter (`✅` / `❌` / `➖`).
7. Isolated GitHub Actions workflow `.github/workflows/interop-c.yml`.

## Requirements

| ID | Requirement | Normative source / project decision | Verification |
|---|---|---|---|
| KMIPKIT-0060-FR-001 | The C validator SHALL use an independent CMakeLists.txt and compile without requiring bindings/c build targets. | Project build isolation decision | `cmake -B build` and `cmake --build build` in `interop/c-kmip-validator/` |
| KMIPKIT-0060-FR-002 | The C suite SHALL parse server capabilities JSON and test catalog to classify cases. | Project interoperability architecture decision | Unit test executable in `tests/test_c_validator.c` |
| KMIPKIT-0060-FR-003 | The C runner SHALL generate unique names per attempt and enforce setup/teardown steps. | Project anti-collision decision | Unit test verification in `tests/test_c_validator.c` |
| KMIPKIT-0060-FR-004 | The CLI SHALL support `run` and `plan` commands outputting `results.json`, `report.html`, and `report.md`. | Project CLI & visual reporting decision | CLI invocation test in `tests/test_c_validator.c` |
| KMIPKIT-0060-FR-005 | A dedicated GitHub Actions workflow `.github/workflows/interop-c.yml` SHALL validate CMake compilation and tests for the C validator. | Project CI automation decision | Local test check and workflow file presence |
