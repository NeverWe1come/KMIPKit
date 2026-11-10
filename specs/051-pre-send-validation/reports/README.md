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
- `binding-export-report-2026-11-07.json` inventories C, Java, and Python
  exports against the same combined manifest. The current exact declaration
  counts are C 1,733, Java 1,906, and Python 2,025, with no mismatches.

Reproduction from the repository root:

```text
python -B tools/api_manifest/audit_rust_exports.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json --check
python -B tools/api_manifest/reconcile_rust_exports.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json
python -B tools/api_manifest/reconcile_0050_parameters_auth_client.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json
python -B tools/api_manifest/generate.py --check
python -B tools/api_manifest/audit_shared_rust_inventory.py --check
python -B tools/api_manifest/audit_binding_exports.py --report specs/051-pre-send-validation/reports/binding-export-report-2026-11-07.json --check
```

The Rust export `--check` and shared inventory `--check` currently exit
nonzero because 4,199 routes remain missing from the whole-tree 0043 manifest.
The scanner itself is complete and reports no kind, conditional, or scope
issues. Those routes require owner-by-owner classification; they are not
attributed wholesale to 0051 or 0053. The 0051 validation declarations remain
blocked for cross-language reconciliation under the approved follow-up scope.
The binding audit and generated-manifest check pass independently. These
reports do not establish hosted CI, coverage, interoperability, certification,
or release readiness.
