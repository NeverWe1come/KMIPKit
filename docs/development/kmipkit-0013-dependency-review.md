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

The current locked baseline scans were then run directly with cargo-deny
0.20.2 so their exact findings could be recorded after the rustls update:

| Workspace | Advisory errors | Duplicate errors | License errors | Warnings | Result |
|---|---:|---:|---:|---:|---|
| Root | 0 | 2 | 5 | 1 unused `NCSA` allowance | Fail |
| Fuzz | 0 | 0 | 0 | 2 unused allowances (`Unicode-3.0`, `Unlicense`) | Pass |

The root duplicate sets are:

- `core-foundation` 0.9.4 via Hickory `system-configuration` 0.7.0 and
  `core-foundation` 0.10.1 via `rustls-native-certs` 0.8.4's
  `security-framework` 3.7.0 dependency on Apple targets.
- `syn` 2.0.119 and 3.0.6, required by distinct proc-macro dependency families
  in the combined workspace graph.

An upstream update to `system-configuration` 0.8.0 would use
`core-foundation` 0.10, but Hickory resolver 0.26.3 constrains its Apple-only
`system-configuration` dependency to the incompatible 0.7 line. Avoiding this
duplicate therefore requires a Hickory release that updates that constraint
or a different system-DNS design; no lockfile-only update can unify it. The
`syn` duplicate is likewise across incompatible major versions and multiple
independent proc-macro families; do not force an unreviewed dependency
replacement to hide it.

The five denied package license expressions are:

- `aws-lc-rs` 1.18.1: `ISC AND (Apache-2.0 OR ISC)`.
- `aws-lc-sys` 0.45.0: includes `ISC`, `BSD-3-Clause`, and `MIT-0` alongside
  Apache-2.0 and MIT alternatives.
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
