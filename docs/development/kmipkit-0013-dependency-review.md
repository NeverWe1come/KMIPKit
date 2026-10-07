# KMIPKIT-0013 dependency review

**Review date:** 2026-10-07  
**Status:** T003 completed on 2026-10-07 for the original production graph, then reopened and re-completed for T004's test-only PKI dependency graph. The closure records below distinguish the original review from the T004 extension.

## Scope and current graph

The transport dependency set adds Tokio, Hyper HTTP/1, rustls with AWS-LC,
tokio-rustls, Hickory system DNS, native certificate loading, and bytes. Direct
dependencies are exact-pinned, and the root lockfile is committed with the
dependency update. No Git dependency or alternate TLS provider is introduced.

The T004 extension adds exact-pinned `rcgen` 0.14.10 to
`kmipkit-test-support` with default features disabled and `aws_lc_rs` selected.
It adds exact-pinned `aws-lc-rs` 1.18.1 with default features disabled and only
`prebuilt-nasm` enabled so standalone Windows fixture builds use the already
selected provider without requiring a system NASM executable. Test-support's
`fixtures` and `scripted-transport` features are separate; transport opts into
fixtures without activating the optional edge back to `kmipkit-transport`.
`kmipkit-client` explicitly opts into `scripted-transport` for its existing
test doubles. No production package depends on `kmipkit-test-support` or
`rcgen`.

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

## T004 test-fixture dependency review

The exact T004 graph was re-reviewed after adding the test-only PKI generator.
The original T003 findings, maintainer-approved policy exceptions, and PR #50
evidence apply to the earlier graph; they do not substitute for these checks.

- `pwsh -File scripts/Test-DependencyPolicy.ps1` passed on 2026-10-07 with host
  `x86_64-pc-windows-msvc`. It validated all four existing exact exception IDs,
  refreshed both RustSec scans to commit
  `b8a1a33e246a0a9a3b5f377248c41a503defec74`, and confirmed the root and fuzz
  lockfiles remained unchanged during the check. The dependency closure is
  covered by the existing finite license allowlist; no new exception or
  advisory waiver was added.
- `cargo tree --manifest-path Cargo.toml -p kmipkit-transport --all-targets -e features --locked --offline --prefix none`
  confirmed Hyper enables `client` and `http1`; rustls/tokio-rustls and rcgen
  use AWS-LC; `aws-lc-rs` enables `prebuilt-nasm`; and the graph has no `ring`
  provider, TLS 1.2, HTTP/2, proxy, or compression feature.
- `cargo tree --manifest-path Cargo.toml -p kmipkit-transport -e normal --locked --offline --prefix none`
  confirmed the production transport dependency path does not include
  `kmipkit-test-support`, `rcgen`, `reqwest`, or `hyper-util`.
- A first Windows build probe before adding the explicit `prebuilt-nasm`
  feature failed because the rcgen-only AWS-LC path attempted to invoke NASM.
  The test-support dependency now enables `prebuilt-nasm` with AWS-LC's
  defaults still disabled. The full platform build matrix remains a CI gate.

This review covers the T004 dependency addition and closes the reopened T003
gate. CI must still compile the test-support fixture and its AWS-LC build path
on the supported Linux, Windows, and macOS targets.

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
and downgrades the explicitly selected native-certificate loader. The accepted
KMIPKIT-0013 platform-trust table selects `rustls-native-certs` 0.8.4, and
research decision D5 records the same version; this alternative therefore
requires a specification review before replacing the approved selection. The
probe ran in a temporary project; it did not modify the repository manifests
or lockfile.

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

## Dependency disposition and license policy approval

The maintainer requested a concrete way to clear the remaining dependency
gate. The approved license allowlist and exact duplicate dispositions are
recorded below. The current specification pins Hickory 0.26.3 and
`rustls-native-certs` 0.8.4;
their Apple-target transitive constraints require `core-foundation` 0.9.4 and
0.10.1 respectively. The two `syn` major versions are also reachable through
Hickory's current proc-macro graph. `syn` 2 is required transitively by
`jni-macros`/`jni-sys-macros` and `serde_derive`, so changing KMIPKit's direct
test-only `syn` dependency does not eliminate it. The patched `zerovec`
0.11.8 and `zerovec-derive` 0.11.6 retain the `syn` 3 path. The minimum
patched `zerovec-derive` 0.11.5 also declares `syn` 3. Keep the patched
versions; the earlier lockfile-only alignment used vulnerable releases.

**Approved duplicate decision (2026-10-07):** The maintainer authorized four
exact exceptions for the currently required versions, each limited to the
crates.io source and expiring on 2026-12-31 (within 90 days). The review and
approval are recorded by `@NeverWe1come` in PR #50. `KMIPKit maintainers` own
removal or renewal before expiry.

| Exception | Exact scope | Current dependency path |
|---|---|---|
| `KMIPKIT-0011-EX-001` | `core-foundation@0.9.4` | Hickory 0.26.3 → `system-configuration` 0.7.0 |
| `KMIPKIT-0011-EX-002` | `core-foundation@0.10.1` | `rustls-native-certs` 0.8.4 → `security-framework` 3.7.0 |
| `KMIPKIT-0011-EX-003` | `syn@2.0.119` | JNI macros, `serde_derive`, and test dependencies |
| `KMIPKIT-0011-EX-004` | `syn@3.0.6` | Hickory/ICU derive chain, including patched `zerovec-derive` 0.11.6 |

