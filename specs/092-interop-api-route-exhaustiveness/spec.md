# Specification: Exhaustive KMIP interoperability API-route coverage

**Feature Branch**: `feature/KMIPKIT-0092-interop-api-route-exhaustiveness`  
**Status**: Approved for implementation by the maintainer's explicit task instruction  
**Identifier**: KMIPKIT-0092  
**Base**: `release/1.0.0` at `9a16cc90a12205101d22a38501a5e74c665df4dc`

## Purpose and scope

Extend the shared interoperability contract so its finite test catalog first
accounts for all normative KMIP 2.0/2.1 constructs and branches, then requires
the Rust, C, Java, and Python validators to test every applicable case through
every distinct public KMIPKit request-construction route that can represent
it. The required construction routes are high-level API, typed protocol API,
and generic TTLV construction/decoding wherever the installed public package
exposes that route. The validators must also cover every encoding and
transport combination supported by the relevant public client API. A route
label is evidence only when the test actually calls that route.

Coverage includes every KMIP 2.0 and 2.1 operation, managed object type,
attribute tag and value type, every assigned enumeration value and bit-mask
flag, every TTLV data type, version boundary, normative request/response
field, optional/conditional branch, negative result, batch order, asynchronous
behavior, and specified edge case. The current catalog is not presumed
complete: its inventory must be checked against the pinned OASIS sources and
expanded where it omits normative values or branches. For unbounded strings,
byte arrays, identifiers, and numeric domains, use every normative boundary
and constraint class plus property-based round trips instead of attempting an
infinite enumeration. The catalog and public API manifest together define the
finite test surface; the same normative case may require different request
recipes or be inapplicable to a particular public route. Local TTLV
construction and decoding is tested even when the package has no public
generic-TTLV send operation.

This contract covers independent consumers of installed packages built from
one recorded exact release commit. It does not modify library implementation
code or add vendor extensions. Tests use generated requests/responses and
controlled local KMIP peers until vendor KMS endpoints are available; those
results must never be described as vendor interoperability evidence.

## Normative and project references

- OASIS KMIP Specification 2.0 and 2.1, with exact section references retained
  on every applicable catalog case.
- Approved specifications KMIPKIT-0083, KMIPKIT-0084, KMIPKIT-0085,
  KMIPKIT-0086, KMIPKIT-0087, and KMIPKIT-0088.
- `specification/api/public-api.json`, generated public API declarations, and
  the installed package artifact for route availability.
- `interop/shared/catalog/kmipkit-1.0-test-catalog.json` and
  `interop/shared/schemas/validation-results.schema.json`.
- `AGENTS.md` §§5, 6, 8–10 for traceability, secure results, API parity, and
  generated-artifact rules.

## Requirements

| ID | Requirement |
|---|---|
| KMIPKIT-0092-FR-001 | A generated coverage matrix SHALL map each catalog case and protocol version to every distinct public construction route that can represent it in each language, and every encoding/transport combination supported by that route. Route capability SHALL be derived from public API metadata and installed-artifact checks, not from validator labels. |
| KMIPKIT-0092-FR-002 | CI SHALL fail when any applicable case × public route × supported encoding × supported transport combination lacks an executable request/response check or a source-backed reason for non-applicability. A public constructor SHALL NOT be classified as unavailable merely because a validator recipe is missing. |
| KMIPKIT-0092-FR-003 | Each operation, object, attribute, enumeration, bit mask, version boundary, negative case, batch/async workflow, and catalog edge case SHALL be exercised through every applicable construction route. Local generic TTLV cases SHALL construct, encode, decode, and compare values for each applicable public API route. |
| KMIPKIT-0092-FR-004 | For equivalent deterministic inputs, validators SHALL compare normalized KMIP request and response trees across construction routes and languages. They SHALL preserve ordered batch items and numeric Result Status/Reason codes. Exact bytes SHALL be compared only when inputs and encoding are identical and deterministic. |
| KMIPKIT-0092-FR-005 | Controlled peers SHALL parse requests and return checked, case-specific responses for every supported route/encoding/transport matrix cell. Tests SHALL verify response shape and observable state, including batch outcomes, persisted attributes, retrieval, cryptographic results, and teardown where applicable. |
| KMIPKIT-0092-FR-006 | Every validator SHALL consume an installed package built from the same recorded exact release commit for a comparable cross-language matrix; reports SHALL retain the artifact digest and all execution axes. |
| KMIPKIT-0092-FR-007 | Shared schemas and aggregation SHALL retain construction route, encoding, transport, protocol version, case identity, and artifact provenance, and SHALL report missing coverage separately from library failures and observed server compatibility. |
| KMIPKIT-0092-FR-008 | Results and diagnostics SHALL exclude credentials, private keys, secret key material, raw KMIP bodies, remote error text, and unredacted cryptographic values. Controlled fixture results SHALL NOT imply vendor support or certification. |
| KMIPKIT-0092-FR-009 | Public API limitations SHALL be explicit and source-backed. A limitation is valid only when the exact installed public artifact lacks the required constructor, encoder, decoder, or send route; an unfinished validator recipe is pending coverage and SHALL fail the completeness gate. |
| KMIPKIT-0092-FR-010 | The normative inventory SHALL enumerate every in-scope operation, managed object, attribute, TTLV data type, assigned and reserved enumeration/bit-mask value, extension allocation, tag and tag-allocation range, request/response field branch, and defined boundary/error class from the pinned OASIS KMIP 2.0/2.1 sources. Values incorporated by normative reference (including PKCS#11 values in KMIP §§11.38–11.39) SHALL have their referenced source version pinned and inventoried. The generator and tests SHALL fail when a source-backed item is absent from the catalog. |

## Acceptance criteria

- The shared generator emits a deterministic matrix from the normative case
  catalog and checked-in public API capability metadata. Source-audit tests
  prove the catalog covers the full normative inventories, including each
  assigned enum value and bit flag rather than just each enum/mask type, as
  well as reserved values, extension allocations, tag ranges, and externally
  referenced assigned values.
- Each language's CI executes the matrix against its installed package and
  rejects both missing rows and duplicate rows with conflicting outcomes.
- Every positive operation case has a recipe-specific request assertion, a
  response assertion, and required setup/cleanup. A result for one variant
  cannot satisfy another case solely because the operation name matches.
- Each catalog enum and mask case checks its own named type and values,
  including preservation of unknown values and bits.
- The parity workflow compares the same release artifact, case, protocol
  version, configuration, route, encoding, transport, and fixture across
  languages. Incomparable runs remain explicitly un-compared.
- The four language changes remain in separate branches/PRs. Every PR stays
  `[WIP]` until all required language and shared coverage checks pass, and no
  PR is merged by the agent.
- Vendor runs remain pending until an operator deploys a KMIP 2.0/2.1 KMS.

## Non-goals

KMIP 1.x, KMIP 3.0, manufacturer extensions, undocumented private interfaces,
and modifying KMIPKit library APIs are outside this contract. The validators
must exercise public APIs as shipped and report genuine API limits without
mislabeling incomplete project work as a library limitation.
