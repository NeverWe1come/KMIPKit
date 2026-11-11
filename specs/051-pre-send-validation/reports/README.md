# KMIPKIT-0051 synchronization reports

These snapshots belong to the combined 0051 source tree after synchronizing
with the updated 0050 predecessor at
`e770f9e2296568c57de4543fc57090d7f629d0ab`. Reports belonging to 0047, 0048,
0049, and the earlier 0050 synchronization remain historical evidence and are
not overwritten.

The 2026-11-09 KMIPKIT-0047 correction merged into 0051 changes only private
version-gate behavior, tests, and documentation; it adds no public export, so
the Rust and binding export snapshots remain applicable to this branch.

- `rust-export-report-pre-sync-2026-11-06.json` preserves the original 0051
  Rust audit previously stored at the shared 0043 report path.
- `rust-export-report-2026-11-07.json` is regenerated against the combined
  source. It contains all 930 0051 validation declarations, including 902
  blocked 0043 candidates, 24 0009 candidates, and 4 0044 candidates. The
  complete scan currently records 28,164 observed routes, 7,967 exact matches,
  15,998 exact exclusions, and 4,199 unresolved whole-tree routes.
- `shared-rust-inventory-2026-11-07.json` is a historical snapshot. The shared
  audit cannot regenerate a passing current inventory while the 4,199
  whole-tree routes remain missing; do not use that older report as current
  evidence.
- `rust-export-report-2026-11-11.json` is the current compiler audit after the
  KMIPKIT-0093 complete API contract was approved. It records 29,862 observed
  Rust paths, 9,145 matched declarations, 20,717 approved exclusions, and no
  missing paths, stale declarations, kind mismatches, or unresolved conditions.
  The manifest classifies 1,615 product API paths as blocked under KMIPKIT-0093
  for later four-language implementation and records 3,467 standard-library
  trait paths as language-level exclusions.
- `shared-rust-inventory-2026-11-11.json` is generated from the current Rust
  audit. It records 4,759 mapped shared declarations, 20,717 approved
  exclusions, 45 Rust-only declarations, and no unresolved routes.
- `binding-export-report-2026-11-07.json` preserves the historical C, Java,
  and Python inventory from November 7. The current audit is
  `binding-export-report-2026-11-11.json`: all three packages match their
  manifest exactly, with no missing, stale, kind-mismatched, or unresolved
  entries. Its exact matches are C 1,967, Java 2,265, and Python 2,445.

Reproduction from the repository root:

```text
python -B tools/api_manifest/audit_rust_exports.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-11.json --check
python -B tools/api_manifest/reconcile_rust_exports.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-11.json --candidate-spec-id KMIPKIT-0093 --requirement-id KMIPKIT-0093-FR-011
python -B tools/api_manifest/reconcile_0050_parameters_auth_client.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-11.json
python -B tools/api_manifest/generate.py --check
python -B tools/api_manifest/audit_shared_rust_inventory.py --check
python -B tools/api_manifest/audit_binding_exports.py --report specs/051-pre-send-validation/reports/binding-export-report-2026-11-11.json --check
```

The November 7 snapshot retains the earlier 4,199-route gap as historical
evidence. The November 11 report classifies the newly reviewed complete API
surface under KMIPKIT-0093 and passes the exact Rust export inventory checks.
The 0051 validation declarations remain blocked for cross-language
implementation. These reports do not establish hosted CI, coverage,
interoperability, certification, or release readiness.
