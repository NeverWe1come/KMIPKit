# Client dependency review: KMIP TTLV wire codec

**Status**: Pending; no dependency or manifest change is approved by this record.

Complete this record and obtain an independent review before any change to
`crates/kmipkit-client/Cargo.toml`. Cover every newly direct dependency of
`kmipkit-client`, including internal workspace crates and test-only packages.
For each entry record its capability, alternatives, maintenance, security
history, MSRV, license, supported platforms, transitive cost, exact package
version, and exact enabled Cargo features. Record whether it is test-only or a
normal dependency and the reason for that scope.

The feature selections below are the exact proposed selections supported by
the current local manifests and cached package metadata. They are not
independent review or approval. If a manifest edit uses a different selection,
or the selection cannot be confirmed against the package metadata at edit
time, resolve the exact `default-features` and `features` settings in this
record and obtain independent review before changing the manifest. An
unresolved feature selection is a hard prerequisite failure for any client
manifest edit.

## Planned direct dependency candidates

This inventory reflects the current task proposal only. It does not approve
these dependencies, versions, features, or manifest changes.

| Package | Proposed scope and pin | Review status |
| --- | --- | --- |
| `kmipkit-ttlv` | Workspace dependency; current workspace version `0.1.0`; Cargo metadata reports no declared features, so proposed default selection resolves to no enabled crate features; initially proposed as a client dev-dependency, then a normal dependency in T003 | Research recorded; independent review pending |
| `zeroize` | Workspace dependency `=1.9.0`, `default-features = false`, feature `alloc`; initially proposed as a client dev-dependency, then a normal dependency in T003 | Research recorded; independent review pending |
| `static_assertions` | Test-only, proposed pin `=1.1.0`; cached package metadata declares only optional `nightly` and no default feature, so proposed default selection resolves to no enabled crate features (`nightly` disabled) | Research recorded; independent review pending |
| `serde` | Test-only, proposed pin `=1.0.228`; the existing TTLV dev-dependency and `cargo tree --locked -e features` show the unqualified pin's default selection enables `std`; proposed client selection is also default/`std`, with optional `alloc`, `derive`, `rc`, and `unstable` disabled | Pin decision and independent review pending |

The metadata basis is the current workspace `Cargo.toml`, `crates/kmipkit-ttlv/Cargo.toml`, `Cargo.lock`, Cargo workspace metadata/tree output, and locally cached package metadata for `serde` 1.0.228 and `static_assertions` 1.1.0. Reconfirm these selections before the future manifest edits; this evidence only resolves the proposed feature sets and does not complete package rationale or independent review.

**Unresolved Serde pin decision (checked 2026-10-05):** the proposal pins
`serde = "=1.0.228"`, while the current published version is 1.0.229
(published 2026-07-18). Before any client manifest edit, the reviewer must
record whether to retain 1.0.228 or revise the pin and recheck its exact
features and dependency graph. This record does not silently update the
proposal or lockfile.

The review for `specs/004-generic-ttlv-model/dependency-review.md` may be
referenced for `zeroize` 1.9.0 only to the extent that package version,
enabled features, and reviewed scope match. This record must explain the
client-specific reason for using `zeroize` in the Red tests and private
encoded-output owner; a reference to 004 alone does not establish that use
rationale or independently approve the client dependency.

## Per-package rationale and independent review

Fill out one entry per candidate above, including any dependency omitted from
the initial inventory before adding it to the client manifest.

