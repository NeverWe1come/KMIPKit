# Cargo dependency policy

KMIPKit applies one reviewed dependency policy to both Cargo workspaces. The
policy is enforced by `cargo-deny` **0.20.2**, pinned and installed with
`cargo install --locked --version 0.20.2 cargo-deny`. The tool is a CI/local
development tool; it is not a KMIPKit runtime or published dependency. Its
independent engineering review is recorded in the
[cargo-deny 0.20.2 review](../../specs/011-dependency-policy/dependency-review.md).

## What each run checks

Run the policy from the repository root in PowerShell:

```powershell
pwsh -File .\scripts\Test-DependencyPolicy.ps1
```

The runner checks `Cargo.toml` with `Cargo.lock` and `fuzz/Cargo.toml` with
`fuzz/Cargo.lock` as separate workspaces. It enables all declared features,
includes development dependencies, and leaves Cargo metadata and cargo-deny's
dependency graph unfiltered by target. This includes target-specific edges
present in the resolved graphs, even when the policy job itself runs on Linux.
The runner captures the actual `rustc -vV` host triple and fails closed if it
is absent from the reviewed CI runner set. It also checks source-less Cargo
metadata packages against the canonical member-manifest union of the two
workspaces, rejecting paths outside the checkout and symlink escapes.

Each workspace invocation uses cargo-deny online so it attempts to refresh
the RustSec advisory database. After each successful check, the runner
verifies the configured RustSec database remote and reports that workspace's
database commit SHA and ISO timestamp separately. An unavailable advisory
source, failed refresh, missing or mismatched database evidence, policy
finding, missing/stale lockfile, invalid workspace path, tool-install or
version failure, or host triple outside the reviewed set makes the command
fail. Cached advisory data alone is not a successful fresh scan. The check
does not update manifests or lockfiles; it uses locked operations and compares
lockfile hashes so an accidental change fails.

On a cargo-deny failure, the runner requests its JSON diagnostic format and
prints only validated package coordinates, rule codes, known advisory
identifiers, and a source reference with URL paths and credentials redacted.
For license findings, it prints a bounded ASCII expression from Cargo
metadata only when every identifier appears in the structured license
inventory emitted by the pinned cargo-deny binary for that workspace. Custom
license references, unrecognized identifiers, malformed expressions, and
non-ASCII separators become `license=unavailable`. The runner omits
cargo-deny's label text and never forwards its raw message, stdout, or stderr.
It obtains the license inventory with cargo-deny's offline `list` command;
advisory scans still run online and refresh RustSec as described above. If a
structured field is missing or malformed, the report uses a generic safe
placeholder while preserving the failed check and exit code. Exact-exception
validation lists every unmatched finding, even when the register is empty,
and includes the same safe source, advisory, and license evidence.

Tool upgrades require a reviewed change that updates the exact runner pin,
the documented local version, and this independent tool review together.
Re-run the policy against both workspaces and verify the new release,
checksum, toolchain requirements, and advisory behavior before changing the
pin.

The [exception register](../../specification/compliance/dependency-policy-exceptions.json)
is the source of exception records. `.cargo/deny.toml` carries only the
corresponding tool-native entries. `.cargo/deny-baseline.toml` preserves the
same policy with registered waivers removed. The runner scans both workspaces
with that baseline first, matches every structured finding exactly to the
register, then runs the configured policy and reports accepted exception IDs
only after both scans pass. This prevents a global advisory ignore or a
duplicate skip from hiding a finding for another exact package/version.

cargo-deny also discovers local exception files from manifest ancestors even
when `--config` is supplied. The runner rejects `deny.exceptions.toml`,
`.deny.exceptions.toml`, and `.cargo/deny.exceptions.toml` anywhere in either
workspace's ancestor chain, so they cannot add unregistered license
allowances.

## Exception review lifecycle

Fix or remove the dependency first when practical. An exception is a temporary
reviewed disposition for an exact remaining finding, not a general approval
for a dependency. Add an entry only after a reviewer has accepted the
specific risk and recorded durable approval evidence. Each entry must identify:

- A unique `KMIPKIT-0011-EX-NNN` ID and one supported rule kind: `advisory`,
  `yanked`, `license`, `source`, or `duplicate`.
- The exact crate name and resolved version, plus the exact advisory ID,
  yanked package/version, source, or license evidence required by that rule.
  Wildcards, ranges, and broad package matching are invalid. Yanked exceptions
  use cargo-deny's exact `crate@version` ignore form; they do not invent an
  advisory identifier.
- A specific rationale, mitigation/remediation path, accountable owner, and
  a reviewer different from that owner.
- An ISO review date, an expiry no more than 90 days later, and an approval
  reference to a durable review record such as an ADR or merged PR.

For a license clarification, the record must include human-reviewed evidence
for that package and its license disposition. A missing automated license
field is not itself evidence that the package is acceptable. Source records
must not contain credentials or secret URL parameters. The validator rejects
source URLs with credentials or secret-bearing query parameters, and its
diagnostics do not expose those secret values.

Keep the register record and the exact cargo-deny configuration entry in sync
in the same change. Run the policy command and include its output in the
review. The validator rejects incomplete or duplicate IDs, future or invalid
review dates, expired entries, expiry beyond 90 days, owner/reviewer
collisions, orphaned entries, mismatched package/version/rule findings, and
any attempt to widen an exception to another package or version.

Before expiry, either remove the dependency/finding and both exception entries,
or submit a fresh review with current evidence, mitigation, and a new expiry.
Renewal is a new human decision; do not extend a date mechanically. Expired
records fail the check even if cargo-deny still reports the finding as
waived. Remove the exception from both files when the finding is resolved.

Exceptions cannot waive wildcard version requirements, paths outside the
canonical workspace-member union, symlink escapes, or the architecture bans
below. The broad cargo-deny `bans.skip-tree` setting is rejected because it
can suppress duplicate findings across a dependency subtree that is not
represented by one exact package/version exception. A source exception, if
later accepted, must use an immutable Git revision. Blanket ignores are not
supported.

## Finite architecture bans

The policy bans exactly `native-tls`, `openssl`, and `openssl-sys`, as required
by [ADR-0005](../adr/0005-transport-and-tls.md)'s selection of
the rustls TLS backend with aws-lc-rs. These names enforce that specific
architecture decision. They are a finite list of exact crate names, not a
pattern intended to ban unrelated packages. Do not add a general-purpose ban
list in this policy change, and do not add exceptions for these three crates.
Any change to the TLS backend boundary requires a separately reviewed
specification and ADR.

Other checks deny wildcard dependency requirements and duplicate versions,
including development dependencies; any duplicate exception must cover the
exact affected package versions. Registry and Git sources are denied unless
explicitly configured under the reviewed source policy. Only the reviewed
crates.io registry is allowed by default.

## License evidence limits

The configured finite SPDX allowlist applies to license expressions reported
by package metadata and the license files cargo-deny recognizes. The policy
also explicitly sets `licenses.private.ignore = false`, so unpublished
workspace packages receive the same license checks. cargo-deny
does not read and legally analyze every source file in each third-party
package, prove that metadata is complete or correct, or decide whether a
license is acceptable for a particular use. A passing result is automated
metadata evidence, not legal advice, a complete source-code license audit, or
a legal validation of the allowlist.

An unknown or missing license remains a failure unless a reviewer records an
exact package/version clarification with human-reviewed license evidence and
approval in the exception register. KMIPKit's own Apache-2.0 license does not
approve third-party licenses. License IDs observed in the
[tool review inventory](../../specs/011-dependency-policy/dependency-review.md)
are diagnostic inputs and must not be treated as legal conclusions.
