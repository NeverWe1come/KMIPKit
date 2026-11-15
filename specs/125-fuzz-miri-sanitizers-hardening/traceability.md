# Traceability: KMIPKIT-0125

| Requirement | Source Reference | Design Element | Verification |
|---|---|---|---|
| KMIPKIT-0125-FR-001 | Profiles §5.4.1, §5.5.1, Spec §6.2 | `fuzz/fuzz_targets/*` | Fuzz campaign execution |
| KMIPKIT-0125-FR-002 | ADR-0017 | `fuzzing` Cargo feature | Manifest & dependency tests |
| KMIPKIT-0125-FR-003 | AGENTS.md §7 | Miri test runner | `cargo +nightly miri test` |
| KMIPKIT-0125-FR-004 | AGENTS.md §8 | CI ASan/UBSan jobs | `.github/workflows/ci.yml` |
| KMIPKIT-0125-FR-005 | Profiles §5.4.1, §5.5.1 | Negative test suites | Unit & integration tests |
| KMIPKIT-0125-FR-006 | Profiles §5.12.3 | `locate_alternative_name_negative.rs` | Unit tests & catalog report |
| KMIPKIT-0125-FR-007 | AGENTS.md §12 | `scripts/run_hardening_campaign.py` | Evidence report verification |
