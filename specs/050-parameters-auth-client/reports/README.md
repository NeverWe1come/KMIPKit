# KMIPKIT-0050 synchronization reports

These reports belong to the combined 0050 source tree after synchronization
with the 0049 predecessor. The reports in the 0047, 0048, and 0049 feature
folders remain historical snapshots and are not overwritten.

- `rust-export-report-pre-reconciliation-2026-11-07.json` inventories the merged
  source against the complete incoming 0049 manifest. Its 226 missing routes
  are the 184 declarations and 42 trait exclusions owned by 0050.
- `rust-export-report-post-sync-2026-11-07.json` is the current full-source
  inventory snapshot after the 0049 merge. The scan is complete, but the
  manifest is not: it records 27,234 observed routes, 7,037 exact matches,
  15,998 exact exclusions, and 4,199 routes still missing from the manifest.
  The missing set includes APIs outside 0050's scope (for example, the
  server-initiated `ClientListener` surface). It is a whole-tree manifest gap
  that must be classified by owning feature; it is not attributable wholesale
  to KMIPKIT-0050 or KMIPKIT-0053. KMIPKIT-0053 separately owns four-language
  parity for 0050's APIs and is the planned final phase-5 reconciliation point.
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

`audit_rust_exports.py --check` currently exits with status 1 because
`matchesManifest` is false while 4,199 routes remain unreconciled; the scanner
itself is complete and reports no kind, conditional, or scope issues. The
paired shared Rust inventory check also remains blocked by that missing set.
The pre-reconciliation snapshot records exactly 226 0050-owned paths (184
declarations and 42 trait exclusions); none of those paths remain missing in
the current full-source report. Do not treat the earlier 0050 reconciliation
snapshot as proof that the whole public Rust surface is fully reconciled, or
assign every remaining route to KMIPKIT-0053 without owner-by-owner review.

The reconciler reads reviewed manifest inputs and matches exact operation
constructor routes; metadata types, metadata fields, and similarly prefixed
routes cannot stand in for those constructors. These reports do not establish
four-language typed-parameter or Authentication execution parity, hosted CI,
coverage aggregation, interoperability, or release readiness.
