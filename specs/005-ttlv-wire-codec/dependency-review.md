# Client dependency review: KMIP TTLV wire codec

**Status**: Pending; no dependency or manifest change is approved by this record.

Complete this record and obtain an independent review before any change to
`crates/kmipkit-client/Cargo.toml`. Cover every newly direct dependency of
`kmipkit-client`, including internal workspace crates and test-only packages.
For each entry record its capability, alternatives, maintenance, security
history, MSRV, license, supported platforms, transitive cost, exact package
version, and exact enabled Cargo features. Record whether it is test-only or a
normal dependency and the reason for that scope.

## Planned direct dependency candidates

This inventory reflects the current task proposal only. It does not approve
these dependencies, versions, features, or manifest changes.

| Package | Proposed scope and pin | Review status |
| --- | --- | --- |
| `kmipkit-ttlv` | Workspace dependency; current workspace version `0.1.0`; initially proposed as a client dev-dependency, then a normal dependency in T003 | Pending full review |
| `zeroize` | Workspace dependency `=1.9.0`, `default-features = false`, feature `alloc`; initially proposed as a client dev-dependency, then a normal dependency in T003 | Pending client-use rationale and independent review |
| `static_assertions` | Test-only, proposed pin `=1.1.0`; enabled features must be recorded before manifest changes | Pending full review |
| `serde` | Test-only, proposed pin `=1.0.228`; enabled features must be recorded before manifest changes | Pending full review |

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
