# Conformance and traceability strategy

## Claims

KMIPKit distinguishes:

- Protocol representation: a value or operation can be expressed.
- Behavioral implementation: requests and responses behave according to the
  applicable clauses.
- Profile conformance: every mandatory requirement and test for a named client
  profile passes.
- Interoperability: behavior has been tested with a named external server.
- Certification: an external formal authority has certified the product.

The project may claim the first four only with linked evidence. It never uses
the term certified without an applicable formal result.

## Requirement records

The checked-in [`KMIP 2.1 inventory`](../../specification/catalog/README.md)
is the canonical source map for later implementation specifications. Its
generated [coverage report](../../specification/catalog/coverage-report.md)
summarizes the reviewed source ledger and leaves unassigned implementation and
verification links visible. Each normative requirement record contains:

- Stable requirement ID.
- Source document, stage, date, and section.
- Exact subject and a short non-copyrighted paraphrase.
- Normative level: MUST, SHALL, MUST NOT, SHALL NOT, SHOULD, MAY, or OPTIONAL.
- Client/server/direction applicability.
- Scope state: `client_1_0`, `client_1_1`, `server_only`,
  `profile_conditional`, `out_of_scope`, `mixed`, or `unclear`.
- Implementation location.
- Positive and negative test IDs.
- Profiles using the requirement.
- Status and review evidence.
- Notes, errata, and approved interpretation decisions.

## Normative level policy

- Applicable MUST and SHALL requirements are release blockers.
- MUST NOT and SHALL NOT requirements require negative tests.
- SHOULD requirements are implemented unless an accepted decision records the
  deviation and consequence.
- Every client MAY and OPTIONAL feature in scope is representable and usable,
  even when disabled by default.
- Server-only requirements are marked not applicable to the 1.0 client.
- Server-initiated operations are marked for 1.1, not misreported as complete
  in 1.0.

## Profile matrix

For each KMIP 2.1 client profile, record:

- Applicability to TTLV and the supported transports.
- Required operations, objects, attributes, authentication, and tests.
- Clause and test status.
- Deviations and server prerequisites.
- Date, KMIPKit version, and evidence for any public claim.

Inventory records keep profile applicability, selected target, runtime
validation availability, catalog evidence state, fixture availability, and
public claims separate. Baseline Client is the selected 1.0 evidence target;
its 294-requirement closure has typed mappings and category-appropriate
negative behavior tests, so explicit runtime selection is available. Its catalog
`claim_state` remains `evidence_incomplete`: the 16 linked mandatory official
XML fixtures are pinned, while T036 records 10 full Baseline Client (`BL-M`)
replays, 3 full Shared Knowledge Link Client (`SKLC-M`) replays, and 3 typed
preflight blockers. KMIP Profiles v2.1 §6.1 item 4 requires support for one or
more cases from §§5.1.3 and 5.6.3, not every linked case. The three blocked
cases remain visible as case-level evidence gaps but do not by themselves
invalidate that minimum. Seventy-seven other linked profile cases have no
local fixture. Local runtime validation and fixture replay do not establish
server interoperability or profile conformance. No profile conformance claim
is published by KMIPKIT-0010.
Other profile records retain their own catalog states; none of these fields is
itself a public claim.

The generated inventory report includes each profile's source clauses,
requirements, elements, dependencies, linked official test IDs and status,
transport, encoding, runtime rule coverage, and blockers so reviewers can
inspect that evidence without cross-referencing the JSON catalog manually.

## Profile readiness workflow

Use the following stages independently:

1. **Applicability and target**: check the profile's scope and the fixed
   `profile-targets.json` manifest. Baseline target membership records the
   project plan; it neither enables runtime selection by itself nor asserts
   conformance.
2. **Runtime validation**: require an explicit typed rule mapping and
   category-appropriate executable verification for every applicable
   requirement. Missing, incomplete, or untested mappings keep profile
   selection unavailable before network I/O. Catalog prose is not executable
   behavior.
3. **Evidence readiness**: inspect the deterministic
   [coverage report](../../specification/catalog/coverage-report.md) for full
   clause and requirement closure, implementation and verification links,
   dependencies, deviations, mandatory tests, and fixture status. An
   unavailable fixture is an evidence gap, not a failed or passing result.
4. **Current verification**: only a same-run aggregate job with
   `event_name=push`, `ref=refs/heads/release/1.0.0`, and
   `ref_protected=true` can contribute verified CI evidence. It must bind the
   checked-out commit, workflow, run and attempt to successful jobs with the
   exact IDs `core`, `script-contracts`, `normative-inventory`, `coverage`,
   `coverage-gate`, and `dependency-policy`. The job uses a short-lived
   GitHub App token with Administration read permission to check current
   branch protection; required reviews, stale-review dismissal, admin
   enforcement, disabled force-push/deletion, and a present bypass-allowance
   object with empty user, team, and app lists are required. Pull-request,
   scheduled, local, unprotected, or mismatched runs are ineligible. The
   ephemeral evidence manifest does not update the catalog or generated
   report.
5. **Release claim**: a passing readiness result does not publish a claim.
   A catalog state advances only after its required human review, and any
   public claim remains a separate reviewed release decision.

Baseline remains ineligible for a formal profile claim until all applicable
requirements and evidence gates are satisfied, including current protected-
branch verification. The three case-level replay blockers remain recorded but
do not by themselves negate the one-or-more test requirement. Runtime
selection being available means only that selected typed rules can be
evaluated before and after an exchange; it does not establish server
interoperability or close the external evidence gates.
The registry also includes a mapping for Wrapped Secret Data under
KMIPKIT-REQ-SPEC-2.7-001. See the
[KMIPKIT-0010 specification](../../specs/010-profile-conformance/spec.md) for
its exact scope and acceptance criteria.

The same report lists each tag range separately, groups named tags by their
OASIS allocation, and shows wire values and allocation states for unassigned
protocol elements. OASIS allocation states remain distinct from KMIPKit's
implementation and verification assignments.

## Source discrepancies and evidence gaps

The inventory records unresolved source conflicts without choosing an
interpretation. It links each discrepancy to affected requirements, elements,
profiles, or policies when the source supports that relationship. An
implementation specification that touches an affected record must remain
gated until the discrepancy is resolved by reviewed evidence. Missing official
fixtures and malformed source labels stay visible as evidence limitations;
they do not change the source text or establish a protocol interpretation.

## Test evidence

Official vectors are pinned and immutable. Project tests reference their source
and add focused cases for uncovered clauses and malformed input. A single large
test cannot replace individually traceable requirements.

## Interoperability matrix

Record server vendor/project, product, exact version, configuration, endpoint
transport, TLS behavior, claimed profiles, tested operations, extensions,
result, date, and any explicit compatibility option. Passing a server test is
evidence of interoperability, not proof that the server or client implements
the entire standard.

See the [KMIP server interoperability matrix](../development/kmip-interoperability.md)
for the current research inventory and the pending integration-test ledger.

## Maintenance

Every protocol PR updates the matrix. CI validates references and rejects
implemented requirements without tests or test IDs without an applicable
requirement. Releases publish a generated human-readable conformance report.
