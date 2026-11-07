# KMIPKIT-0051 synchronization reports

These snapshots belong to the combined 0051 source tree after synchronizing
with the final 0050 predecessor at
`09244e62860ae581484e55514b8aa245c6429d15`. Reports belonging to 0047, 0048,
0049, and 0050 remain historical evidence and are not overwritten.

- `rust-export-report-pre-sync-2026-11-06.json` preserves the original 0051
  Rust audit previously stored at the shared 0043 report path.
- `rust-export-report-2026-11-07.json` inventories the combined source against
  the merged manifest, including the 930 blocked validation declarations.
- `shared-rust-inventory-2026-11-07.json` explicitly uses that 0051 Rust
  snapshot as its input.
- `binding-export-report-2026-11-07.json` inventories C, Java, and Python
  exports against the same combined manifest.

Reproduction from the repository root:

```text
python -B tools/api_manifest/audit_rust_exports.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json --check
python -B tools/api_manifest/reconcile_rust_exports.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json
python -B tools/api_manifest/reconcile_0050_parameters_auth_client.py --report specs/051-pre-send-validation/reports/rust-export-report-2026-11-07.json
python -B tools/api_manifest/generate.py --check
python -B tools/api_manifest/audit_shared_rust_inventory.py --check
python -B tools/api_manifest/audit_binding_exports.py --report specs/051-pre-send-validation/reports/binding-export-report-2026-11-07.json --check
```

Both reconciliation commands make no changes for this snapshot. The
reconciliation added no aliases, declarations, mappings, exclusions, or
readiness claims beyond the reviewed predecessor and existing 0051 inputs.
The 0051 validation declarations remain blocked for
cross-language reconciliation under the approved follow-up scope. These
reports do not establish hosted CI, coverage, interoperability, certification,
or release readiness.
