# KMIPKIT-0011 Implementation Gate Record

**Gate**: T002
**Disposition**: Passed with evidence limitations recorded below
**Recorded**: 2026-10-05

## Approved specification revision

The feature specification was merged by maintainer `NeverWe1come` in PR
[#35](https://github.com/NeverWe1come/KMIPKit/pull/35). Its specification
commit is `6ff949c5f99ae4c7824c60d06e2fc82fabde5e8a8`, contained in merge
commit `59b23ed18713c2e8c59bd4ec0f09751653a46b1a`. The merge commit is an
ancestor of the active release base
`ee4c6aedcf2c7d8c27cb131faaf6daeaf2587c1e` (`release/1.0.0`). The active
release reference was confirmed against `origin` on the recorded date.

No new architectural decision is required. ADR-0005 remains the authority
for the exact `native-tls`, `openssl`, and `openssl-sys` bans; this feature
does not change the selected TLS backend or any product boundary.

## Baseline dispositions

The current report-only scans cover both locked workspaces and leave
cargo-deny's target graph unfiltered. The root graph passes. The fuzz graph
has exactly two findings, both assigned to FR-013 and the Red/Green sequence
in T003/T005/T006:

1. `kmipkit-ttlv-fuzz@0.0.0` has no declared package license. T006 adds the
   repository's existing `Apache-2.0` metadata after a failing test.
2. The fuzz package's local `kmipkit-ttlv` path edge has no version
   requirement and is reported as a wildcard. T006 adds the matching local
   crate version after a failing test. FR-006 makes this non-waivable.

Neither graph reported an advisory, unapproved source, ADR-0005 hard-ban,
or duplicate-version finding. Both lockfiles and resolved package versions
remain unchanged.

## SPDX inventory review

The unfiltered root inventory reports `Apache-2.0`, `MIT`, `Unicode-3.0`,
`Unlicense`, and `LGPL-2.1-or-later`. The fuzz inventory reports
`Apache-2.0`, `MIT`, `NCSA`, and `LGPL-2.1-or-later`. Cargo-deny's
`Unlicensed` marker on the fuzz package is missing metadata, not an SPDX ID.
The package expressions containing conjunctions are preserved in
`dependency-review.md`: `unicode-ident` requires an allowed choice from
`MIT OR Apache-2.0` plus `Unicode-3.0`, and `libfuzzer-sys` requires an
allowed choice from `MIT OR Apache-2.0` plus `NCSA`. The UEFI-only `r-efi`
expression is `MIT OR Apache-2.0 OR LGPL-2.1-or-later`; the candidate checks
pass through its MIT/Apache alternatives without adding LGPL to the finite
allowlist.

The report-only candidate allowlists are therefore root:
`Apache-2.0`, `MIT`, `Unicode-3.0`, `Unlicense`; fuzz: `Apache-2.0`, `MIT`,
`NCSA`. The review checks identifier meaning and Cargo expression structure
against the linked SPDX/Cargo records. It is technical inventory review, not
legal advice or a complete source-code license audit; cargo-deny's metadata
limitations remain documented.

## Target coverage and recorded limitation

The core CI runner-host set is recorded in `dependency-review.md` from the
workflow, runner job metadata, and GitHub's published hosted-runner
architectures. The exact hosted and self-hosted `rustc -vV` output could not
be retrieved: the repository Actions log endpoint returned HTTP 403 because
the configured API access lacks repository administrator rights. The
self-hosted runner's `Linux, ARM64` labels do not independently identify its
libc ABI.

The policy graph has no target filter, so it checks all resolved
target-specific dependency edges and cannot omit a supported CI target due to
an incomplete target list. T003/T008 add a fail-closed runtime `rustc -vV`
host check against the recorded core CI runner set; the implementation and
CI evidence must preserve the actual host output and must not report inferred
triples as direct observations. This evidence gap does not leave a target
edge out of the candidate dependency scan.

## Gate checklist

| T002 condition | Evidence | Result |
|---|---|---|
| Human-merged specification revision is on active release | PR #35 and merge ancestry above | Pass |
| Exact license-ID inventory reviewed | All-target raw inventories, full expressions, and technical SPDX review above | Pass; no legal audit claim |
| No unaddressed ADR-0005 hard-ban finding | Both report-only scans | Pass |
| Every baseline finding has an in-scope disposition | Both fuzz findings map to FR-013 Red/Green tasks | Pass |
| Root and fuzz workspaces are covered | Separate locked metadata and cargo-deny invocations | Pass |
| All current CI targets are covered | Unfiltered cargo-deny graph; runtime host check required by T008 | Pass; direct host-output limitation retained |

T002 permits starting the Red phase. The target-output limitation and the
nonexistent cargo-deny `--disable-fetch` wording correction are carried into
the implementation changes and final PR review; neither is treated as a
policy exception.
