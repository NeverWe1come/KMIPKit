# T008 Property-Test Dependency Review

**Review date:** 2026-10-05  
**Scope:** Test-only deterministic property testing for KMIPKIT-0006. The workspace declares Rust 1.94, Edition 2024. The workspace `Cargo.lock` currently contains neither `proptest` nor `quickcheck`/`rand`; no property-test dependency is present in the checked-in crate manifests.

## Recommendation

Use `quickcheck` 1.1.0 as a crate-local development dependency, with its optional default features disabled:

```toml
[dev-dependencies]
quickcheck = { version = "=1.1.0", default-features = false }
```

Use the public seeded generator and runner configuration for every property:

```rust
QuickCheck::new()
    .rng(Gen::from_size_and_seed(size, 0x4B4D49504B495430))
    .tests(256)
    .quickcheck(property as fn(TestInput) -> bool);
```

Implement the bounded generator and shrinker on a cloneable, test-only input representation, then construct fresh TTLV/model values from it. Encode the plan's size, depth, field-count, identifier, and allocation-valid Tag bounds in that generator; `Gen`'s general size setting alone does not establish these project-specific bounds. Properties should not discard cases, so `.tests(256)` exercises 256 successful cases. Promote any minimized witness to a named fixed regression before refactoring, as the plan requires.

QuickCheck 1.1.0 documents `Gen::from_size_and_seed` as reproducible for equal seeds and `QuickCheck::rng` as the runner's RNG setter. It also states that generated sequences may change between QuickCheck releases. The exact crate-version pin and committed `Cargo.lock` therefore form part of this reproducibility contract; re-review and update named regressions before changing the pin. QuickCheck reports minimized failure witnesses through the test failure and has no external regression-file persistence feature, so it meets the no-external-persistence requirement without configuration. See [Gen](https://docs.rs/quickcheck/1.1.0/quickcheck/struct.Gen.html) and [QuickCheck](https://docs.rs/quickcheck/1.1.0/quickcheck/struct.QuickCheck.html).

## Capability and alternatives

The required capability is a deterministic, bounded generator with shrinking for ordered recursive message data and raw integer/bitmask values, at 256 cases per property. QuickCheck supports local `Arbitrary` implementations and custom shrinkers, a caller-supplied seeded generator, a per-property case count, and useful minimized witnesses. The generator is test-only; it does not constrain protocol inputs or add production code.

| Option | Assessment |
| --- | --- |
| `quickcheck` 1.1.0 | Recommended. The API meets the seed, case-count, custom-generator, and shrink requirements. Disable default `env_logger`/`log` features and do not add `quickcheck_macros`; direct runner calls give explicit seed and case-count control. Its shrinking is less composable than Proptest's strategy/value-tree model, so custom recursive input shrinking is local test code. |
| `proptest` 1.11.0 | Functionally attractive, with composable strategies and integrated shrinking. Do not select this release: its upstream issue tracker currently has an open soundness report against 1.11.0 (see security review below); upstream lists the fixes under “Unreleased.” Reconsider only after a fixed release is published and reviewed. `default-features = false, features = ["std"]` would still enable `std`'s `regex-syntax` and `rand/os_rng` dependencies; disabling default features would remove the default `fork`, `timeout`, and `bit-set` features, but would not resolve the soundness report. |
| Hand-written seeded loop and generator | Avoids a third-party property framework, but makes KMIPKit responsible for maintaining its RNG integration, shrink behavior, and failure reporting. This is more test-support code than the selected dependency for no material benefit here. |