No global duplicate allowance or `skip-tree` entry was added. Reassess these
exact scopes when the dependency graph changes and remove them when upstream
constraints converge. Preserve `zerovec` 0.11.8 and `zerovec-derive` 0.11.6
or later patched releases; do not align the `syn` versions by downgrading to
the vulnerable releases described above. Changing the accepted resolver or
platform-trust dependency design still requires review of KMIPKIT-0013.

**Approved license decision (2026-10-07):** The KMIPKit maintainer
(@NeverWe1come) reviewed the locked package license evidence and the standard
[SPDX ISC text](https://spdx.org/licenses/ISC.html) and
[SPDX BSD-3-Clause text](https://spdx.org/licenses/BSD-3-Clause.html), and
approved adding both identifiers to the finite dependency license allowlist.
This approval covers dependencies in both root and fuzz graphs whose declared
SPDX expressions use these identifiers. The allowlist is synchronized in
`.cargo/deny.toml` and `.cargo/deny-baseline.toml`; the package license files
and third-party notices remain the distribution evidence. The approval was
given directly by the maintainer in the Codex task on 2026-10-07 and is
recorded here in the reviewed PR commit.

A temporary cargo-deny 0.20.2 config, differing from the previous
`.cargo/deny.toml` only by these two identifiers, was checked offline for the
license rule. The root workspace changed from five license errors to zero;
fuzz remained at zero. This probe did not alter either checked-in config and
does not replace the online advisory check. The checked-in allowlist change
applies that approved decision.

The exact locked package archives expose these top-level license files. The
hashes below are SHA-256 of the files extracted from the crates.io packages;
the reviewer should inspect the complete files, especially `aws-lc-sys`,
whose `LICENSE` includes notices for third-party components compiled into
the native library and others that are not compiled:

| Package | License file | SHA-256 |
|---|---|---|
| `aws-lc-rs@1.18.1` | `LICENSE` | `b50b376e7d24a0598488b730c3034ffcd0b58dd36c0913a24b9903a3cfd04bf7` |
| `aws-lc-sys@0.45.0` | `LICENSE` | `728536b4160e051f86d7c9c388f704866b3d512cd7df97ac3516c65279523c4e` |
| `rustls-webpki@0.103.15` | `LICENSE` | `5b698ca13897be3afdb7174256fa1574f8c6892b8bea1a66dd6469d3fe27885a` |
| `subtle@2.6.1` | `LICENSE` | `d1fc1bc0d155df60b2e7705b6b2ae02a05c96f948e1cec6e2fb86360b09f346b` |
| `untrusted@0.9.0` | `LICENSE.txt` | `7abd9b6960dcf7d4d0a48606a5b71bfe37d472db68d70637f3a58a56785f1621` |

The license allowlist decision and the four exact duplicate exceptions above
are approved and synchronized with the machine-readable register and
`.cargo/deny.toml`. T003 remains incomplete until its full dependency,
native-build, MSRV, and supported-target review is complete.

**Policy verification (2026-10-07):** `./scripts/Test-DependencyPolicy.ps1`
passed on `x86_64-pc-windows-msvc` with cargo-deny 0.20.2. Both root and fuzz
workspace checks refreshed RustSec to commit
`f246cde705ecb3a6b421d6d5462c6d88f317db5f`; the runner validated all four
exception IDs and confirmed that `Cargo.lock` and `fuzz/Cargo.lock` were
unchanged.

## Original T003 closure evidence

The following final evidence supersedes the earlier historical statements in
this note that T003 remained open:

- The maintainer-reviewed exact duplicate exceptions and finite ISC/BSD-3-Clause
  license allowlist are recorded above and were merged in [PR #50](https://github.com/NeverWe1come/KMIPKit/pull/50).
- The official command `pwsh -File scripts/Test-DependencyPolicy.ps1` passed
  from the updated production-transport branch on 2026-10-07. It validated all
  four exception IDs, refreshed both RustSec scans to commit
  `b8a1a33e246a0a9a3b5f377248c41a503defec74`, and confirmed that `Cargo.lock`
  and `fuzz/Cargo.lock` were unchanged.
- [CI run 37604245540](https://github.com/NeverWe1come/KMIPKit/actions/runs/37604245540)
  passed dependency policy, coverage and coverage-gate checks, script contracts,
  and workspace formatting, Clippy, tests, and documentation builds on Ubuntu,
  Windows, and macOS with both Rust 1.94 and stable. Those platform builds
  cover the selected AWS-LC native dependency path; the CI run also passed the
  normative inventory and immutable-source checks.
- The locked feature graph, MSRV metadata, native build requirements, and
  supported-target dependency paths are documented in this review. No
  unresolved dependency-policy finding remains for the accepted design.

The original T003 review passed before adding the T004 certificate fixture
dependency. It no longer closes the reopened dependency gate. The client-
execution task gaps identified by the independent audit must be reconciled
before starting T046 through T051. Historical audit notes remain here so
future dependency reviews can reuse their evidence.
