# Git, versioning, and releases

## Branches

- `master`: protected, stable, and publishable.
- `release/<version>`: integration branch for one release.
- `feature/<spec-id>-<slug>`: one approved specification.
- Hotfix conventions will be defined when a public release requires them.

No direct pushes to master or release branches. Agents open draft PRs; humans
approve and merge. Feature PRs squash merge so one specification becomes one
logical release-branch commit.

## Commits and changelog

- Conventional Commit titles for final/squash commits.
- Development commits preserve Red, Green, and Refactor evidence.
- Each user-visible PR supplies a categorized changelog fragment.
- Release automation assembles Keep a Changelog output.

## Versioning

KMIPKit uses SemVer. Official Rust crates, ABI package, Java, Python, and C
artifacts share one version. Adapters verify native compatibility at load time.

- 0.x may contain documented breaking changes.
- 1.x preserves public API and ABI compatibility.
- Compatible additions use a minor release.
- Fixes use a patch release.
- Removal or incompatible change uses a major release and migration guide.

The MSRV is fixed for a published release. Raising it requires a documented
minor release and remains at least six months behind current stable when
selected.

## Compatibility checks

- Rust: cargo-semver-checks.
- C: header, symbol, and versioned layout comparison.
- Java: japicmp.
- Python: public API snapshot and import compatibility tests.
- Error codes, catalog values, and package contents receive snapshots.

## Distribution

- crates.io for public Rust runtime crates.
- Maven Central for Java API and platform native JARs.
- PyPI for Python wheels and sdist.
- C archives with CMake and pkg-config.
- GitHub Releases for native archives, signatures, hashes, SBOM, provenance,
  and release notes.

All publication occurs from CI after a protected signed tag. No official
artifact is published from a developer workstation. Candidate artifacts remain
private until the 1.0 public launch.

## Package integrity

- Cargo lockfile committed.
- GitHub Actions pinned by commit SHA.
- No runtime native downloads.
- Trusted publishing or short-lived CI credentials where supported.
- SHA-256 and signatures for downloadable artifacts.
- Reproducibility tested for release candidates where toolchains permit it.

## Release readiness

Release notes contain scope, conformance profiles, tested servers/platforms,
security review status, known limitations, compatibility changes, and upgrade
instructions. Version 1.0 additionally requires all gates in the roadmap.