QuickCheck 1.1.0 was published on 2026-02-10. Its upstream CI tests Linux (including the pinned MSRV, stable, beta, and nightly), macOS stable, and Windows MSVC and GNU stable, which covers KMIPKit's named Linux/Windows/macOS target set. The crate declares Rust 1.85.0 MSRV, below KMIPKit's 1.94. The upstream README notes that the effective MSRV can follow the public `rand` dependency; keep the workspace MSRV check in the normal validation workflow. The crate is licensed `Unlicense OR MIT`; selecting MIT is compatible with this Apache-2.0 project. Sources: [release metadata and dependency list](https://docs.rs/crate/quickcheck/1.1.0), [upstream Cargo.toml](https://github.com/BurntSushi/quickcheck/blob/master/Cargo.toml), [upstream README/MSRV policy](https://github.com/BurntSushi/quickcheck/blob/master/README.md), and [upstream CI matrix](https://github.com/BurntSushi/quickcheck/blob/master/.github/workflows/ci.yml).

## Maintenance, security, and dependency cost

QuickCheck's latest release is 1.1.0 (2026-02-10); the preceding 1.0.3 release was in 2021, so release cadence has been sparse. Its current upstream CI covers a broad compiler/platform matrix. GitHub's upstream security-advisory page showed no published advisories on the review date. That is an advisory-record check, not a guarantee that the crate or its dependency is defect-free. QuickCheck's documented `Arbitrary` strategy behavior can change in compatible releases, which is another reason for the exact pin and explicit regressions. Sources: [release history](https://docs.rs/crate/quickcheck/1.1.0) and [security advisories](https://github.com/BurntSushi/quickcheck/security/advisories).

Proptest was also checked because it is the strongest fit by strategy ergonomics. Its repository has no published GitHub advisories, but its issue [#648](https://github.com/proptest-rs/proptest/issues/648), opened 2026-06-20 and still open on the review date, documents a possible data race/undefined behavior involving `static mut DEFAULT_HOOK` during panic-hook initialization and a safe `hardware_rng` API using RDRAND without runtime feature checking. The upstream changelog lists both soundness fixes under **Unreleased**, not under the published 1.11.0 entry ([changelog](https://github.com/proptest-rs/proptest/blob/main/proptest/CHANGELOG.md)). This is sufficient reason to reject 1.11.0 despite the lack of a published advisory. The report is scoped to Proptest and does not imply a defect in QuickCheck.

With `default-features = false`, QuickCheck's only direct normal dependency is `rand` (`sys_rng` feature); this omits the optional `env_logger` and `log` dependencies and avoids adding a separate macro crate. A temporary minimal-feature resolution selected `rand 0.10.3`, `rand_core 0.10.1`, and `getrandom 0.4.3`, with target-support crates `cfg-if 1.0.5`, `libc 0.2.190`, and `r-efi 6.0.0`. Those versions are the probe's current resolution, not pins proposed for the workspace manifest; the committed workspace lockfile will record the actual resolution. This is a modest test-only graph and does not add QuickCheck or its RNG graph to downstream normal/runtime dependencies. Source for the direct dependency and features: [QuickCheck Cargo.toml](https://github.com/BurntSushi/quickcheck/blob/master/Cargo.toml).

## Decision record

### Independent peer review

On 2026-10-05, an independent reviewer confirmed that QuickCheck 1.1.0 exposes the documented public seeded generator (`Gen::from_size_and_seed`), runner RNG injection (`QuickCheck::rng`), and case-count control (`tests(256)`). The reviewer found no blocker for deterministic bounded properties and confirmed that QuickCheck does not persist failure state externally. The exact version pin and lockfile are part of the reproducibility contract.

The reviewer also noted that `rand::SmallRng` does not promise identical output across architectures or future `rand` versions. Therefore the fixed-seed guarantee applies within the pinned test dependency/toolchain environment; generated samples are not specified to be byte-identical across platforms. This does not weaken the protocol assertions, which validate every generated case independently, and does not conflict with the plan's fixed-seed requirement.

Based on that independent review, add only the exact QuickCheck development dependency to `crates/kmipkit-protocol`; do not add it as a workspace runtime dependency. Do not select Proptest 1.11.0 unless upstream publishes a release containing the tracked soundness fixes and that release receives a new dependency review.
