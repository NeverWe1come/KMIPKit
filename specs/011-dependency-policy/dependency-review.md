# Dependency Review: cargo-deny 0.20.2

**Review date**: 2026-10-05
**Scope**: CI-only policy tool; not a published KMIPKit or runtime dependency.
**Review status**: Independent read-only engineering review completed. This is
not the required qualified human security review before 1.0.0.

## Decision

Use `cargo install --locked --version 0.20.2 cargo-deny` in one dedicated
Linux policy job, after implementation-gate approval. Do not install a mutable
action or add `cargo-deny` to any KMIPKit crate manifest. The exact source
release and its packaged lockfile pin the tool's build-time transitive graph.

## Capability and alternatives

`cargo-deny` 0.20.2 covers the existing policy categories in one tool:
advisories, license expressions, banned/duplicate crates, and dependency
sources. `cargo-audit` covers RustSec lockfile advisories but not the other
three categories, so adding it to every run would duplicate the advisory
source without closing a requirement. `cargo-vet` adds a separate audit-record
process not required by this bounded specification. A third-party GitHub
Action was rejected because the repository uses exact action SHA pins and a
direct exact-version install avoids additional action code and permissions.

## Package and release evidence

- Upstream release 0.20.2 was marked the latest release on 2026-10-05 and was
  published 2026-07-09.
- Tagged upstream `Cargo.toml` reports edition 2024, Rust 1.88 MSRV, license
  `MIT OR Apache-2.0`, and maintenance status `actively-developed`. KMIPKit's
  Rust 1.94 CI toolchain satisfies the declared MSRV.
- The package provides the `cargo-deny` binary and a packaged `Cargo.lock`;
  Cargo's `--locked` installation uses that lockfile. This deliberately holds
  tool transitive versions steady until a reviewed tool upgrade.
- The tagged release lockfile has 211 package entries. That is the tool's
  source lockfile size, not the count compiled into a KMIPKit runtime.
- Upstream release assets include binaries for major platforms on a
  best-effort basis. The selected flow builds the tool from the exact crate
  release on the Linux policy runner, avoiding dependence on an ARM64 release
  artifact. Upstream documents a C toolchain prerequisite; confirm that the
  current self-hosted Linux ARM64 runner has a usable compiler before enabling
  the same-repository job.

## Security and supply-chain evidence

- No GitHub security advisory was listed for cargo-deny by upstream during
  this review. A direct OSV query for `crates.io/cargo-deny@0.20.2` returned
  no vulnerability records on 2026-10-05. This is dated evidence and does not
  prove the tool's transitive dependencies are vulnerability-free.
- Crates.io reported the 0.20.2 package checksum as
  `e528dfcbe739af7ce37a77d3d6df1b29dd6887b1c701d888820c0f16b864f737` and
  `yanked = false` at review time.
- A source build still trusts the crates.io package, registry checksums, and
  Rust compiler/toolchain. `--locked` controls the tool's dependency versions;
  it does not establish source provenance or prove the absence of malicious
  code.
- The cargo-deny license check trusts crate SPDX metadata and recognized
  license files; it does not exhaustively inspect every source file in each
  dependency. The feature makes no complete legal audit claim.
- A single Linux job minimizes repeated build cost. It must cover each
  supported target in the checked-in cargo-deny graph configuration; running
  the scanner on Linux does not by itself prove platform-specific code was
  included.

## Core CI target mapping (2026-10-05)

The current workflow's OS labels and runner expression imply this candidate
target set. The report distinguishes documented hosted-runner architectures
from target triples directly printed by `rustc -vV`; the implementation must
capture and validate the policy runner's actual host triple before enforcement:

| CI runner path | Expected Rust target triple |
|---|---|
| `ubuntu-latest` for fork PRs | `x86_64-unknown-linux-gnu` |
| self-hosted `Linux`, `ARM64` for same-repository Linux PRs | `aarch64-unknown-linux-gnu` |
| `windows-latest` | `x86_64-pc-windows-msvc` |
| `macos-latest` | `aarch64-apple-darwin` |

