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

For Python dataclasses, the scanner resolves only statically imported
`dataclasses.dataclass` decorators and literal boolean options. It inventories
generated constructors, representation/equality/order/hash methods, frozen
assignment guards, and dataclass metadata fields. It records the constructor
signature from field annotations and literal defaults without executing class
code or decorators. A dynamic dataclass option produces an issue instead of a
partial inventory.

The scanner's class-member boundary is intentionally narrower than Python's
entire runtime object model. It inventories package-facing constructors,
ordinary public members, context-manager methods, and the implemented public
special-method protocols in `_PYTHON_PUBLIC_DUNDER_METHODS`; it does not
enumerate methods inherited from `object`. The dataclass `__post_init__` hook
is an initialization-time validation/normalization callback, not a package
operation. The private handle base's `__del__` is best-effort cleanup; explicit
`close` and context-manager methods remain the supported lifecycle surface.
The private handle base's `__repr__` is a handle-state diagnostic, not a
public value representation; dataclass-generated public value representations
remain inventoried. Their sensitive inputs are hidden by `repr=False` fields
and are covered by `bindings/python/tests/test_client.py` tests
`test_secret_bearing_client_records_redact_repr` and
`test_endpoint_direct_constructor_enforces_text_shape_and_repr_redaction`.
Public constructors inherited from package-private bases are still observed
and must be reconciled. In particular, observing an inherited raw-handle
constructor does not approve it as a product construction path; it remains a
separate ownership/security review item until resolved.

The report is deterministic and omits timestamps and temporary paths. Exit code
0 from `--check` means every package exactly matches the manifest and every
scanner completed; exit code 1 means a completed report has discrepancies or
incomplete inventory; exit code 2 means an input, build, or tool failure. A
manifest row in state `blocked` still counts as an exact inventory match, but
does not satisfy readiness or parity. T011 requires complete scans and exact
matching; a blocked row cannot compensate for a missing manifest record, stale
record, kind mismatch, duplicate, unsupported export, or traversal issue.

## Historical snapshot

The earlier Windows x86-64 snapshot is checked in at
`specs/043-public-api-bindings/reports/binding-export-report-2026-10-12.json`.
The prior C/Java/Python audit command produced exit code 0 at that checkpoint:

| Package | Scanner complete | Observed | Manifest | Exact matches | Missing | Stale | Kind mismatches | Issues |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| C | yes | 182 | 178 | 178 | 0 | 0 | 0 | 0 |
| Java | yes | 351 | 351 | 351 | 0 | 0 | 0 | 0 |
| Python | yes | 274 | 274 | 274 | 0 | 0 | 0 | 0 |

The C count observes 182 exported paths: 178 manifest declarations and four
approved C-specific exclusions. These include the `struct_size` and
`abi_version` fields of the versioned limits structure plus the diagnostic
functions `kmipkit_last_error_clear` and `kmipkit_last_error_get`. The
diagnostic functions remain in the public C header and native ABI; their
exclusions apply only to the shared cross-language capability inventory. The patched
`aws-lc-sys@0.45.0` vendor source prevents the nine transitive
`aws_lc_0_45_0_jent_*` functions from becoming public ABI symbols while
preserving jitterentropy. CI repeats a C-only header/binary/manifest audit on
Windows, Linux, and macOS using the library already built by the C job.

This snapshot predates the T076 mapping and expanded Python source inventory.
It is historical evidence only and is not the current binding-export gate.
T011 is open while the refreshed report and all current declarations are
reconciled; the Rust export crawler also requires its own exhaustive audit and
QA review before T011 is complete.

The scanner does not decide whether an unexpected native export should be
removed or represented as an API, whether a public declaration needs an
approved owner, or how overload signatures should be keyed. Those decisions
must be made under the approved 0043 contract; until reconciled, the report is
evidence of an open gate and does not close T011.

## Current reconciliation status (2026-10-14)

The checked-in report was regenerated with the pinned audit tool after the
transport facade mapping update (`d576d04f`). The scanner is complete for all
three packages. C and Java match the manifest exactly; Python still has
unclassified exports, so this snapshot is not a passing export gate:

| Package | Scanner complete | Observed | Manifest declarations | Exact matches | Exact exclusions | Missing | Stale | Kind mismatches | Issues | `matchesManifest` |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| C | yes | 264 | 237 | 237 | 27 | 0 | 0 | 0 | 0 | yes |
| Java | yes | 469 | 434 | 434 | 35 | 0 | 0 | 0 | 0 | yes |
| Python | yes | 540 | 415 | 415 | 77 | 48 | 0 | 0 | 0 | no |

The C inventory's 27 paths outside the 237 declarations and Java's 35 paths
outside the 434 declarations have exact approved exclusions. Java's transport
builder, encoding, and timeout exports now have exact manifest classifications.
Python's 77 exact exclusions and 415 declarations account for 492 paths; the
48 remaining gaps include value and dataclass constructors/members, enum
members, error constructors/category exports, context methods, and inherited
handle constructors. Resolve value construction separately from wrappers that
could adopt a native handle; the latter must not permit arbitrary or duplicate
ownership. The audit `--check` command writes the current report and exits 1
because Python has 48 missing classifications; that is the expected gate
result, not a scanner or report-generation failure. Regenerate the report after
each reconciliation batch. T011, T076, T021, and T024 remain open until every
observed export is mapped or has an exact approved exclusion and required
parity/CI evidence is complete.
