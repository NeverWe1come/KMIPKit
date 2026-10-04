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

Inventory records keep profile applicability, selected target, fixture
availability, evidence completion, and public claim state separate. A profile
record is not a conformance claim; current inventory claims remain
`not_claimed` until all applicable clauses and official tests have evidence.
The generated inventory report includes each profile's source clauses,
requirements, elements, dependencies, linked official test IDs and status,
transport, and encoding so reviewers can inspect that evidence without
cross-referencing the JSON catalog manually.

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

## Maintenance

Every protocol PR updates the matrix. CI validates references and rejects
implemented requirements without tests or test IDs without an applicable
requirement. Releases publish a generated human-readable conformance report.