GitHub documents the current standard runner architectures in its
[hosted-runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).
The workflow uses moving `-latest` labels, so this mapping is dated evidence,
not a permanent guarantee; implementation must fail its target inventory if a
runner reports another triple and update the reviewed policy target set.

The local diagnostic host reported `x86_64-pc-windows-msvc` from `rustc -vV`.
The runner-derived values are candidate targets, not direct `rustc -vV`
observations. The self-hosted Linux ARM64 runner's libc ABI remains unverified;
the policy command must fail if the runtime host triple is not in the reviewed
set, and its evidence must distinguish the observed runner target from the
other matrix targets.

## Diagnostic dependency baseline (2026-10-05)

This is a report-only baseline for the exact PR base, not an approved policy
allowlist or a green implementation gate:

| Evidence | Value |
|---|---|
| PR #35 scan checkout | `a84e366b803d47b7d019c89d401b3accb5eec732` |
| Active `release/1.0.0` / PR base | `35a445f500d0ac0b55fe39cd95bf25984ea65216` |
| cargo-deny | `0.20.2` |
| Local Rust host | `rustc 1.99.0`, `x86_64-pc-windows-msvc` |
| Root lockfile SHA-256 | `08cbbb0bbfb0db6e567eec2db83d2b63788cc6ffada8531be0d50033a8ff0231` |
| Fuzz lockfile SHA-256 | `ea34d89d36fa78841f0b1c63064726f09a353f62e68725cc7cdc99d32c6c7778` |
| Root metadata graph | 43 packages / 43 resolved nodes |
| Fuzz metadata graph | 13 packages / 13 resolved nodes |

The diagnostic command was `cargo deny --manifest-path <manifest> --config
<temporary-config> --locked --workspace --all-features check`, once for the
root manifest and once for `fuzz/Cargo.toml`. The temporary graph selected the
four target triples above, included development dependencies, denied
unsound/unmaintained/yanked advisories, duplicate versions, wildcard
requirements, the ADR-0005 TLS crates, and non-crates.io sources. `cargo deny
list --format tsv` was also run for each workspace to inventory license IDs.
Neither command wrote a repository file or lockfile.

The observed license-ID inventory is metadata evidence only; no legal review
or approval is implied:

| Workspace | Observed license IDs and packages |
|---|---|
| Root | `Apache-2.0` (33 packages), `MIT` (32), `Unicode-3.0` (`unicode-ident@1.0.26`), and `Unlicense` (`memchr@2.8.3`, `quickcheck@1.1.0`, `termcolor@1.4.1`, `winapi-util@0.1.11`) |
| Fuzz | `Apache-2.0` (11 packages), `MIT` (10), `NCSA` (`libfuzzer-sys@0.4.12`), plus `kmipkit-ttlv-fuzz@0.0.0` with no usable license field |

The complete package/version-to-license inventory derived from `cargo deny
list --format tsv` is preserved in [`evidence/license-inventory-root.tsv`](evidence/license-inventory-root.tsv)
and [`evidence/license-inventory-fuzz.tsv`](evidence/license-inventory-fuzz.tsv).
Each row is a package/version; the semicolon-separated identifiers are the
licenses reported by cargo-deny. This inventory does not constitute legal
approval.

With a temporary candidate allowlist containing the five observed SPDX IDs
(`Apache-2.0`, `MIT`, `NCSA`, `Unicode-3.0`, `Unlicense`), the root graph
reported advisories, bans, licenses, and sources as passing. The fuzz graph
reported advisories and sources as passing, but failed bans and licenses for
these exact baseline findings:

1. `fuzz/Cargo.toml:12` declares
   `kmipkit-ttlv = { path = "../crates/kmipkit-ttlv" }` without an explicit
   version requirement; cargo-deny 0.20.2 reports it as a wildcard dependency.
   Wildcards are non-waivable under FR-006.
