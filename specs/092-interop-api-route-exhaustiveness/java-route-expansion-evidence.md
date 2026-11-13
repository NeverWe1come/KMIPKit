# Java local route expansion evidence

This implements local obligations of KMIPKIT-0092-FR-002, FR-003, FR-006,
and FR-008 against the installed artifact pinned at
`9a16cc90a12205101d22a38501a5e74c665df4dc`. The complete matrix remains
mandatory; unimplemented cases remain pending, never API-unavailable.

## Tag identity and allocation

Sources: OASIS KMIP 2.0 Specification §11.54 and KMIP 2.1 Specification
§11.56 (Tag enumeration), plus §10.1 (TTLV representation).

Red commit `3e8d2b469` adds per-case, per-version route assertions and fails
because the expected installed RawTag calls were absent. Integration with
the shared gate established that each selected case must complete every
applicable route. This phase therefore selects the 264 Tag cases whose
approved Java surface is entirely generic, including all 20 reserved tags.
Another 110 tags have additional typed obligations. The existing Message
Extension fixture already completes one of those cases; the other 109 remain
pending. Their absence does not change the manifest or full-matrix gate.

Each assigned tag is constructed with `RawTag.fromRaw`, allocation-checked,
used in `TtlvItem.create`, read through an item view, and decoded from a
deterministic TTLV fixture. The assertions compare the exact tag, type, and
sentinel value. Reserved tags retain their raw identity and exercise both
the actual checked-tag rejection and the decoder's `RESERVED_TAG` category.
These are tag-identity checks, not semantic payload or server-exchange claims.

Verification:

```sh
bash ../../bindings/java/gradlew --max-workers=2 test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 37d7eeae9e64644fd7cdde4bfd64b7da5ccbb0bf2be53a949826eb62dc3aaf86 \
  --write-ledger
```

Green commit `fd78236aa` passes the focused Java test. The shared validator accepts 10,958 executed
cells, zero failed cells, and 60,497 pending out of 71,455 planned cells.
This adds 2,560 validated cells to the 8,398-cell baseline.

The local host's `/tmp` was full during development. Tests use
`JAVA_TOOL_OPTIONS=-Djava.io.tmpdir=<worktree>/interop/java-kmip-validator/build/java-tmp`
to retain private native extraction on the worktree filesystem. This changes
no package, test assertion, matrix obligation, or CI gate.

## Referenced PKCS#11 values

Sources: OASIS KMIP 2.0/2.1 Specification §§11.38-11.39 and the catalog's
SHA-256-pinned OASIS PKCS #11 3.0 headers (`pkcs11f.h` and `pkcs11t.h`).

Red commit `d8c8d747d` requires each referenced value through all four
applicable routes and fails because those route events were absent.

The fixture invokes the installed Enumeration constructor, Item constructor,
Item view and decoder for all 92 Function values and 97 Return Code values.
It checks each exact unsigned value under tag `0x42015a` or `0x42015d`,
including `CKR_VENDOR_DEFINED`'s high bit. The four routes must produce the
same normalized tree hash for each case and protocol version. This does not
invoke PKCS#11 functions or claim support for a vendor extension.

Green commit `de4ad991a` passes the same focused Java test and shared validation commands with 12,470
executed cells, zero failures and 58,985 pending out of 71,455 planned cells.
PKCS#11 adds 1,512 validated cells; the two blocks together add 4,072 cells
to the 8,398-cell baseline. The selected cases are complete; the full Java
matrix remains incomplete.

## Refactor and final verification

The two local fixture categories share one scalar readback assertion. The
assertion checks the exact tag, TTLV type, and scalar value for both created
and decoded Items and releases each borrowed handle. Event publication stays
separate from invocation, and only sanitized hashes enter the report.

The full installed-consumer suite and regenerated ledger are verified after
the refactor. README and FR-003 traceability report the same planned,
executed and pending cell counts as the generated ledger; no common manifest
or matrix limitation is changed by this phase.

Final checks after the refactor:

- Full Java consumer suite: 51 tests passed, no failures or skips.
- Shared interoperability suite: 178 tests passed.
- Java installer/ledger tooling: 12 tests passed.
- Requirement and feature traceability: 33 tests passed.
- Shared route evidence validator: 12,470 executed cells, zero failed cells,
  58,985 pending cells, and `selected_scope_complete=true`.
