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

The five denied package license expressions are:

- `aws-lc-rs` 1.18.1: `ISC AND (Apache-2.0 OR ISC)`.
- `aws-lc-sys` 0.45.0: includes `ISC`, `BSD-3-Clause`, and `MIT-0` alongside
  Apache-2.0 and MIT alternatives.
- `rustls-webpki` 0.103.15 and `untrusted` 0.9.0: ISC.
- `subtle` 2.6.1: BSD-3-Clause.

The feature graph review found no Hyper HTTP/2, TLS 1.2, rustls early-data,
compression, or `ring` feature enabled. AWS-LC is the only selected TLS
provider. Dependency metadata inspected for the resolved graph reports no
crate requiring a Rust version newer than 1.94. AWS-LC builds native C/C++
code; the Linux, Windows MSVC, and macOS CI jobs must have their respective
native toolchains. The metadata review is not evidence that all three target
builds have passed.

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
