# C, Java, and Python public export audit

`audit_binding_exports.py` supplies the non-Rust evidence required by
KMIPKIT-0043-FR-021 and T011. It independently discovers declarations from the
published C header and ABI library, compiled Java 17 classes, and Python source
exports, then compares exact package ID, export path, and kind with manifest v2.
It does not add manifest rows, approve owners, or make excluded-scope decisions.

## Reproduce

Run from the repository root with Python 3, Cargo, a C build toolchain, JDK 17 or
later, and the platform's native symbol reader (`nm` on Unix):

```text
python -B -m unittest tools.api_manifest.tests.test_binding_exports -v
python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json --check
```

The runner builds `kmipkit-ffi` with `cargo build --locked --offline` into a new
temporary target directory. It reads named exports directly from the PE export
table on Windows, or from the platform shared library with `nm` on Unix. An
exported symbol without the `kmipkit_` prefix, a binary/header mismatch, an
ordinal-only PE export, or an unsupported PE image prevents a complete C audit.
The audit inventories header macros, types, fields, and function prototypes as
well as binary symbols. Temporary files are removed automatically.

Java sources compile with `javac --release 17` into a separate temporary
directory; `javap -protected` inventories public and protected types and members,
including compiler-created record and enum members. Python is parsed without
importing or executing package code. The scanner follows explicit `__all__`,
visible module definitions, imported names, class members, and inherited
members from private helper bases. Unsupported dynamic export expressions,
unresolved imports, duplicate paths, or ambiguous overloads remain issues.

The report is deterministic and omits timestamps and temporary paths. Exit code
0 from `--check` means every package exactly matches the manifest and every
scanner completed; exit code 1 means a completed report has discrepancies or
incomplete inventory; exit code 2 means an input, build, or tool failure. A
manifest row in state `blocked` still counts as an exact inventory match, but
does not satisfy readiness or parity. T011 requires complete scans and exact
matching; a blocked row cannot compensate for a missing manifest record, stale
record, kind mismatch, duplicate, unsupported export, or traversal issue.

## Current snapshot

The current Windows x86-64 snapshot is checked in at
`specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json`.
The complete audit command produced exit code 1, so T011 remains open:

| Package | Scanner complete | Observed | Manifest | Exact matches | Missing | Stale | Kind mismatches | Issues |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| C | no | 192 | 175 | 175 | 15 | 0 | 0 | 18 |
| Java | yes | 329 | 319 | 318 | 11 | 1 | 0 | 0 |
| Python | no | 274 | 262 | 262 | 12 | 0 | 2 | 2 |

The C binary contains nine named `aws_lc_0_45_0_jent_*` exports absent from the
header and outside the stable `kmipkit_` symbol prefix; they are recorded as
observed exports and prevent exact accounting. Six other C missing paths are
five public `KMIPKIT_ERROR_*` macros and the `KMIPKIT_H` include guard. The Java
report identifies the generated `PublicApi` class and members plus the public
record members of `RuntimeMetadata.Version` as missing, and
`org.kmipkit.internal.NativeHandle.run` as stale in the manifest. Python is
missing the root `kmipkit` module, seven error-category attributes, and four
`Version` fields. Its two free-function kinds differ from the manifest, and
`validate_extension_value` has overlapping declarations/manifest rows without
a distinct export path. These are exact outstanding inventory or manifest
reconciliation work, not approved exclusions.

The scanner does not decide whether an unexpected native export should be
removed or represented as an API, whether a public declaration needs an
approved owner, or how overload signatures should be keyed. Those decisions
must be made under the approved 0043 contract; until reconciled, the report is
evidence of an open gate and does not close T011.
