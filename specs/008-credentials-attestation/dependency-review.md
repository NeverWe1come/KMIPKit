# Dependency Review: KMIPKIT-0008

**Reviewed**: 2026-10-06  
**Scope**: Test-only dependencies for deterministic TTLV and credential-model properties.  
**Disposition**: No new dependency or Cargo manifest change is required.

## Existing dependency

`quickcheck = 1.1.0` is already pinned as a dev dependency with default features
disabled in `kmipkit-protocol` and `kmipkit-client`. Existing tests in both
crates use it. KMIPKIT-0008 will reuse this dependency; it will not add
`proptest`, `quickcheck_macros`, another direct test dependency, or a production
dependency. Its existing `rand` and `getrandom` dependencies remain transitively
in the test graph.

| Review area | Finding |
|---|---|
| License | Upstream publishes `Unlicense OR MIT`; the MIT option is compatible with this Apache-2.0 project. |
| Maintenance | Upstream's public crate metadata records QuickCheck 1.1.0 as published on 2026-02-10. The repository contains source, tests, CI configuration, and release history. This is evidence of recent maintenance, not a guarantee of future support. |
| MSRV | Upstream declares Rust 1.85.0, below KMIPKit's Rust 1.94 MSRV. |
| Platforms | The library is a Rust test utility and is used only in dev-dependencies. The resolved graph includes `rand 0.10.3` and `getrandom 0.4.3`; the complete all-target lockfile graph also contains target-specific `r-efi 6.0.0`. Linux, Windows, and macOS compatibility is verified by KMIPKit's platform CI rather than inferred solely from metadata. |
| Feature and transitive footprint | Both current manifest entries disable QuickCheck's default `regex` and `use_logging` features. `cargo tree -p kmipkit-protocol --edges normal,dev` resolves the test path `quickcheck 1.1.0 -> rand 0.10.3 -> getrandom 0.4.3`, with `rand_core 0.10.1`, `cfg-if 1.0.5`, and `libc 0.2.190`; the all-target graph additionally includes target-specific `r-efi 6.0.0`. `rand` enables its `sys_rng` path, so disabling QuickCheck defaults does not remove `getrandom`. QuickCheck is absent from production-only dependency edges. These versions are the current lockfile snapshot. |
| Security history | The RustSec advisory index was searched for `quickcheck` on 2026-10-06 and returned no matching advisory; an exact-version OSV query for QuickCheck 1.1.0 also returned no advisory. This is not a claim that undisclosed vulnerabilities cannot exist; release CI must continue auditing the full lockfile. QuickCheck is used only to generate test inputs. |
| Reproducibility | QuickCheck 1.1.0 exposes `Gen::from_size_and_seed` and `QuickCheck::rng`. Property tests can use a fixed seed and explicit case/size limits, but must assert invariants rather than exact generated sequences. QuickCheck documents that generated values can change between releases, and its `SmallRng` implementation is not portable across platforms. A fixed seed therefore repeats cases only within the pinned dependency, toolchain, and target context; it does not promise cross-platform or cross-release sequence identity. |

## Alternatives considered

- **Add `proptest`**: richer strategy and shrinking APIs, but adds a new direct test dependency and lockfile footprint that is not needed for the bounded typed-tree generators here.
- **Hand-written fixed cases only**: avoids the existing QuickCheck-to-`rand`/`getrandom` transitive test graph, but does not satisfy the repository's property-based TTLV round-trip test requirement.
- **Reuse existing QuickCheck**: selected. It already builds in both relevant crates, can use bounded seeded runs and shrinking, and adds no direct dependency or manifest/lockfile change; its existing `rand` and `getrandom` transitive dependencies remain part of the test graph.

## Security and change controls

No Cargo manifest or lockfile change is authorized by this review. If implementation
later demonstrates that another test dependency is necessary, stop before
editing Cargo manifests, update this review with that specific candidate's
license, maintenance/security history, MSRV, supported platforms, transitive
footprint, and alternatives, then obtain independent dependency review.

An independent dependency reviewer inspected this document and the resolved
lockfile graph at `bbc2d83`. The review confirmed that no Cargo manifest change
is needed and requested the target-specific `r-efi` entry, the `SmallRng`
portability limitation, and the distinction between direct and transitive
dependencies. Those points are incorporated above. The review did not run
tests and is not a substitute for release CI's complete lockfile audit.

## Sources

- QuickCheck 1.1.0 upstream manifest and MSRV: <https://github.com/BurntSushi/quickcheck/blob/master/Cargo.toml>
- QuickCheck 1.1.0 crate metadata and release date: <https://docs.rs/crate/quickcheck/1.1.0>
- QuickCheck seeded generator API: <https://docs.rs/quickcheck/1.1.0/quickcheck/struct.Gen.html>
- QuickCheck runner RNG API: <https://docs.rs/quickcheck/1.1.0/quickcheck/struct.QuickCheck.html>
- RustSec advisory index: <https://rustsec.org/advisories/>
