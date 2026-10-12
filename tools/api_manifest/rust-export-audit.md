# Rust public declaration audit

`audit_rust_exports.py` supports KMIPKIT-0043-FR-020/FR-021 and the Rust portions
of T011/T013. It discovers declarations from compiler output independently of
the manifest, then compares exact package, export path, and declaration kind.
It does not edit the manifest, approve an owner, verify language parity, or
close either task.

## Reproduce the report

The exact Rust **1.94.0** toolchain and the checked-in locked dependencies must
already be available locally. Python uses only its standard library. Run from
the repository root:

```text
python -B -m unittest tools.api_manifest.tests.test_rust_exports -v
python -B tools/api_manifest/audit_rust_exports.py --repo-root . --report rust-export-report.json --check
```

The runner checks `rustup toolchain list` before invoking any toolchain proxy,
rejects a missing exact compiler, and never installs one. For each of the five
published Rust crates it runs:

```text
cargo +1.94.0 rustdoc --locked --offline --all-features --lib -p <package> -- -Z unstable-options --output-format json --document-private-items --document-hidden-items
```

The child process alone receives `RUSTC_BOOTSTRAP=1`, enabling that installed
compiler's rustdoc JSON interface; library source and the global toolchain are
unchanged. JSON format **57** is required. Builds use a fresh temporary target
directory, which is removed after reading the five documents. The runner
clears inherited Rust/rustdoc flag and executable overrides. This run covers
the local host target with all Cargo features; additional supported platform
and feature configurations still need their own audits.

The report omits timestamps and absolute build-directory names, normalizes
source separators, and orders records independently of input JSON object and
manifest record order. Standard output contains the JSON unless `--report`
selects a file. Exit codes are 0 for a completed report, 1 when `--check` detects
a mismatch or an incomplete audit, and 2 for a build/input failure. A report
without `--check` is an inventory snapshot, never a claim of gate completion.

## Accurate supported subset

Traversal begins at each public crate root and follows public modules,
renamed re-exports, glob re-exports, and cross-crate re-exports resolved through
the five documents. Public items inside private modules become reachable only
through their public re-export. Private and restricted fields/methods remain
excluded even though rustdoc documents them. Public `doc(hidden)` items are
included.

Records cover modules, structs, enums, unions, type aliases, public traits,
functions, constants/statics, macros, enum variants, named and tuple fields,
public inherent methods/constants, and public trait declarations' associated
items. Associated items stay under their owning type or trait rather than
being mistaken for free module functions. Enum payload fields have the
variant's public visibility. Field inventory notation is `Owner::field`;
numeric tuple positions use `Owner::0`. This is declaration identification,
not a Rust expression for accessing an enum's payload.

Each distinct reachable path appears once. Re-export aliases retain a shared
definition identity plus their own exact paths and export-site locations.
Different names therefore remain visible to the manifest comparison, while
duplicate imports and repeated associated-item visits do not increase the
count. Source locations identify original declarations; export locations
identify their re-export sites. Definition identity excludes compiler item
IDs, which can vary between builds. Member identity also includes the original
owning type or trait, keeping macro-expanded methods with equal source spans
distinct from one another.

The following surfaces **remain explicitly unaudited**:

- Trait implementation behavior, including derived implementations, external
  trait members, blanket implementations, and provided trait methods. These
  need a reviewed representation of qualified trait/member paths and
  overlapping or generic implementations. The tool records every observed
  owner with such members as `trait-implementation-members-not-audited`.
- Members inherited through transparent type aliases. The alias itself is
  observed; inherited methods/variants are recorded as
  `type-alias-members-not-audited` until their resolution is implemented.
- Unresolved external re-exports or unsupported reachable item kinds. They
  produce issue records with exact paths and available source locations;
  the tool never invents their declarations.

Any such issue makes `complete` false and makes `--check` fail, even when every
observed path matches the manifest. This tested subset alone cannot close
the full export audit required by FR-021. C, Java, and Python are outside
this runner's scope.

## Exact comparison

`matched` lists paths whose package, name, and kind match a manifest record.
`missingFromManifest` lists compiler-observed paths absent from both manifest
declarations and exact approved exclusions. `notObserved` lists manifest
records absent from this supported observed subset; this does not establish
their absence from unaudited trait or alias surfaces. `kindMismatches` lists
exact name matches with inconsistent kinds. Duplicate manifest names are
rejected. A `blocked` manifest record counts as an inventory match while
remaining blocked for readiness and parity.

## Current snapshot and remaining reconciliation

On **2026-10-12**, with the current continuation branch Rust sources, Windows
x64, Rust 1.94.0 and all features, the report found:

| Published package | Observed paths | Manifest rows | Exact matches | Missing from manifest | Not observed |
| --- | ---: | ---: | ---: | ---: | ---: |
| `kmipkit-ttlv` | 107 | 95 | 95 | 12 | 0 |
| `kmipkit-protocol` | 1,631 | 964 | 958 | 673 | 6 |
| `kmipkit-transport` | 98 | 96 | 96 | 2 | 0 |
| `kmipkit-client` | 651 | 291 | 291 | 360 | 0 |
| `kmipkit` | 785 | 321 | 321 | 464 | 0 |
| **Total** | **3,272** | **1,767** | **1,761** | **1,511** | **6** |

There were no kind mismatches or Rust exclusions. The report also contained
**375 incomplete-scope records**: 365 owners with trait implementation members
and 10 transparent type aliases. Counts are a dated source snapshot, not a
coverage percentage or permanent expected test result; rerun after Rust or
manifest changes.

The full deterministic JSON from this snapshot is checked in at
`specs/043-public-api-bindings/reports/rust-export-report-2026-10-12.json`.
Regenerate the report after any Rust export or manifest change; the timestamped
path identifies the source snapshot and is not a generated manifest output.

The 12 missing TTLV paths are `kmipkit_ttlv::Item::try_clone` and the 11 public
`ValueView` tuple payload fields. The six protocol records not observed use
dot notation for `ExtensionDefinitionAccounting` fields; their source fields
exist but the manifest spelling differs from the consistent `Owner::field`
notation emitted by this auditor. These are exact-name discrepancies to
review, not proof that the fields were removed. Newly integrated operation
types and methods account for many other missing paths; the JSON report lists
every exact path and source location for reconciliation.

## Development verification

The focused tests exercise public/private reachability, aliases, local and
cross-crate glob imports, cycles, field visibility, enum payloads, trait and
inherent association, macros and hidden items, unsupported scope reporting,
schema/version rejection, duplicate names, exact comparison, exclusions,
ordering, and the installed-toolchain guard. Red/Green commits are distinct:

- `62708414`: 16 tests failed because the auditor did not exist.
- `80854e99`: traversal/comparison tests passed and the full offline runner
  produced the mismatches above; `--check` correctly returned 1.
- `90d1b6df`: the installed-toolchain guard regression failed.
- `f110bd7c`: all 17 focused tests passed with the offline guard.
- `f9ddc1d`: a macro-generated member identity regression failed when two
  different types' methods had the same macro source span.
- `9513ee17`: all 18 focused tests passed after retaining original owner paths.

The pinned real compiler established JSON format 57; the initial exploratory
stable compiler's format 61 is deliberately rejected by this runner.