- `python3 tools/compare_route_ledger.py`: generated coverage matches the
  checked-in reference.
- `git diff --check`: passed.

Python tooling tests use an external, disk-backed task temporary directory
when `/tmp` is full so that their outside-project path validation fixtures
retain their intended meaning.

## Version-specific Discover Versions route deferral

Sources: OASIS KMIP 2.0 and 2.1 Specifications §6.1.16. This behavior is
scoped to the client-initiated `test-op-discover-versions` exchange; the
server-initiated case remains unavailable for its existing workflow reason,
and local KMIP 2.0 request/response model checks remain executable.

The installed artifact pinned by the shared route manifest is
`9a16cc90a12205101d22a38501a5e74c665df4dc`. Its typed Discover Versions
response conversion validates the response against the default offer, which
contains KMIP 2.1 only. The public request route exists, but the KMIP 2.0
response cannot currently complete through that typed API. This is a
KMIPKit correctness limitation, not evidence about any server's compatibility.

Red commit `579523301` adds an isolated KMIP 2.0 regression case. Before the
route fix, the consumer attempted the unavailable endpoint and classified the
transport failure as a failed exchange. The test requires `not_run`, unknown
server compatibility, and a sanitized diagnostic identifying the default
KMIP 2.1 offer. Green commit `22f915742` makes route classification depend on
the selected protocol version and uses that classification consistently in
`plan` and `run`. KMIP 2.0 Discover Versions is reported as pending without
opening a connection; KMIP 2.1 remains executable.

The follow-up refinement adds inventory assertions for the KMIP 2.0 pending
classification and the still-executable KMIP 2.1 and version-neutral routes.
It also synchronizes FR-003/FR-009 traceability with the current 13,414
executed and 58,041 pending ledger. It makes no new public API or server
compatibility claim.

Verification:

```sh
bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.MainContractTest.kmip20DiscoverVersionsIsPendingInsteadOfClaimingAClientFailure \
  --tests org.kmipkit.validator.JavaRouteInventoryTest
```

The focused suite passes. The complete KMIP 2.0/2.1 route matrix is still
incomplete; this scoped deferral does not satisfy the pending Discover
Versions route cell or alter the full-matrix gate.

## Block Cipher Mode enumeration route completion

Source: OASIS KMIP 2.0 and 2.1 Specifications §11.6 (Block Cipher Mode
Enumeration) and §4.16 (Cryptographic Parameters). This selected local scope
executes all 19 catalogued assigned and extension values in both protocol
versions through five public routes: `TtlvValue.Enumeration`,
`TtlvItem.create`, `TtlvItem.view`, `TtlvCodec.decode`, and
`CryptographicParameters.Builder.blockCipherMode`. Each case/value/version
cell is checked against the installed package pinned to
`9a16cc90a12205101d22a38501a5e74c665df4dc`; normalized tree hashes are used
for the generic TTLV paths.

Red commit `3ad9b2f21` added an assertion for the catalogued AEAD value under
KMIP 2.0 and 2.1. The focused controlled-peer route test failed because the
`test-enum-block-cipher-mode` generic Enumeration constructor event was absent
for that value and version. Green commit `c9d54c4f2` derives fixtures from the
catalog, invokes every assigned value and the representative extension value,
checks Item construction, view and decode against the exact Block Cipher Mode
tag, and invokes the typed Cryptographic Parameters builder for every value.
The typed call must retain the builder handle. The report records only
sanitized tree hashes and API invocation metadata.

Verification after the implementation:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 37d7eeae9e64644fd7cdde4bfd64b7da5ccbb0bf2be53a949826eb62dc3aaf86 \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The full Java consumer suite passes. The shared validator reports a complete
selected scope of 13,604 invoked cells with zero failures, including all 190
Block Cipher Mode route cells. The refreshed Java ledger records 13,604
executed, 57,851 pending, and 71,455 planned cells. This completes only the
Block Cipher Mode local route family; other enum families and the overall
KMIP 2.0/2.1 matrix remain pending.

The installed artifact was rebuilt from the exact release pin. Its SHA-256 is
`37d7eeae9e64644fd7cdde4bfd64b7da5ccbb0bf2be53a949826eb62dc3aaf86`; the
public route probe verified 243 API symbols, 677 named values, and 189 numeric
values against the installed JAR. The offline report also conforms to the
shared result schema. The artifact and controlled-peer checks do not establish
compatibility with an external KMS.
