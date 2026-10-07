# KMIPKIT-0013 dependency review

**Review date:** 2026-10-07  
**Status:** In progress; T003 is not complete and implementation remains gated.

## Scope and current graph

The transport dependency set adds Tokio, Hyper HTTP/1, rustls with AWS-LC,
tokio-rustls, Hickory system DNS, native certificate loading, and bytes. Direct
dependencies are exact-pinned, and the root lockfile is committed with the
dependency update. No Git dependency or alternate TLS provider is introduced.

The initial `rustls = 0.23.43` pin was affected by RUSTSEC-2026-0285. The
upstream fixed release is 0.23.45; its release notes identify 0.23.13 through
0.23.44 as affected. The manifest and root lockfile now pin 0.23.45. Sources:
[RustSec RUSTSEC-2026-0285](https://rustsec.org/advisories/RUSTSEC-2026-0285.html)
and [rustls 0.23.45 release notes](https://github.com/rustls/rustls/releases).

## Verification evidence

The repository's official command was run after the rustls update:

```text
pwsh -File scripts/Test-DependencyPolicy.ps1
exit 1: dependency policy exact exception validation failed
```

It installed and verified cargo-deny 0.20.2, checked the Windows host against
the reviewed target set, and ran both waiver-free workspace scans. The command
did not pass because the current exception register is empty while the root
workspace still contains license and duplicate-version findings.

The committed locked graph was scanned directly with cargo-deny 0.20.2. A
separate lockfile-only attempt to remove the `syn` duplicate was rejected
after security review, and `Cargo.lock` was restored to the committed graph.

| Workspace | Advisory errors | Duplicate errors | License errors | Warnings | Result |
|---|---:|---:|---:|---:|---|
| Root | 0 | 2 | 5 | 1 unused `NCSA` allowance | Fail |
| Fuzz | 0 | 0 | 0 | 2 unused allowances (`Unicode-3.0`, `Unlicense`) | Pass |

The root duplicate sets are:

- `core-foundation` 0.9.4 via Hickory `system-configuration` 0.7.0 and
  `core-foundation` 0.10.1 via `rustls-native-certs` 0.8.4's
  `security-framework` 3.7.0 dependency on Apple targets.
- `syn` 2.0.119 and 3.0.6, required by distinct proc-macro dependency
  families in the combined workspace graph.

An upstream update to `system-configuration` 0.8.0 would use
`core-foundation` 0.10, but Hickory resolver 0.26.3 constrains its Apple-only
`system-configuration` dependency to the incompatible 0.7 line. Avoiding this
duplicate therefore requires a Hickory release that updates that constraint
or a different system-DNS design; no lockfile-only update can unify it. A
separate attempt to remove the `syn` duplicate with compatible transitive
pins was rejected after it required vulnerable `zerovec` packages; the
committed lockfile has been restored. Both duplicate sets remain unresolved.

The five denied package license expressions are:

- `aws-lc-rs` 1.18.1: `ISC AND (Apache-2.0 OR ISC)`.
- `aws-lc-sys` 0.45.0:
  `ISC AND (Apache-2.0 OR ISC) AND Apache-2.0 AND MIT AND BSD-3-Clause AND (Apache-2.0 OR ISC OR MIT) AND (Apache-2.0 OR ISC OR MIT-0)`.
- `rustls-webpki` 0.103.15 and `untrusted` 0.9.0: ISC.
- `subtle` 2.6.1: BSD-3-Clause.

The feature graph review found no Hyper HTTP/2, TLS 1.2, rustls early-data,
compression, or `ring` feature enabled. AWS-LC is the only selected TLS
provider. Hickory's `tokio` feature does enable `tokio/rt-multi-thread`
transitively through `hickory-net`, despite KMIPKit directly selecting only
Tokio's `rt` feature; it does not alter the specified current-thread worker
design. Dependency metadata inspected for the resolved graph reports no crate
requiring a Rust version newer than 1.94. AWS-LC builds native C/C++ code.

The dependency set compiled successfully on two hosts:

| Host | Command | Result |
|---|---|---|
| Windows `x86_64-pc-windows-msvc` | `cargo check --manifest-path Cargo.toml --locked -p kmipkit-transport` | Pass; AWS-LC native build and the selected transport dependency graph compiled. |
| Ubuntu 26.04 WSL, Rust 1.94.0 `x86_64-unknown-linux-gnu` | `cargo check --manifest-path /mnt/c/Users/ramp1953/.codex/worktrees/kmipkit-0013-transport/KMIPKit/Cargo.toml --locked -p kmipkit-transport` | Pass; AWS-LC native build and the selected transport dependency graph compiled. |

The macOS target has not yet been compiled locally. Its Apple-target dependency
graph was checked with `cargo tree --target all`; the macOS CI job remains the
verification point for the `system-configuration` and Security Framework
native build path.

## Disposition still required

No advisory waiver is warranted after the rustls update. The direct baseline
scan has zero advisory errors in both workspaces.

The duplicate-version rule is intentionally strict and includes development
dependencies. Before T003 can pass, each duplicate set must either be removed
without changing the accepted DNS and platform-trust design, or receive an
exact, expiring exception in both the machine-readable register and
`.cargo/deny.toml`. Broad `skip-tree`, global advisory ignores, and architecture
ban exceptions remain prohibited.

License findings require either an explicitly reviewed finite SPDX allowlist
decision or package/version-specific clarifications with exact license-file
evidence. This note records the machine findings, not a legal audit or a
completed license disposition. The exact license texts and obligations must
be retained in any distributed third-party notices.

Until these dispositions are recorded and the official script passes for both
workspaces, T003 remains unchecked and implementation tasks T004 onward must
not start.

The standing authorization to continue the roadmap does not claim that the
maintainer personally reviewed these exact license and duplicate-version
exceptions. The dependency-policy contract requires a distinct human reviewer
for each exception; none is represented as approved here.

## Follow-up version and license audit

An additional read-only review on 2026-10-07 checked published dependency
versions and the exact locked graph. `cargo info hickory-resolver` reports
0.26.3 as the latest published release (MSRV 1.88); its manifest constrains
Apple `system-configuration` to version 0.7. The current upstream main
manifest labels its workspace `0.27.0-alpha.1` and has updated
`system-configuration` to 0.8, which is compatible with `core-foundation`
0.10. This appears to address the Hickory side of the Core Foundation
duplicate once a reviewed, published release is available, but
`cargo info hickory-resolver@0.27.0-alpha.1` cannot resolve that version from
crates.io today. A git dependency is prohibited by KMIPKIT-0011. The upstream
references checked were [Hickory releases](https://github.com/hickory-dns/hickory-dns/releases),
the [Hickory 0.26.3 manifest](https://github.com/hickory-dns/hickory-dns/blob/v0.26.3/Cargo.toml),
the [current Hickory manifest](https://github.com/hickory-dns/hickory-dns/blob/main/Cargo.toml),
and [`system-configuration` 0.8.0](https://docs.rs/crate/system-configuration/0.8.0/source/Cargo.toml).

The Apple-target Cargo graph for the currently pinned 0.26.3 confirms both
duplicate paths directly:

```text
hickory-resolver 0.26.3 -> system-configuration 0.7.0 -> core-foundation 0.9.4
rustls-native-certs 0.8.4 -> security-framework 3.7.0 -> core-foundation 0.10.1
```

Removing Hickory's `system-config` feature and loading resolver settings with
separate platform code is a possible design alternative, but it is not a
lockfile or feature-only cleanup: it would need a reviewed design update and
tests for system DNS, hosts, search domains, split DNS/VPN behavior, request
limits, and deadlines on Linux, Windows, and macOS. Replacing Hickory with a
blocking system lookup would not implement the bounded asynchronous resolver
contract in `spec.md` and `research.md`.

An isolated Apple-target graph was also checked with
`rustls-native-certs` 0.7.3, the latest 0.7 release, alongside Hickory 0.26.3.
That candidate resolves both platform dependencies through
`core-foundation` 0.9.4 and removes the `core-foundation` 0.10.1 duplicate for
that dependency pair. It does not remove the workspace's `syn` 2/3 duplicate,
which remains reachable through Hickory's macro and ICU/IDNA dependency graph.
The candidate also changes the security-framework major version from 3 to 2
and downgrades the explicitly selected native-certificate loader. KMIPKIT-0013
pins `rustls-native-certs` 0.8.4 in its accepted spec and TLS contract, so this
alternative is not compatible with the approved dependency set without a
specification review. The probe ran in a temporary project; it did not modify
the repository manifests or lockfile.

An attempted lockfile-only alignment pinned compatible releases of
`async-trait`, `displaydoc`, `futures-util`, `thiserror`, `tokio-macros`, and
the ICU derive chain so the reachable proc-macro graph used `syn` 2.0.119.
The trial compiled with `cargo check --workspace --all-features --locked`,
and cargo-deny reported no known advisory for that graph. Independent
security review then found that this alignment required `zerovec` 0.11.6 and
`zerovec-derive` 0.11.3, both affected by [GHSA-7fx9-626j-vqph](https://github.com/unicode-org/icu4x/security/advisories/GHSA-7fx9-626j-vqph).
The advisory rates the issue High: the derive can accept invalid bytes in
later elements of a multi-element buffer, potentially violating type
validity. It identifies `zerovec` 0.11.8 and `zerovec-derive` 0.11.5 as
patched; the upstream [fix](https://github.com/unicode-org/icu4x/pull/8393)
documents the validation bug. The current lockfile keeps these fixed versions
(`zerovec` 0.11.8 and `zerovec-derive` 0.11.6), which means the `syn` 3
dependency remains. The trial lockfile was discarded. The cargo-deny
advisory result alone is insufficient for this package because it did not
report this GHSA.

The independent package-source review confirmed the published license
expressions recorded above in the versions fixed by `Cargo.lock`. The ISC and
BSD-3-Clause declarations are required by the active Rustls and AWS-LC
dependency graph; no selected feature toggle removes them while retaining the
accepted `rustls` + `aws-lc-rs` architecture. For `aws-lc-sys`, the package's
complete `LICENSE` and bundled third-party notices are needed for any human
disposition; the SPDX expression alone is not a legal analysis. The review
did not make a legal determination or approve an exception.

Commands used for this follow-up included `cargo info hickory-resolver`,
`cargo info rustls-native-certs`, `cargo info system-configuration`,
`cargo tree --locked --workspace --target aarch64-apple-darwin -i
core-foundation@0.9.4` / `core-foundation@0.10.1`, `cargo tree --locked
--workspace -d`, `cargo deny check`, and `cargo deny check advisories`. Both
`syn` paths were checked with `cargo tree --locked --workspace -i
syn@2.0.119` and `cargo tree --locked --workspace -i syn@3.0.6`. The version
and graph findings supplement, but do not replace, the required human review.
T003 remains unchecked.
