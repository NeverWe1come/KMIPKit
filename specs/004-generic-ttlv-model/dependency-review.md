# Implementation dependency review: `zeroize` 1.9.0

**Reviewed**: 2026-10-04

**Scope**: KMIPKIT-0004 T017; runtime dependency of `kmipkit-ttlv`.

## Capability and alternatives

`zeroize` provides safe trait methods for clearing integer primitives and
owned `String`/`Vec` storage using compiler-resistant writes. KMIPKit calls
only its safe API; the crate uses internal unsafe volatile writes to provide
that guarantee. KMIPKit needs no FFI or assembly for this behavior.

Bare owned buffers do not clear themselves. Fixed boxed byte slices would
complicate text ownership without improving the current-capacity guarantee.
Handwritten volatile code is prohibited in this crate and would duplicate
audited dependency work. A generic `Zeroize for Box<T>` is not provided, so
T017 uses a private `Secret<T>` that owns a `Box<T>` and zeroizes its payload
before deallocation.

## Pinned package and feature set

- Workspace requirement: exact `zeroize = 1.9.0`, with default features off
  and only `alloc` enabled.
- Lockfile checksum: `e13c156562582aa81c60cb29407084cdb54c4164760106ab78e6c5b0858cf64e`.
- Package metadata: edition 2024, MSRV 1.85, license `Apache-2.0 OR MIT`;
  compatible with this workspace's Rust 1.94 MSRV.
- The package is pure Rust and supports portable `alloc` targets, including
  WASM and `no_std` environments. The `alloc` feature has no runtime
  third-party dependencies. `derive`, `std`, and optional serde integration
  are disabled; the Cargo feature tree contains only `alloc` for this
  dependency. The separate `serde` and `trybuild` dependencies remain
  test-only.
- `zeroize` 1.9.0 has no dependency entries in its lockfile package record.
  `cargo tree --locked -e normal -i zeroize` shows it as a normal dependency
  only of `kmipkit-ttlv`.

## Maintenance and security history

The upstream changelog shows releases through September 2026. Upstream states
that maintenance is provided by volunteers on a reasonable-effort basis; this
is maintenance context, not a security certification. Upstream PR #1538
addressed a 32-bit ARM allocator test-harness false positive, not a runtime
library defect.

RustSec's indexed package search lists historical advisory
`RUSTSEC-2021-0115` for the separate `zeroize_derive` package before 1.1.1.
This implementation does not enable or depend on `zeroize_derive`. This
observation is limited to that advisory and is not a claim that the selected
dependency or complete lockfile is vulnerability-free.

This checkout has no installed `cargo audit` or `cargo deny` subcommand:
`cargo audit --version` and `cargo deny --version` both returned “no such
command”. No repository lockfile-audit workflow was found in `.github` or
`docs`, so no automated vulnerability or license audit ran locally. The
exact package metadata, feature tree, source behavior, and lock entry were
reviewed; a CI or reviewer audit remains necessary if the project requires
one.

## Zeroization guarantee and limits

The pinned 1.9.0 source implements `Vec<T>::zeroize` by clearing initialized
elements, clearing the vector length, and zeroizing its full current capacity.
`String::zeroize` delegates to that vector implementation. Earlier buffers
left by caller-side reallocations, caller copies, temporary stack/register
copies, and managed-runtime copies are outside KMIPKit's guarantee. The model
does not expose mutable owned payload buffers after construction, so its
String/Vec payloads do not grow or reallocate while KMIPKit owns them.

Verification in T017 uses the safe Drop spy and live-object probe in
`crates/kmipkit-ttlv/tests/value_zeroization.rs`; the separate address test
checks that growing a Structure does not move a boxed payload.

## Sources reviewed

- [Pinned package metadata for zeroize 1.9.0](https://docs.rs/crate/zeroize/1.9.0)
- [Pinned zeroize 1.9.0 library source](https://docs.rs/crate/zeroize/1.9.0/source/src/lib.rs)
- [Upstream zeroize changelog](https://github.com/RustCrypto/utils/blob/master/zeroize/CHANGELOG.md)
- [Upstream security policy](https://github.com/RustCrypto/utils/security)
- [Upstream PR #1538](https://github.com/RustCrypto/utils/pull/1538)
- [RustSec RUSTSEC-2021-0115](https://rustsec.org/advisories/RUSTSEC-2021-0115.html)
