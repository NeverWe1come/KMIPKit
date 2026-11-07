# KMIPKIT-0050 synchronization reports

These reports belong to the combined 0050 source tree after synchronization
with the 0049 predecessor. The reports in the 0047, 0048, and 0049 feature
folders remain historical snapshots and are not overwritten.

- `rust-export-report-pre-reconciliation-2026-11-07.json` inventories the merged
  source against the complete incoming 0049 manifest. Its 226 missing routes
  are the 184 declarations and 42 trait exclusions owned by 0050.
- `rust-export-report-post-sync-2026-11-07.json` checks the same runtime source
  against the reconciled 0050 manifest.
- `shared-rust-inventory-post-sync-2026-11-07.json` uses that current 0050 Rust
  report as its explicit input.
- `binding-export-report-post-sync-2026-11-07.json` inventories the C, Java,
  and Python surfaces against the same combined manifest.

Reproduction from the repository root:

```text
python -B tools/api_manifest/audit_rust_exports.py --report specs/050-parameters-auth-client/reports/rust-export-report-post-sync-2026-11-07.json --check
python -B tools/api_manifest/reconcile_0050_parameters_auth_client.py --report specs/050-parameters-auth-client/reports/rust-export-report-post-sync-2026-11-07.json
python -B tools/api_manifest/generate.py --check
python -B tools/api_manifest/audit_shared_rust_inventory.py --check
python -B tools/api_manifest/audit_binding_exports.py --report specs/050-parameters-auth-client/reports/binding-export-report-post-sync-2026-11-07.json --check
```

The reconciler reads reviewed manifest inputs and matches exact operation
constructor routes; metadata types, metadata fields, and similarly prefixed
routes cannot stand in for those constructors. These reports do not establish
four-language typed-parameter or Authentication execution parity, hosted CI,
coverage aggregation, interoperability, or release readiness.
