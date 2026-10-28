# KMIP 2.1 client profile evidence

This guide explains how to read KMIPKit's client profile evidence and use its
explicit profile-validation entry point. Baseline Client runtime selection is
available for its 294 mapped requirements, including caller-state rules for
KMIP Specification v2.1 §§4.38 and 4.59. This enables typed rule validation;
it does not establish official conformance evidence or server acceptance.

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
coverage before any transport write. This context-free entry point supplies no
remote object state, so requests that require such facts fail closed. Use the
context-aware entry point below for those requests. Any request can still fail
a rule check before transport I/O; inspect structured failures with
`ClientError::profile_validation_error()`. The ordinary `Client::execute`
method remains unprofiled and does not select a hidden default. Passing local
checks does not establish server interoperability, official test-suite
success, or profile conformance.

## Caller-attested object state

For rules that depend on remote object state, supply a separate context for
this exchange. Bind each state entry to the zero-based batch item and the exact
Text String Unique Identifier used by that request:

```rust,no_run
use kmipkit::{
    Client, ClientBatch, ClientBatchResponse, ClientError, ProfileObjectState,
    ProfileValidationContext, ProfileValidationSelection,
};
use kmipkit_ttlv::codec::CodecLimits;

fn execute_with_object_state(
    client: &mut Client,
    batch: ClientBatch,
    object_id: &str,
    limits: &CodecLimits,
) -> Result<ClientBatchResponse, ClientError> {
    let selection = ProfileValidationSelection::new(["KMIPKIT-PROFILE-BASELINE-CLIENT"]);
    let context = ProfileValidationContext::new().with_object_state(
        0,
        ProfileObjectState::new(object_id)
            .with_original_creation_date_present(Some(true)),
    );
    client.execute_with_profiles_and_context(batch, &selection, &context, limits)
}
```

Each fact uses `Option`: `Some(true)` and `Some(false)` are caller assertions;
`None` means unknown. Missing, unknown, duplicate, or mismatched entries fail
before transport writes. KMIPKit does not query or cache state, infer it from
another batch item, or verify whether a caller assertion is true. Identifiers
and state values are redacted from validation errors and debug output.

For Original Creation Date, KMIP Specification v2.1 §4.38 permits Add when the
caller attests that the attribute is absent. Table 106 marks it non-modifiable
and non-deletable by the client, so Modify, Delete, and Set are rejected even
when the caller reports it absent; after it is set, §4.38 also prohibits
updates or deletion. For Usage Limits, §4.59 and Table 149 prohibit a Total
change after protection use begins or after Get Usage Allocation under
DEC-024. A Set that changes Total requires known-present Usage Limits and the
current Total and Count, preserves Count, and rejects a malformed structure
that omits a required member from §7.40 Table 392. The separate KMIPKIT-0016
restriction on Add/Modify Usage Limits remains in force. These checks enforce
the rule against caller-supplied facts; the application is responsible for
providing trustworthy state.

## Current status

KMIP Specification v2.1 §14.1 requires a client to conform to one or more
client profiles. KMIPKit records Baseline Client as the initial 1.0 evidence
target. Target membership is a project planning choice. Runtime validation is
available for Baseline Client's 294 mapped requirements. This enables local
rule checks; it does not establish profile support or satisfy the
specification's conformance requirement.

The Baseline Client catalog state is `evidence_incomplete`. All 16 mandatory
official fixtures are pinned. T036 records 13 full deterministic client
replays and 3 typed preflight blockers. Separately, the broader Profiles
catalog contains 77 linked case records without local fixture files (70
mandatory, 7 optional); none belongs to Baseline's 16-case list. These fixture
and replay gaps remain independent from local runtime validation. KMIPKit does
not recreate or infer absent fixture contents.

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
(§4.18), Process Start Date (§4.40), Protect Stop Date (§4.41), the effective
Cryptographic Usage Mask (§4.17), lease expiry (§§4.30 and 6.1.35), and Usage
Allocation and its date window (§6.1.23).
KMIP Specification v2.1 §4.34 also recommends one cryptographic usage per
object and says the server does not enforce it. KMIPKit records this as a
caller-owned client recommendation; the application should choose and preserve
the intended single usage. KMIPKit does not inspect remote object state or
enforce the usage count. The mandatory Hashed Password Timestamp monotonicity
requirement (§9.11) is mapped to a caller-owned sequence helper under the
resolved KMIPKIT-0008 OD-003 decision. It requires strictly increasing
caller-supplied timestamps per Username and logical credential producer; the
application manages persistence across restarts. The helper does not compute
hashes or send credentials, and the mapping does not establish wire ordering
or full profile conformance. The OPTIONAL §9.20 relative Date capability is
separate: if the application uses it, it provides the countdown timer and
performs subtraction using the exposed Time Stamp and Date-Time values.
Sections 4.40 and 4.41 also define when a caller may use an object for
protected-information processing, when it must stop applying protection, and
the allowed relationships to Activation and Deactivation Dates. These duties
depend on the caller's current time, local cryptographic use, and object state.
The paragraphs do not assign which endpoint performs protected-information
processing; the attribute-rule tables separately allow initial setting by the
server or client and describe client mutability. KMIPKit does not perform those
cryptographic operations, read a clock, or retrieve remote object attributes,
so the mappings document caller duties without enforcing them. Profile
preflight does not retrieve that state, track
it across calls, or enforce these conditions for ordinary or raw requests. For
Deactivation Date, the caller must also establish any extraordinary
circumstance and special permission before using the stated exceptions. These
static mappings do not establish server acceptance, an official Test Case
pass, or profile conformance.

KMIP Specification v2.1 §4.57 paragraph 2 prohibits using a Pre-Active object
for any cryptographic purpose but does not identify an actor or message
direction. KMIPKIT-DEC-020 interprets this as server-only for the KMIPKit 1.0
client: §4.57 says State is known to and changed only by the server, and
§6.1.17 defines Encrypt as a client request for the server to perform
encryption. The source clause remains in client profile clause closures, but
the requirement is excluded from client requirement closures and runtime
rules. KMIPKit does not query remote State, perform local cryptography, or
claim to enforce this server-side restriction.

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
  Baseline Client currently meets this runtime gate; its official evidence
  status remains incomplete.
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