2. `fuzz/Cargo.toml` has no package `license` field, so
   `kmipkit-ttlv-fuzz@0.0.0` is reported as unlicensed. Its license metadata
   needs an explicit reviewed disposition; the repository license is not
   silently inherited as third-party approval.

The initial empty-allowlist run intentionally rejected every licensed crate
and is not treated as a set of policy violations. The candidate-ID run
isolates the two fuzz findings above but does not approve those SPDX IDs. The
root baseline had no advisory, source, banned-crate, duplicate-version, or
wildcard finding. The fuzz baseline had no advisory, source, banned-crate, or
duplicate-version finding beyond its wildcard.

The post-scan local RustSec database checkout was
`ef6173cbc5c50ec8166f9a5b28f07834144373ee` (`2026-10-03T10:14:03+02:00`). The
baseline used the shared local Cargo home, so this record does not claim a
separately attributable refresh event for each workspace invocation. The
implementation must use an isolated `CARGO_HOME` per job and emit the database
revision and timestamp after each successful root and fuzz check as required
by FR-004.

## Required follow-up before implementation

1. Confirm the upstream tool release and crates.io checksum still match this
   record when implementation begins; refresh the independent review if the
   pinned release changes.
2. Verify a C toolchain is present on the self-hosted Linux ARM64 runner and
   test the source installation on the hosted Linux runner used for fork PRs.
3. Before implementation, resolve the non-waivable fuzz wildcard and establish
   the fuzz package's first-party `Apache-2.0` metadata through the FR-013
   Red/Green tasks; do not use an exception for either finding. Then refresh
   both lockfile baselines and complete review of every third-party SPDX ID
   before setting a finite allowlist or any eligible exact exception.
4. Keep this record scoped to the tool; new KMIPKit runtime dependencies
   continue to require their feature-specific dependency review.

## T001 evidence refresh on the active release

**Checkout**: `ee4c6aedcf2c7d8c27cb131faaf6daeaf2587c1e`, the current
`origin/release/1.0.0` head on 2026-10-05. The dependency manifests, both
lockfiles, and `.github/workflows/ci.yml` are byte-for-byte unchanged from
the earlier baseline at `35a445f500d0ac0b55fe39cd95bf25984ea65216`.

The candidate scans were repeated with cargo-deny 0.20.2 and separate new
temporary `CARGO_HOME` directories for the root and fuzz workspaces. Each
workspace used its own temporary report-only configuration, the five SPDX
IDs already observed for that graph, and these exact command forms:

```powershell
cargo deny --manifest-path Cargo.toml --config <root-candidate-config> --workspace --all-features --locked check
cargo deny --manifest-path fuzz/Cargo.toml --config <fuzz-candidate-config> --workspace --all-features --locked check
```

No `--offline` or `--frozen` flag was supplied. The initial per-workspace
`CARGO_HOME` contained no advisory database; cargo-deny cloned the configured
RustSec repository during each scan. The root check returned 0 with all four
check families passing. The fuzz check returned 6, with the expected
unlicensed first-party fuzz package and wildcard local path dependency; its
advisories and sources checks passed. No advisory, source, ADR-0005 hard-ban,
or duplicate-version finding was reported in either graph.

| Workspace | RustSec remote | Database commit | Commit timestamp | Lockfile SHA-256 |
|---|---|---|---|---|
| Root | `https://github.com/RustSec/advisory-db` | `ef6173cbc5c50ec8166f9a5b28f07834144373ee` | `2026-10-03T10:14:03+02:00` | `08cbbb0bbfb0db6e567eec2db83d2b63788cc6ffada8531be0d50033a8ff0231` |
| Fuzz | `https://github.com/RustSec/advisory-db` | `ef6173cbc5c50ec8166f9a5b28f07834144373ee` | `2026-10-03T10:14:03+02:00` | `ea34d89d36fa78841f0b1c63064726f09a353f62e68725cc7cdc99d32c6c7778` |

The fetched database commit is older than the run date because the successful
online refresh returned that repository head; the scan did not infer
freshness from commit age. The pre-existing root and fuzz license inventories
were compared as normalized package/version-to-SPDX sets with fresh
`cargo deny list --format tsv` output. Both inventories matched exactly
(root: 39 rows; fuzz: 12 rows). The manifests and lockfiles are unchanged:

