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
python -B tools/api_manifest/audit_binding_exports.py --repo-root . --report specs/043-public-api-bindings/reports/binding-export-report-2026-10-28.json --check
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
and must be reconciled. Observing an inherited raw-handle constructor does not
approve it as a product construction path. At the current T076 checkpoint,
native wrapper constructors require a private identity token supplied by the
internal handle factory; calls that try to adopt a raw CFFI handle directly
fail before the wrapper is initialized. The 29 exact observed `.__init__`
paths are individually excluded as internal ownership plumbing; each wrapper
type and public construction/factory operation remains separately inventoried
and mapped or explicitly blocked.

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

## Current reconciliation status (2026-10-28)

The current report at
`specs/043-public-api-bindings/reports/binding-export-report-2026-10-22.json`
was regenerated by the pinned audit tool on Windows after the T037 Get Attribute
List slice, T045, T050, T054–T057, T062–T069, T076–T078, and current manifest
reconciliation. The 2026-10-20 report and earlier dated reports remain
historical snapshots. All three
scanners are complete, and every observed export has an exact declaration or
approved exact exclusion:

| Package | Scanner complete | Observed | Manifest declarations | Exact matches | Exact exclusions | Missing | Stale | Kind mismatches | Issues | `matchesManifest` |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| C | yes | 461 | 424 | 424 | 37 | 0 | 0 | 0 | 0 | yes |
| Java | yes | 988 | 776 | 776 | 212 | 0 | 0 | 0 | 0 | yes |
| Python | yes | 1,338 | 755 | 755 | 583 | 0 | 0 | 0 | 0 | yes |

The Java audit accounts for the exact JNI bridge method
`NativeExtensionRegistry.clientExecuteWithOptions` and the generated record
methods `RequestOptions.equals`, `hashCode`, and `toString`. The bridge method
is excluded as minimal raw-handle plumbing; the typed `Client.executeWithOptions`
capability remains mapped. Record boilerplate exclusions apply only to those
three exact paths. New exact JNI bridge exclusions cover the T062–T069
operations; operation-specific typed facades remain inventoried. Exact
response `toString` exclusions are limited to redacted diagnostic methods, and
the Validate overload signatures are listed explicitly. Six Python declaration
kinds now match the source scanner's function/field classification. The 29 Python
native-wrapper initializers remain exact
exclusions because a mandatory private identity token restricts them to the
internal handle factory; wrapper types and public factories remain separately
reconciled. T055 and T078 add exact exclusions for the Java `NativeExtensionRegistry`
handle-kind discriminator and raw-handle methods; their typed Logout and Usage
Allocation facades remain inventoried as product mappings. The exact
`UsageAllocationContext.toString` exclusion is limited to its fixed
`identifier=redacted` diagnostic summary, which the TLS integration test checks.

The semantic transport review maps per-exchange timeout `DEFAULT` to the Rust
default policy while preserving bounded zero and unbounded as distinct values.
Certificate chains, private keys, and trust certificates cover PEM/DER from
memory or selected files; CRLs cover PEM/DER from memory only. C `encoding`,
`source_kind`, nested byte-span `data`, slice counts, and the exact versioned
configuration carrier are classified as ABI representations while the parent
configuration fields retain the semantic mappings and generated C layout.
`endpoint_port` is conditional on RAW_TLS; HTTPS carries its optional port in
the endpoint URI. The current C/Java/Python binding scan is structurally exact:
all three scans are complete with zero missing, stale, kind-mismatched, or
otherwise unclassified exports. This evidence closes the binding-export part
of T011 for this Windows x86-64 snapshot. The Rust export audit and manifest
generator checks are tracked in their own T011 evidence; end-to-end language
parity and offline package-loading gates remain assigned to T029/T071 and
T028/T072. Regenerate the current report after each binding-export
reconciliation batch.
