# Typed-attribute audit evidence

The tracked reports serve different purposes:

- `rust-export-report-post-sync-2026-11-07.json` is the official final Rust
  audit: 21,507 observations, 6,285 exact matches, 15,222 exclusions, and no
  missing routes. It proves final inventory consistency and idempotence, not
  reconstruction of absent 0048 rows.
- `rust-export-report-pre-reconciliation-2026-11-07.json` is a deterministic
  comparison input derived with the official auditor's `compare` function.
  It uses the same compiler observations against the current manifest with
  exactly the 286 typed declarations and 2,072 0048 exclusions removed. It
  therefore records 2,358 missing routes, 5,999 matches, 13,150 exclusions,
  and no stale routes or other audit discrepancies. `matchesManifest` is
  intentionally false for this incomplete baseline.
- `shared-rust-inventory-post-sync-2026-11-07.json` is the final shared API
  inventory, generated and checked by `audit_shared_rust_inventory.py`.

The older untracked `rust-export-report-2026-11-07.json` is not a prerequisite:
it predates the current 0047 reconciliation and contains 3,182 missing routes
and three unobserved declarations. It is preserved as local historical data.

## Reproduce the comparison and reconstruction

Run this Python from the repository root, for example by passing it to
`python -B -`. It regenerates the tracked pre-reconciliation comparison with
the official audit implementation, then invokes the official reconciler
against an isolated temporary baseline. It does not modify the real manifest
or either historical/final audit report.

```python
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile

from tools.api_manifest.audit_rust_exports import compare

root = Path.cwd()
reports = root / "specs/048-typed-attributes/reports"
manifest = json.loads((root / "specification/api/public-api.json").read_text(encoding="utf-8"))
observed = json.loads((reports / "rust-export-report-post-sync-2026-11-07.json").read_text(encoding="utf-8"))
baseline = copy.deepcopy(manifest)
baseline["publicDeclarations"] = [
    row for row in baseline["publicDeclarations"]
    if row.get("capabilityId") != "typed_object_attributes"
]
baseline["approvedExclusions"] = [
    row for row in baseline["approvedExclusions"]
    if row.get("candidateSpecId") != "KMIPKIT-0048"
]
report = compare(observed, baseline)
assert report["counts"]["missingFromManifest"] == 2358
assert report["counts"]["notObserved"] == 0
report_path = reports / "rust-export-report-pre-reconciliation-2026-11-07.json"
report_path.write_text(
    json.dumps(report, indent=2, sort_keys=True, ensure_ascii=False) + "\n",
    encoding="utf-8", newline="\n",
)
with tempfile.TemporaryDirectory(prefix="kmipkit-0048-reconstruction-") as directory:
    temporary_root = Path(directory)
    manifest_path = temporary_root / "specification/api/public-api.json"
    manifest_path.parent.mkdir(parents=True)
    manifest_path.write_text(json.dumps(baseline), encoding="utf-8")
    subprocess.run([
        sys.executable, "-B", "tools/api_manifest/reconcile_0048_typed_attributes.py",
        "--repo-root", str(temporary_root), "--report", str(report_path), "--write",
    ], cwd=root, check=True)
    assert json.loads(manifest_path.read_text(encoding="utf-8")) == manifest
print("Reconstructed 286 declarations and 2072 exclusions exactly.")
```

The reconciler command must print `+286 declarations, +2072 exclusions`.
Running it against the final manifest instead would only check idempotence.
The final post-sync audit remains independently reproducible with:

```text
python -B tools/api_manifest/audit_rust_exports.py --report specs/048-typed-attributes/reports/rust-export-report-post-sync-2026-11-07.json --check
```