| File | SHA-256 |
|---|---|
| `Cargo.toml` | `a7e20915a799efe26f80d04c89c7b168dedb0d921ed580e606ad81f109dec8f4` |
| `Cargo.lock` | `08cbbb0bbfb0db6e567eec2db83d2b63788cc6ffada8531be0d50033a8ff0231` |
| `fuzz/Cargo.toml` | `6e40c35e20450f8256ac217ed950fb97120534df4d2ab73cae8256d79db4d0eb` |
| `fuzz/Cargo.lock` | `ea34d89d36fa78841f0b1c63064726f09a353f62e68725cc7cdc99d32c6c7778` |

Both full-feature metadata commands succeeded without a platform filter and
resolved 43 packages/43 nodes for the root workspace and 13 packages/13 nodes
for the fuzz workspace. The current CI runner host set is
`x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
`x86_64-pc-windows-msvc`, and `aarch64-apple-darwin`; the evidence status for
these values is detailed below. The candidate policy graph itself is now
unfiltered, so the policy check covers all resolved target-specific edges,
not only these four runner hosts.

The latest available Actions job metadata is from [CI run 37354210003](https://github.com/NeverWe1come/KMIPKit/actions/runs/37354210003),
commit `3aaa68d321f2478f1fd0e04a8238a25684d6ca71`. It records successful
Linux job `111912375459` on `raspberry-home` with `Linux, ARM64` labels,
Windows job `111912375503` with `windows-latest`, and macOS job `111912375382`
with `macos-latest`. GitHub's current hosted
runner reference maps `ubuntu-latest` to x64, `windows-latest` to x64, and
`macos-latest` to arm64, including for private repositories
([runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)).
However, the runner logs were not accessible through the configured GitHub
API credential: downloading a job log returned HTTP 403, `Must have admin
rights to Repository`. No current workflow step prints `rustc -vV`, and there
is no completed fork-PR run among the 88 Actions runs returned by the API.
Therefore the hosted OS/architecture mapping is documented, but the exact
`rustc -vV` output was not captured from the hosted runners. The self-hosted
runner's Linux/ARM64 labels likewise do not prove its libc ABI. Keep the four
triples as the candidate CI runner-host set and make the implementation print
and validate `rustc -vV` at runtime; do not describe the self-hosted GNU target
as directly observed until that output is available. This is an evidence
limitation, not a dependency finding.

One task wording defect was confirmed against `cargo deny --help`: cargo-deny
0.20.2 has no `--disable-fetch` option. The intended online-refresh invariant
is already met by omitting both supported offline switches, `--offline` and
`--frozen`. T008 should avoid naming the nonexistent flag in its invocation
contract.

## T001 unfiltered target-graph and SPDX-expression refresh

FR-001 includes target-specific packages represented by the lockfiles. The
candidate checks were therefore repeated without a `[graph].targets` entry.
The official cargo-deny configuration reference says its default graph
includes every resolved crate, including target-specific dependencies; a
target filter drops edges that do not match the listed triples
([cargo-deny graph configuration](https://embarkstudios.github.io/cargo-deny/checks/cfg.html)).
The exact report-only configurations and normalized license matrices derived
from `cargo deny list --format tsv` are preserved in
[`candidate-root-all-targets.toml`](evidence/candidate-root-all-targets.toml),
[`candidate-fuzz-all-targets.toml`](evidence/candidate-fuzz-all-targets.toml),
[`license-inventory-all-targets-root.tsv`](evidence/license-inventory-all-targets-root.tsv),
and
[`license-inventory-all-targets-fuzz.tsv`](evidence/license-inventory-all-targets-fuzz.tsv).

With this unfiltered graph, root again returned 0 with all four check families
passing. Fuzz again returned 6, and the only findings were the two expected
FR-013 items: missing license metadata on `kmipkit-ttlv-fuzz@0.0.0` and the
wildcard `kmipkit-ttlv` path requirement. Advisory, source, architecture-ban,
and duplicate checks passed for both workspaces. The all-target inventories
contain 40 root package rows and 13 fuzz package rows. Their license-ID
unions are `Apache-2.0`, `LGPL-2.1-or-later`, `MIT`, `Unicode-3.0`, and
`Unlicense` for root; fuzz has `Apache-2.0`, `LGPL-2.1-or-later`, `MIT`, and
`NCSA`. Cargo-deny's `Unlicensed` output for the first-party fuzz package is
a missing-metadata sentinel, not an SPDX identifier.

The full Cargo SPDX expressions explain how the policy handles the IDs:

| Package | Exact `Cargo.toml` expression | Candidate disposition |
|---|---|---|
| `unicode-ident@1.0.26` | `(MIT OR Apache-2.0) AND Unicode-3.0` | The candidate root allowlist covers both sides of the `AND`. |
| `libfuzzer-sys@0.4.12` | `(MIT OR Apache-2.0) AND NCSA` | The candidate fuzz allowlist covers both sides of the `AND`. |
| `r-efi@6.0.0` | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | The unfiltered graph includes this UEFI-only edge. The check passes through the allowed MIT/Apache alternatives; the candidate does not globally allow LGPL. |

Cargo's expression fields preserve these `AND`/`OR` relationships; the TSV
matrices list per-package identifiers for review, use `-` for an absent
identifier, and do not replace the complete expression. The earlier target-filtered files
`license-inventory-root.tsv` and `license-inventory-fuzz.tsv` remain as
historical evidence; the all-target normalized matrices are authoritative for
this refresh. `equivalent`, `hashbrown`, and `indexmap` also occur in Cargo
metadata's package catalog, but their optional dependency edges are inactive
in the current workspace feature graph, so cargo-deny does not include them
in its active graph report. Their presence is not represented as an active
dependency finding.

## Sources

- [SPDX Apache-2.0](https://spdx.org/licenses/Apache-2.0.html)
- [SPDX MIT](https://spdx.org/licenses/MIT.html)
- [SPDX Unicode-3.0](https://spdx.org/licenses/Unicode-3.0.html)
- [SPDX Unlicense](https://spdx.org/licenses/Unlicense.html)
- [SPDX NCSA](https://spdx.org/licenses/NCSA.html)
- [SPDX LGPL-2.1-or-later](https://spdx.org/licenses/LGPL-2.1-or-later.html)
- [Cargo manifest license-expression fields](https://doc.rust-lang.org/cargo/reference/manifest.html#the-license-and-license-file-fields)
- [Cargo-deny graph configuration](https://embarkstudios.github.io/cargo-deny/checks/cfg.html)
- [GitHub-hosted runner architectures](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
- [cargo-deny 0.20.2 release](https://github.com/EmbarkStudios/cargo-deny/releases/tag/0.20.2)
- [Tagged cargo-deny manifest](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/Cargo.toml)
- [Tagged cargo-deny lockfile](https://github.com/EmbarkStudios/cargo-deny/blob/bca0dde53651ee946720e4540b5ce2610bec8f06/Cargo.lock)
- [Cargo install command](https://doc.rust-lang.org/cargo/commands/cargo-install.html)
- [Cargo package and packaged lockfiles](https://doc.rust-lang.org/cargo/commands/cargo-package.html)
- [cargo-deny installation requirements](https://embarkstudios.github.io/cargo-deny/cli/index.html)
- [cargo-deny check coverage](https://embarkstudios.github.io/cargo-deny/checks/index.html)
- [cargo-deny license-check limitations](https://embarkstudios.github.io/cargo-deny/checks/licenses/index.html)
- [cargo-deny source-check behavior](https://embarkstudios.github.io/cargo-deny/checks/sources/index.html)
- [Upstream security advisories](https://github.com/EmbarkStudios/cargo-deny/security/advisories)
- [OSV query API](https://api.osv.dev/v1/query)
