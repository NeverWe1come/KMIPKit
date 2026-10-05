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
| `kmipkit-ttlv` | Workspace dependency; current workspace version `0.1.0`; Cargo metadata reports no declared features, so proposed default selection resolves to no enabled crate features; initially proposed as a client dev-dependency, then a normal dependency in T003 | Pending full review |
| `zeroize` | Workspace dependency `=1.9.0`, `default-features = false`, feature `alloc`; initially proposed as a client dev-dependency, then a normal dependency in T003 | Pending client-use rationale and independent review |
| `static_assertions` | Test-only, proposed pin `=1.1.0`; cached package metadata declares only optional `nightly` and no default feature, so proposed default selection resolves to no enabled crate features (`nightly` disabled) | Pending full review |
| `serde` | Test-only, proposed pin `=1.0.228`; the existing TTLV dev-dependency and `cargo tree --locked -e features` show the unqualified pin's default selection enables `std`; proposed client selection is also default/`std`, with optional `alloc`, `derive`, `rc`, and `unstable` disabled | Pending full review |

The metadata basis is the current workspace `Cargo.toml`, `crates/kmipkit-ttlv/Cargo.toml`, `Cargo.lock`, Cargo workspace metadata/tree output, and locally cached package metadata for `serde` 1.0.228 and `static_assertions` 1.1.0. Reconfirm these selections before the future manifest edits; this evidence only resolves the proposed feature sets and does not complete package rationale or independent review.

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
| `kmipkit-ttlv` | Pending | Pending | Pending | Pending | Pending |
| `zeroize` | Pending | Pending | Pending | Pending | Pending |
| `static_assertions` | Pending | Pending | Pending | Pending | Pending |
| `serde` | Pending | Pending | Pending | Pending | Pending |

Do not treat the candidate rationale in `tasks.md` or the prior 004 review as
completion of this table. The reviewer-owned design checklist's dependency
review item remains open until this record is complete and independently
reviewed.
