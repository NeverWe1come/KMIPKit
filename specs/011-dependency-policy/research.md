# Research: Dependency Policy Gates

**Baseline**: `release/1.0.0` at `35a445f500d0ac0b55fe39cd95bf25984ea65216` (2026-10-05)

## Existing project evidence

- `docs/development/testing.md` requires dependency, advisory, source, and
  license policy on every PR and lists updated vulnerability review under
  scheduled checks. It also separates release signing, hashes, SBOM, and
  provenance into release checks.
- `specs/001-cross-platform-ci/spec.md` explicitly deferred the dependency,
  advisory, and license policy to a separate infrastructure specification;
  `specs/001-cross-platform-ci/plan.md` says that feature must not change
  dependency policy.
- `.github/workflows/ci.yml` runs every PR to `master` and `release/**`, has
  daily schedule `17 3 * * *`, requests `contents: read`, and uses full-SHA
  action references with checkout credentials disabled. The scheduled job
  currently covers only informational branch coverage.
- Root and fuzz workspaces use separate lockfiles. Current CI Rust checks use
  `--locked`; there is no checked-in cargo-deny configuration or dependency
  policy job on this baseline.
- The unpublished `fuzz` package is a separate workspace and therefore does
  not inherit the root workspace's `license = "Apache-2.0"` metadata. Its
  manifest currently omits a package license and its local `kmipkit-ttlv`
  path dependency omits the version already declared by the path package.
  cargo-deny 0.20.2 reports these as an unlicensed local package and a
  wildcard dependency. The T001 baseline and full license-ID inventory are in
  [dependency-review.md](dependency-review.md) and `evidence/`.
- `docs/development/coding-standards.md` requires every new dependency review
  to cover capability, alternatives, maintenance, security history, MSRV,
  license, platform support, and transitive cost. Published builds may not use
  Git dependencies.
- `docs/security/threat-model.md` rates a compromised dependency/source as a
  high-impact supply-chain threat. Existing reviews of `zeroize` and
  QuickCheck apply only to those exact dependency scopes; they do not define a
  repository-wide license allowlist or exception policy.
- The previous CI feature's T016 evidence records successful advisory, ban,
  and source checks with cargo-deny, while noting that the license phase still
  needs a repository allowlist. This is historical evidence, not a current
  clean scan of both lockfiles.

## Official tooling research

### Decision: use a pinned `cargo-deny` release as the unified check

The official cargo-deny documentation lists four check families: advisories,
licenses, bans/duplicates, and sources. Its advisory check uses RustSec and
can report vulnerable, unmaintained, unsound, and yanked packages. Its license
check evaluates SPDX metadata and license files against an explicit configured
allowlist. Its source check can allow specific registries and Git locations.

Sources:

- [cargo-deny supported checks](https://embarkstudios.github.io/cargo-deny/checks/index.html)
- [cargo-deny CLI and locked installation](https://embarkstudios.github.io/cargo-deny/cli/index.html)
- [cargo-deny license configuration](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html)
- [cargo-deny advisory configuration](https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html)
- [cargo-deny source checks](https://embarkstudios.github.io/cargo-deny/checks/sources/index.html)
- [cargo-deny ban configuration](https://embarkstudios.github.io/cargo-deny/checks/bans/cfg.html)
- [cargo-deny graph, feature, target, and configuration discovery](https://embarkstudios.github.io/cargo-deny/checks/cfg.html)
- [cargo-deny source policy configuration](https://embarkstudios.github.io/cargo-deny/checks/sources/cfg.html)

The tool is suitable for a unified policy job because one checked-in config
can cover all four existing every-PR policy categories. Exact severity values,
version pin, install method, and target list still require the independent
dependency/tool review in T001. The official CLI documentation supports
installing a specific crate release with `cargo install --locked` and also
describes precompiled releases. This plan avoids a third-party setup action.
The current upstream release candidate is `cargo-deny` 0.20.2; its upstream
manifest declares Rust 1.88 and `MIT OR Apache-2.0`, which is compatible with
the repository's Rust 1.94 toolchain requirement. T001 records an independent
review before the exact pin is added to workflow code.

### Alternatives considered

- **`cargo-audit` in addition to cargo-deny**: it audits `Cargo.lock` against
  the RustSec database but does not cover licenses, bans, duplicates, or
  sources. Adding both creates overlapping advisory checks against the same
  upstream database, so it is not required for these acceptance criteria.
  Source: [RustSec cargo-audit README](https://github.com/rustsec/rustsec/blob/main/cargo-audit/README.md).
- **`cargo-vet`**: a distinct audited-dependency workflow that is not required
  by the existing testing guide's four concrete policy classes. It would add
  audit attestations and maintenance work outside this bounded infrastructure
  gap; defer unless a later reviewed requirement asks for it.
- **A third-party GitHub Action**: rejected for this feature because the
  existing workflow avoids setup actions and requires full-SHA pinning. A
  direct exact-version install keeps the tool version explicit in repository
  code and avoids granting extra action permissions.
- **Only `cargo-audit`**: rejected because it leaves license, source, ban, and
  duplicate policy unenforced.

## Policy decisions

1. **Lockfiles and target coverage**: run against both root and fuzz graphs;
   enable all features, include development dependencies in license and
   duplicate checks, and represent every target triple used by the core CI
   matrix. T001 records the exact target strings from the workflow.
2. **Licenses**: use a finite SPDX allowlist and explicitly include dev
   dependencies; no wildcard or blanket
   permissive-license classifier. T001 inventories current graph evidence.
   Unknown or absent license data fails. Entries need exact reviewed evidence;
   project `Apache-2.0` metadata is not a third-party license approval.
3. **Graph**: enable all declared features and include every target triple in
   the core CI matrix; do not filter target-specific packages by the Linux
   policy runner's host.
4. **Advisories**: vulnerability advisories are errors; explicitly set
   `unsound = "all"`, `unmaintained = "all"`, and `yanked = "deny"`. An
   exception must be exact, documented, and expire within 90 days. Do not use
   offline/frozen mode. Each successful workspace invocation emits the actual
   RustSec database Git SHA and commit timestamp it fetched; an unavailable
   refresh is a failed scan.
5. **Sources and package bans**: explicitly allow the crates.io registry,
   deny unknown registries and all Git sources, and require `rev` on any future
   Git exception. Deny configured architectural bans, wildcard requirements,
   and duplicate versions, including dev-only duplicates. Ban exactly the
   crates `native-tls`, `openssl`, and `openssl-sys` under ADR-0005's rustls
   TLS-backend decision; no wildcard or pattern bans are introduced.
   Cargo-deny's source check does not constrain local paths; the metadata validator accepts
   source-less packages only when their canonical manifest is a member of the
   root/fuzz workspace union and is inside the checkout.
6. **Scheduled evaluation**: retain the existing daily event; add a policy job
   that checks out the active release branch. GitHub starts scheduled workflow
   runs from the default branch, so the scheduled code becomes live when the
   workflow is integrated there. Pull-request checks refresh advisories before
   that integration point.
7. **License tool limits**: cargo-deny trusts crate SPDX metadata and known
   license files; it does not exhaustively inspect all source files in every
   dependency. Documentation must state this limitation and must not claim a
   complete legal license audit.
8. **Separate controls**: keep immutable OASIS inputs, generated catalog
   checks, action SHA pinning, and release signatures/SBOM/provenance separate.
   Do not alter branch protection or claim a failing job is a required check.
9. **First-party fuzz metadata**: after test-only Red commits, add the
   repository's declared `Apache-2.0` license to its unpublished fuzz package
   and add an explicit version requirement matching the current local
   `kmipkit-ttlv` package. Keep the resolved versions and both lockfiles
   unchanged. This is not an approval of any third-party license identifier.

## Clarification outcome

The existing documents settle why this feature exists and its non-goals; the
official tool documentation settles the selected checker and supported policy
families. The only exact values deferred to T001 are evidence-derived inputs
(tool release, platform targets, current license/source/advisory baseline), not
open behavior choices. No protocol-standard interpretation is involved.
