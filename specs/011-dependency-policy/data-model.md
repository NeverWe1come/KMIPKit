# Data Model: Dependency Policy Gates

This feature models repository policy data only. It adds no runtime KMIP
types, protocol fields, or serialized network data.

## Entities

### Dependency graph

| Field | Meaning | Validation |
|---|---|---|
| workspace | Root or fuzz Cargo workspace | Must match one of the two committed lockfiles |
| manifest | Workspace root manifest path | Must resolve beneath the checkout |
| lockfile | Resolved package/version/source graph | Must exist, be tracked, and remain unchanged during the scan |
| targets | Rust target triples represented by core CI | Every supported matrix target is included in policy resolution |
| dependency kinds | Normal, build, development, and target-specific edges | No kind is omitted from policy review |
| workspace members | Canonical member manifest paths from root and fuzz Cargo metadata | Path packages must match the member union and remain inside the canonical checkout root |

### Policy finding

| Field | Meaning | Validation |
|---|---|---|
| rule | Advisory, license, ban, duplicate, wildcard, or source policy | Must map to a configured check class |
| package | Crate name and resolved version | Must come from Cargo metadata/lockfile |
| source | Registry, Git revision, or local workspace path | Must match the source rule or an exact exception; emitted evidence removes URL user-info and secret query values |
| evidence | SPDX expression, advisory ID, duplicate set, or source record | Must be included in CI output without secret data |
| disposition | Denied or accepted by a recorded exception | No implicit suppression |

### Policy exception

The machine-readable exception register is checked in at
`specification/compliance/dependency-policy-exceptions.json`. Its initial list
is empty. Any later entry has this logical schema:

| Field | Meaning | Validation |
|---|---|---|
| id | Stable `KMIPKIT-0011-EX-NNN` identifier | Unique and referenced by the policy config |
| kind | `advisory`, `license`, `source`, or `duplicate` | Exact known value; architecture bans and wildcard requirements cannot be excepted |
| package | Exact crate name | No wildcard |
| version | Exact resolved package version | Required; no range or wildcard |
| advisory_id | Exact RustSec ID for advisory exception | Required only for advisory kind |
| source | Exact source or immutable Git revision for source exception | Required for source kind |
| license_evidence | Human-reviewed license evidence and disposition | Required only for a license clarification; a waiver alone is insufficient |
| rationale | Why the finding cannot be removed now | Non-empty and specific |
| mitigation | Compensating control or remediation path | Non-empty |
| owner | Person or team responsible for remediation before expiry | Required; distinct from reviewer |
| reviewed_by | Review identity | Non-empty; not inferred from author |
| reviewed_on | ISO date of decision | Valid date, not future-dated |
| expires_on | ISO date at most 90 days after review | Must be future-dated and within the maximum window |
| approval_ref | Review record, ADR, or merged PR link | Must resolve to durable project evidence |

Source exceptions must store a canonical source identifier without embedded
credentials. Secret-bearing URL user-info or query parameters are invalid and
must never be echoed in diagnostics.

The validator checks exact matching, field completeness, date ordering,
maximum expiry, and correspondence between register entries and actual policy
exceptions. Expired or unmatched entries fail CI. An exception never grants
permission to another package/version or broader source.

### Policy run

| Field | Meaning | Validation |
|---|---|---|
| kind | Pull-request or scheduled run | Selects merge-tree or active-release checkout |
| tool_version | Exact reviewed cargo-deny release | Must match the checked-in pin |
| workspace_results | Result per root/fuzz workspace and target set, including online refresh status and RustSec commit SHA/timestamp | Both workspaces must be present; each recorded revision is the one fetched by that successful invocation |
| status | Pass or fail | No skipped or unknown status can count as pass |