| Package | Capability and alternatives | Maintenance and security history | MSRV, license, platforms | Transitive cost; exact version/features; dependency scope | Independent reviewer and disposition |
| --- | --- | --- | --- | --- | --- |
| `kmipkit-ttlv` | Provides the generic `Item`, `Value`, and related TTLV model consumed by the client’s private writer and decoder. `kmipkit-protocol` does not re-export these types. Moving the writer into this crate or adding protocol re-exports would change the specified layer boundary; the 005 design keeps the writer private in `kmipkit-client` and the public model/decoder here. | First-party workspace crate maintained with KMIPKit; no third-party release or advisory history applies. Source forbids unsafe code and model payloads are zeroized on drop. No independent security audit was reviewed. | Inherits workspace Rust 1.94 MSRV and Apache-2.0 license. No target-specific manifest dependencies. Current repository CI covers Ubuntu, Windows, and macOS, with Linux ARM64 used for some jobs; this matrix is not a guarantee for every Rust target. | Path workspace crate, version `0.1.0`; Cargo metadata reports no declared crate features (the implicit default feature is empty). Already present through `kmipkit-protocol`, so a direct client edge adds no package node. Its `serde` and `trybuild` dependencies are dev-only and do not enter the client’s normal dependency tree. Dev-dependency during Red tests, then normal dependency when client production code imports these types. | Pending |
| `zeroize` | Supplies safe `Zeroize` and `Zeroizing` APIs for the separately allocated encoded `Vec<u8>` owner; the TTLV model’s private zeroizing allocations do not cover this client-owned wire buffer. Plain `Vec` drop leaves bytes in allocator memory; handwritten volatile writes would duplicate unsafe dependency code. `alloc` enables owned-buffer implementations without `std`. See [1.9.0 API/features](https://docs.rs/zeroize/1.9.0/zeroize/). Reallocation copies, caller copies, and external runtime copies remain outside this crate’s zeroization guarantee. | 1.9.0 was released 2026-06-12. Upstream supports only its most recent release and describes volunteer, reasonable-effort maintenance in its [security policy](https://github.com/RustCrypto/utils/blob/master/SECURITY.md). Targeted RustSec evidence found [RUSTSEC-2021-0115](https://rustsec.org/advisories/RUSTSEC-2021-0115.html) for the separate `zeroize_derive` package; the proposed `derive` feature is disabled. This lookup is not a complete vulnerability audit. The [1.9.0 performance regression](https://github.com/RustCrypto/utils/issues/1504) was addressed by [PR #1535](https://github.com/RustCrypto/utils/pull/1535), merged to upstream master on 2026-09-11 after the pinned release. The issue/PR describe a performance regression, not a security or correctness finding; the maintainer states volatile writes alone retain the optimization-resistance guarantee. The [1.9.0 release source](https://docs.rs/crate/zeroize/1.9.0/source/src/lib.rs) still includes the redundant barrier. | Upstream MSRV 1.85, compatible with workspace Rust 1.94. License `Apache-2.0 OR MIT`. Portable pure Rust with `no_std` support; upstream describes WASM support. `alloc` provides `Vec`/`String` support; no FFI or assembly is used. | Exact workspace selection `=1.9.0`, `default-features = false`, `features = ["alloc"]`; lock checksum `e13c156562582aa81c60cb29407084cdb54c4164760106ab78e6c5b0858cf64e`. Only `alloc` is enabled; `std`, `derive`, and optional serde integration are disabled. Lock entry has no dependencies. Already in the client’s normal package tree through TTLV; a direct edge adds no package node. This package-tree fact does not remove runtime work: the private owner deliberately zeroizes its buffer capacity on drop. Dev-dependency during Red tests, then normal dependency for the private production owner. | Pending |
| `static_assertions` | Provides compile-time type/trait assertions, including `assert_not_impl_any!`, for proving that the private encoded owner is not `Clone`, `Copy`, `Debug`, `Display`, `serde::Serialize`, `AsMut<[u8]>`, or `Into<Vec<u8>>`. `trybuild` negative compile fixtures are an alternative but are more fixture/diagnostic-sensitive; native const assertions do not provide the same compact negative-trait checks. See the [crate documentation](https://docs.rs/static_assertions/1.1.0/static_assertions/). | Package metadata marks it passively maintained; 1.1.0 was published 2019-03-11 and the latest repository commit observed was 2020-11-02. Exact-version OSV query for `static_assertions` 1.1.0 returned an empty `vulns` array on 2026-10-05; RustSec has no crate advisory directory for it. These are dated database observations, not evidence that the package is vulnerability-free. | Upstream README advertises rustc `^1.37.0`; the manifest does not declare `rust-version`, so this is not a machine-enforced MSRV. License `MIT OR Apache-2.0`. Source is `no_std` with no target-specific dependencies; broad target compatibility is an inference, not a KMIPKit support guarantee. | Test-only proposed pin `=1.1.0`; no default feature is declared, and the only optional `nightly` feature is disabled. The package has no runtime or build dependencies and contributes compile-time macros only. It is not in the current workspace lock/tree; adding it as a dev-dependency would not add runtime packages. | Pending |
| `serde` | The private-owner tests need the real `serde::Serialize` trait for a negative trait assertion; they do not serialize the owner or need a data format. `trybuild` cannot avoid referencing the trait package. Using `serde_core` directly is possible but less conventional than the public `serde` facade. See the [pinned manifest](https://docs.rs/crate/serde/1.0.228/source/Cargo.toml.orig). | Serde is actively maintained; 1.0.228 was published 2025-09-27, and 1.0.229 was the latest published version on 2026-10-05 (published 2026-07-18). The exact 1.0.228 OSV query returned an empty `vulns` array on 2026-10-05; RustSec has no crate advisory directory for `serde`. This is a dated database observation, not a vulnerability-free claim. The one-patch-behind pin remains an unresolved review decision before manifest edits (see above). | Declared MSRV Rust 1.56. License `MIT OR Apache-2.0`. Serde is portable and supports `no_std` with defaults disabled; this proposed default/`std` feature selection is for host-side tests and has no OS-specific dependency. | Proposed test-only exact pin `=1.0.228`, default features enabled (`std`); `alloc`, `derive`, `rc`, and `unstable` are disabled. Mandatory exact transitive package is `serde_core =1.0.228`; Serde’s `derive` feature is disabled, although `serde_derive` is already present in the workspace test graph through `trybuild`. Workspace already contains serde/core and proc-macro tooling through TTLV’s test graph, so this client dev-dependency adds no production edge and should add no package node in the current workspace graph. The precise package-tree effect must be reconfirmed if the pin changes. | Pending |

### Dated package metadata and security checks

- `zeroize` metadata and pinned manifest: [docs.rs package page](https://docs.rs/crate/zeroize/1.9.0), [release-tag Cargo.toml](https://github.com/RustCrypto/utils/blob/zeroize-v1.9.0/zeroize/Cargo.toml), and [release-tag changelog](https://github.com/RustCrypto/utils/blob/zeroize-v1.9.0/zeroize/CHANGELOG.md). Its package record in the current `Cargo.lock` contains no dependencies, and `cargo tree --locked --offline -e features -i zeroize` reports only `alloc`.
- `static_assertions` exact registry metadata: [crates.io API](https://crates.io/api/v1/crates/static_assertions/1.1.0); upstream source and maintenance context: [repository](https://github.com/nvzqz/static-assertions), [README/MSRV badge](https://github.com/nvzqz/static-assertions#readme), and [latest-commit API](https://api.github.com/repos/nvzqz/static-assertions/commits?per_page=1). Cargo metadata reports no normal or build dependencies; the proposed manifest selection disables its sole optional `nightly` feature.
- Serde exact metadata: [1.0.228 package source manifest](https://docs.rs/crate/serde/1.0.228/source/Cargo.toml.orig), [current package page](https://docs.rs/crate/serde/latest), and [upstream release history](https://github.com/serde-rs/serde/releases). The current `Cargo.lock` records 1.0.228 and its `serde_core` dependency.
- Security database checks performed 2026-10-05: exact-version POST queries to the [OSV API](https://api.osv.dev/v1/query) for `static_assertions` 1.1.0 and `serde` 1.0.228 returned empty `vulns` arrays. Corresponding [RustSec `static_assertions` package](https://api.github.com/repos/RustSec/advisory-db/contents/crates/static_assertions?ref=main) and [Serde package](https://api.github.com/repos/RustSec/advisory-db/contents/crates/serde?ref=main) paths returned 404. For `zeroize`, the identified RustSec record is for the separate optional `zeroize_derive` package, [RUSTSEC-2021-0115](https://rustsec.org/advisories/RUSTSEC-2021-0115.html). Absence of a matching indexed record is not proof that a package is vulnerability-free, and no comprehensive workspace audit is claimed here.

Do not treat the candidate rationale in `tasks.md` or the prior 004 review as
independent review or approval. This record remains pending until the pin
decision is resolved and every candidate has an independent reviewer and
disposition. The reviewer-owned design checklist's dependency review item
remains open until then.
