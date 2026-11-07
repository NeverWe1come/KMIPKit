| Requirement ID | Normative Reference / Project Decision | Implementation Path | Verification Reference |
|---|---|---|---|
| KMIPKIT-0060-FR-001 | Project build isolation decision | `interop/c-kmip-validator/CMakeLists.txt` | `cmake --build interop/c-kmip-validator/build` |
| KMIPKIT-0060-FR-002 | Project interoperability architecture decision | `interop/c-kmip-validator/src/c_validator.c::kmip_is_case_supported` | `interop/c-kmip-validator/build/test_c_validator` |
| KMIPKIT-0060-FR-003 | Project anti-collision decision | `interop/c-kmip-validator/src/c_validator.c::kmip_init_attempt_context` | `interop/c-kmip-validator/build/test_c_validator` |
| KMIPKIT-0060-FR-004 | Project CLI & visual reporting decision | `interop/c-kmip-validator/src/main.c`, `src/c_validator.c` | `interop/c-kmip-validator/build/kmip_validator_c run` |
| KMIPKIT-0060-FR-005 | Project CI automation decision | `.github/workflows/interop-c.yml` | `.github/workflows/interop-c.yml` |
