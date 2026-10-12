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

For a failed report with exact `missingFromManifest` paths, the separately
reviewable reconciliation utility can add blocked rows to the canonical
manifest input. It never generates signatures, mappings, or exclusions:

```text
python -B tools/api_manifest/reconcile_rust_exports.py --report rust-export-report.json --write
```

Regenerate the report and run `generate.py --check` afterward. The Rust audit
remains a failing gate while it reports unresolved issues.

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

Trait implementation members and members projected through transparent type
aliases are inventoried when rustdoc provides enough identity and generic
information to establish reachability and applicability. Their paths include
the qualified owner and trait/member identity; alias projections use the
reachable public alias path. This includes derived, external-trait, blanket,
provided, and default associated members when the crawler can prove the exact
applicable implementation. Unknown targets or predicates that cannot be
proven produce an issue tied to the exact implementation and reachable owner;
they are never silently treated as complete. Unresolved external re-exports
and unsupported reachable kinds likewise produce issue records with the exact
available paths and source locations.

Any unresolved issue makes `complete` false and makes `--check` fail, even when
all observed paths match the manifest. The current crawler still reports
unresolved implementation predicates, so T011 remains open. C, Java, and
Python are outside this runner's scope.

## Exact comparison

`matched` lists paths whose package, name, and kind match a manifest record.
`missingFromManifest` lists compiler-observed paths absent from both manifest
declarations and exact approved exclusions. `notObserved` lists manifest
records absent from the observed inventory. `kindMismatches` lists
exact name matches with inconsistent kinds. Duplicate manifest names are
rejected. A `blocked` manifest record counts as an inventory match while
remaining blocked for readiness and parity.

## Current snapshot and remaining reconciliation

On **2026-10-12**, with the current continuation branch Rust sources, Windows
x64, Rust 1.94.0 and all features, the report found:

| Published package | Observed paths | Manifest rows | Exact matches | Missing from manifest | Not observed | Issues |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| `kmipkit-ttlv` | 331 | 331 | 331 | 0 | 0 | 0 |
| `kmipkit-protocol` | 5,806 | 5,806 | 5,806 | 0 | 0 | 31 |
| `kmipkit-transport` | 365 | 365 | 365 | 0 | 0 | 0 |
| `kmipkit-client` | 1,031 | 1,031 | 1,031 | 0 | 0 | 0 |
| `kmipkit` | 1,915 | 1,915 | 1,915 | 0 | 0 | 8 |
| **Total** | **9,448** | **9,448** | **9,448** | **0** | **0** | **39** |

There were no kind mismatches or approved Rust exclusions. All 39 issues are
`unresolved-rustdoc-implementation-predicate` (31 in `rust-kmipkit-protocol`
and 8 in `rust-kmipkit`); each identifies the implementation and reachable
owner whose applicability could not be proved. The crawler emits exact trait
implementation and alias paths for cases it can resolve. The 7,681 previously
missing declarations now have exact blocked manifest rows without language
mappings or API signatures. `matchesManifest` is true, but `complete` is false
and `--check` exits 1 because the 39 predicate issues remain. Counts are a
dated source snapshot, not a coverage percentage or permanent expected test
result; rerun after Rust or manifest changes.

The full deterministic JSON from this snapshot is checked in at
`specs/043-public-api-bindings/reports/rust-export-report-2026-10-12.json`.
Regenerate the report after any Rust export or manifest change; the timestamped
path identifies the source snapshot and is not a generated manifest output.

The JSON report lists each issue requiring predicate review. Its declarations
match all 9,448 Rust manifest paths by canonical package, path, and kind. The
reconciliation tool adds blocked rows only for exact paths in
`missingFromManifest`; it inherits a candidate owner only from an exact
manifest owner path, otherwise uses the already blocked `public_binding_surface`
candidate. It does not add signatures, mappings, or approved exclusions.

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
