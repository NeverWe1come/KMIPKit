# KMIP 2.1 client profile evidence

This guide explains how to read KMIPKit's client profile evidence and use its
explicit profile-validation entry point. Runtime selection fails closed today:
the typed-rule registry contains a mapping for Wrapped Secret Data under §2.7,
but Baseline still lacks complete applicable rule and test coverage, so no
profile is yet available for a successful exchange.

## Explicit per-exchange selection

Pass a configured client, a typed request batch, and codec limits to a helper
that selects a profile for one exchange. This complete Rust function compiles
as a rustdoc example; its body returns the structured success or failure to
the caller:

```rust,no_run
use kmipkit::{
    Client, ClientBatch, ClientBatchResponse, ClientError, ProfileValidationSelection,
};
use kmipkit_ttlv::codec::CodecLimits;

fn execute_for_profile(
    client: &mut Client,
    batch: ClientBatch,
    limits: &CodecLimits,
) -> Result<ClientBatchResponse, ClientError> {
    let selection = ProfileValidationSelection::new(["KMIPKIT-PROFILE-BASELINE-CLIENT"]);
    client.execute_with_profiles(batch, &selection, limits)
}
```

The method validates applicability, dependencies, and complete typed rule
coverage before any transport write. With the current catalog and rule
registry, Baseline Client fails closed and the error reports stable profile,
requirement, and source-clause IDs with `NotSent` delivery state. Inspect the
structured error with `ClientError::profile_validation_error()`. The ordinary
`Client::execute` method remains unprofiled and does not select a hidden
default. A successful profiled exchange will become available only after the
applicable typed rules and their verification tests are registered.

## Current status

KMIP Specification v2.1 §14.1 requires a client to conform to one or more
client profiles. KMIPKit records Baseline Client as the initial 1.0 target for
future evidence work. Target membership is a project planning choice; it does
not establish profile support or satisfy the specification requirement.

The Baseline Client catalog state is `evidence_incomplete`. All 16 mandatory
official fixtures are pinned. T036 records 13 full deterministic client
replays and 3 typed preflight blockers. The replays do not establish server
interoperability or profile conformance; the blocked cases and other evidence
gaps remain open. Seventy-seven other linked profile cases have no local
fixture. KMIPKit does not recreate or infer absent fixture contents.

The typed mapping for archived Locate retrieval records a caller-directed
workflow only. Profile preflight checks one exchange at a time; it does not
enforce or perform the required Recover-then-Get sequence from KMIP
Specification v2.1 §6.1.28. The caller must issue Recover after Locate
succeeds, then Get after Recover succeeds.

The Recover profile mappings also record optional caller-directed follow-ups:
the caller can issue Poll with the correlation value when Recover returns
Pending, and can issue Get after Recover succeeds (KMIP Specification v2.1
§6.1.42). Profile preflight does not poll, repeat Poll, or issue Get
automatically, and it does not guarantee server completion.

The Baseline Client mapping for Derive Key records the caller's duty to choose
a source Managed Object whose Cryptographic Usage Mask includes the Derive Key
bit (KMIP Specification v2.1 §6.1.14). That mask belongs to server-held object
state. Profile preflight does not fetch or attest it, and does not make a
Derive Key request eligible by itself.

KMIP Specification v2.1 §§6.1.45–6.1.47 recommends performing Re-certify,
Re-key, and Re-key Key Pair at most once for the same existing certificate,
key, or key pair. KMIPKit does not keep cross-call object history, so the
application remains responsible for avoiding repeated replacement operations.
When an application needs new client credentials, it must explicitly issue
Re-Provision under §6.1.48. KMIPKit does not infer credential expiry or invoke
Re-Provision automatically. These caller-directed mappings do not establish
server acceptance or profile conformance.

Other Baseline mappings record caller responsibilities that depend on remote
object state or application time: Activation Date (§4.1), Deactivation Date
(§4.18), the effective Cryptographic Usage Mask (§4.17), lease expiry
(§§4.30 and 6.1.35), and Usage Allocation and its date window (§6.1.23).
Profile preflight does not retrieve that state, track it across calls, or
enforce these conditions for ordinary or raw requests. For Deactivation Date,
the caller must also establish any extraordinary circumstance and special
permission before using the stated exceptions. These static mappings record
caller duties; they do not establish server acceptance, an official Test Case
pass, or profile conformance.

## Reading profile status

The [generated coverage report](../../../specification/catalog/coverage-report.md)
shows profile applicability, selected target, runtime rule coverage, catalog
evidence state, linked tests, and fixture availability as separate facts:

- **Applicability** describes the profile's scope in KMIPKit 1.0. A profile
  outside `client_1_0` may be server-only, conditional, or deferred.
- **Target selection** identifies a profile whose evidence the project plans
  to establish. It does not turn the profile into a runtime option.
- **Runtime validation** requires a complete typed mapping and an
  executable, keyword-appropriate test for every applicable requirement.
  Missing or incomplete mappings keep selection unavailable.
- **Evidence state** reflects the catalog's recorded evidence. A local report
  cannot upgrade it, and stale `evidence_complete` data is treated as
  incomplete.
- **Public claim** is a separate release decision. This feature does not
  publish a conformance claim.

Use the linked clause and requirement IDs to inspect the source obligations.
Catalog summaries are descriptive metadata; they are never interpreted as
executable validation rules. A server's Query or Discover Versions response
also does not establish profile support.

## Claim readiness

A profile can become eligible for a later claim only after every applicable
requirement and mandatory official test has current evidence. Unavailable
fixtures, missing implementation or verification links, incomplete typed
rules, unsatisfied profile conditions, and unmet profile dependencies keep the
profile ineligible. An eligible evidence result still does not publish a
claim; publication belongs to the separately reviewed release process.

See the [conformance strategy](../../compliance/conformance.md), the
[KMIP 2.1 inventory](../../../specification/catalog/README.md), and the
[KMIPKIT-0010 specification](../../../specs/010-profile-conformance/spec.md)
for the evidence model and its scope.
