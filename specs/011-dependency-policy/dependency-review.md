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

## Required follow-up before implementation

1. Confirm the upstream tool release and crates.io checksum still match this
   record when implementation begins; refresh the independent review if the
   pinned release changes.
2. Verify a C toolchain is present on the self-hosted Linux ARM64 runner and
   test the source installation on the hosted Linux runner used for fork PRs.
3. Audit the actual root and fuzz lockfiles with the candidate version and
   record all license/advisory/source/ban/duplicate findings before selecting
   a finite policy allowlist or exceptions.
4. Keep this record scoped to the tool; new KMIPKit runtime dependencies
   continue to require their feature-specific dependency review.

## Sources

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
