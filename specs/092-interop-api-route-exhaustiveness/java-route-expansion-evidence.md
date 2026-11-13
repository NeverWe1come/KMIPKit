# Java local route expansion evidence

This implements local obligations of KMIPKIT-0092-FR-002, FR-003, FR-006,
and FR-008. The current installed artifact pin is
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`; earlier route sections retain the
pin and digest used for their historical runs. The complete matrix remains
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

## PKCS#11 aggregate catalog cases

Sources: OASIS KMIP 2.0/2.1 Specification §§11.38-11.39 and the pinned
PKCS #11 3.0 headers listed above. The catalog has two additional aggregate
cases, one each for Function and Return Code. The per-value rows were already
executed, but the aggregate rows remained pending because those invocations
were emitted only with per-value case IDs.

Red commit `fac9a643c` adds assertions for all 92 Function and 97 Return Code
targets under the aggregate case IDs. The focused test fails because the
expected aggregate route events are absent. Green commit `220a74a10` emits
the actual installed Enumeration constructor, Item constructor, Item view,
and decoder invocations under both the per-value and corresponding aggregate
case identities. Each event retains the exact source-backed target ID,
protocol version, unsigned value, tag, and normalized hash; no request body or
secret material is included.

Verification:

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

The installed Java consumer suite passes 52 tests with no failures, errors,
or skips. The shared validator accepts 16,322 invoked cells, zero failures,
and `selected_scope_complete=true`; the refreshed ledger records 16,322
executed, 55,133 pending, and 71,455 planned cells. This closes the two
aggregate case rows only; the complete Java matrix remains pending.

## Refactor and final verification

The two local fixture categories share one scalar readback assertion. The
assertion checks the exact tag, TTLV type, and scalar value for both created
and decoded Items and releases each borrowed handle. Event publication stays
separate from invocation, and only sanitized hashes enter the report.

The PKCS#11 fixture selects its aggregate case identity from the source-backed
parent name while reusing each value's already-asserted installed API
invocation. README and FR-003 traceability report the same planned, executed
and pending cell counts as the generated ledger; no common manifest or matrix
limitation is changed by this phase.

Final checks after the refactor:

- Full Java consumer suite: 52 tests passed, no failures, errors, or skips.
- Shared route evidence validator: 18,652 executed cells, zero failed cells,
  52,803 pending cells, and `selected_scope_complete=true`.
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

## Rotate Name Type enumeration route completion

Source: OASIS KMIP 2.1 Specification §11.51 (Rotate Name Type Enumeration)
and §10.1 (TTLV representation). The selected local case covers its two
assigned values and the extension-range representative through four public
routes: `TtlvValue.Enumeration`, `TtlvItem.create`, `TtlvItem.view`, and
`TtlvCodec.decode`. All 12 value/route cells run under KMIP 2.1 against the
installed artifact pinned to `9a16cc90a12205101d22a38501a5e74c665df4dc`.

Red commit `237ac7d07` asserted all three catalogued targets on each route.
The focused controlled-peer route test failed because no
`test-enum-rotate-name-type` route events were emitted. Green commit
`87908fac0` adds this case to the catalog-driven local enumeration fixture;
the same test then passed, including all 12 target/route assertions.

Verification:

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

The selected scope contains 13,616 invocations with zero failures, and the
updated ledger records 13,616 executed, 57,839 pending, and 71,455 planned
cells. Rotate Name Type is complete for this local route scope; the remaining
enum values and the full KMIP 2.0/2.1 matrix are still pending.

## Additional local enumeration route groups

These catalog-driven local scopes close the remaining routes for four small
enumeration families using the same installed artifact pin. Each family runs
only in versions declared by its catalog case:

- Opaque Data Type: one extension-range representative across TTLV
  Enumeration construction, Item creation/view, decoding, TypedAttribute
  creation/readback, and `AttributeSet.tryNew` for KMIP 2.0/2.1. Sources:
  KMIP 2.0/2.1 §§10.1, 11.35.
- Processing Stage: three assigned values and one extension-range
  representative across the four generic TTLV routes for KMIP 2.1. Sources:
  KMIP 2.1 §§10.1, 11.40.
- Ticket Type: the Login value and one extension-range representative across
  the four generic TTLV routes for KMIP 2.0/2.1. Sources: KMIP 2.0 §11.55;
  KMIP 2.1 §§10.1, 11.57.
- Mask Generator: MFG1 and one extension-range representative across four
  generic TTLV routes and `CryptographicParameters.Builder.maskGenerator` for
  KMIP 2.0/2.1. Sources: KMIP 2.0/2.1 §§4.16, 10.1, 11.30.

Red commit `5017877f6` asserted each target and route cell and failed because
the Opaque Data Type scope had no route events. Green commit `f48ed064a`
adds all four cases to the catalog-driven fixture; the focused controlled-peer
test passes and the shared validator accepts all 66 new cells without a
failed invocation.

Verification for these scopes:

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

The generated selected scope records 13,682 invocations, zero failures, and
`selected_scope_complete=true`; the ledger records 13,682 executed, 57,773
pending, and 71,455 planned cells. All four cases are complete in their local
route scopes. Other enum families and the overall KMIP 2.0/2.1 matrix remain
pending.

## Common enum TTLV route group

Nine catalogued enum cases now complete the same four public generic routes:
`TtlvValue.Enumeration`, `TtlvItem.create`, `TtlvItem.view`, and
`TtlvCodec.decode`. Each case contains two assigned values and one extension
range representative and runs for KMIP 2.0 and 2.1, yielding 216 cells total:

| Enumeration | KMIP 2.0 section | KMIP 2.1 section |
|---|---:|---:|
| Encoding Option | §11.18 | §11.18 |
| Endpoint Role | §11.19 | §11.19 |
| Key Value Location Type | §11.27 | §11.27 |
| Key Wrap Type | §11.29 | §11.29 |
| Name Type | §11.31 | §11.31 |
| Object Group Member | §11.33 | §11.33 |
| Put Function | §11.42 | §11.43 |
| Secret Data Type | §11.50 | §11.52 |
| Usage Limits Unit | §11.58 | §11.60 |

Each generic TTLV fixture uses §10.1's Enumeration item encoding. Red commit
`3615b77e5` asserted every catalogued target/version/route and failed because
the Encoding Option events were absent. Green commit `21b3dae2c` adds all nine
cases to the catalog-driven local fixture; the focused test passes for every
target and route.

Verification:

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

The Java 17 consumer suite passes 52 tests with zero failures or skips. The
installer/ledger suite passes 15 tests, feature traceability passes 31 tests,
and `tools/compare_route_ledger.py` confirms the generated ledger matches the
checked-in reference. The shared validator confirms 13,898 invoked cells,
zero failures, and `selected_scope_complete=true`; the refreshed ledger records
13,898 executed, 57,557 pending, and 71,455 planned. These nine enum cases are
complete for the four generic routes; the rest of the Java operation,
attribute, and protocol matrix remains pending.

## Cryptographic Algorithm enumeration route completion

Source: OASIS KMIP 2.0 and 2.1 Specifications §§4.16 (Cryptographic
Parameters), §10.1 (TTLV), and §11.12 (Cryptographic Algorithm Enumeration).
The local aggregate case executes all 57 catalogued assigned and extension
values for KMIP 2.0 and 2.1 through eight applicable routes: generic
`TtlvValue.Enumeration`, `TtlvItem.create`, `TtlvItem.view`, and
`TtlvCodec.decode`; typed `TypedAttribute.create`, `TypedAttribute.fromItem`,
and `AttributeSet.tryNew`; and
`CryptographicParameters.Builder.cryptographicAlgorithm`. This completes
912 value/route/version cells against the artifact pinned to
`9a16cc90a12205101d22a38501a5e74c665df4dc`.

Red commit `fbf80808c` required all 57 distinct normative target IDs on each
route and failed with zero `test-enum-cryptographic-algorithm` events. Green
commit `55c4e195d` includes the aggregate case in the catalog-driven local
fixture. The focused controlled-peer test passes for every target, route, and
version.

Verification:

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

The shared validator accepts 14,810 invoked cells with zero failures and
`selected_scope_complete=true`; the current ledger records 14,810 executed,
56,645 pending, and 71,455 planned cells. The Cryptographic Algorithm enum
route family is complete for this local scope. Other KMIP operations, values,
attributes, and protocol routes remain pending.

## Recommended Curve enumeration route completion

Sources: OASIS KMIP 2.0 §11.44, KMIP 2.1 §11.45, and §10.1 (TTLV
Enumeration representation). The aggregate local case contains 71 catalogued
values, including its catalogued extension value, and runs under both protocol
versions through `TtlvValue.Enumeration`, `TtlvItem.create`, `TtlvItem.view`,
and `TtlvCodec.decode`. This is 568 value/route/version cells on the installed
artifact pinned to `9a16cc90a12205101d22a38501a5e74c665df4dc`.

Red commit `4b7ea6735` requires all 71 target IDs and the four generic routes;
the focused test fails because the aggregate has zero local route fixtures.
Green commit `020b34f18` adds the case to the catalog-driven local enumeration
scope. The focused test passes for every target, route, and version. Refactor
commit `17d13256c` extracts shared enum target selection for the Recommended
Curve and Cryptographic Algorithm assertions without changing the matrix.

Verification:

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

The full installed Java consumer suite passes 52 tests with no failures,
errors, or skips. The shared validator accepts 16,890 cells with zero
failures and `selected_scope_complete=true`; the generated ledger records
16,890 executed, 54,565 pending, and 71,455 planned cells. This completes
Recommended Curve's local TTLV route scope; all other pending matrix work
remains open.

## Result Reason enumeration route completion

Sources: OASIS KMIP 2.0 §11.45, KMIP 2.1 §11.46, and §10.1 (TTLV
Enumeration representation). The aggregate local case covers all 78
catalogued Result Reason values through `TtlvValue.Enumeration`,
`TtlvItem.create`, `TtlvItem.view`, and `TtlvCodec.decode` for both protocol
versions. This adds 624 value/route/version cells. Three values are assigned
by KMIP 2.1 only; the test also constructs, encodes, decodes, and preserves
them as unknown Enumeration values under KMIP 2.0, as required by the
forward-compatible generic TTLV API.

Red commit `7fff99316` requires every catalogued target and fails with zero
local Result Reason fixtures. Green commit `3fa281f65` adds the aggregate case
to the catalog-driven local fixture and retains the three KMIP 2.1 values in
the KMIP 2.0 unknown-value round-trip. The selected local route assertions
and shared validator pass for every target, route, and version. An initial
green attempt exposed 12 missing cells: three KMIP 2.1-assigned values had
been filtered from the KMIP 2.0 unknown-value case. The fixture now tests
those three values in KMIP 2.0 as unknown enumerations and in KMIP 2.1 as
assigned values. Refactor commit `b2924ec6c` extracts this version rule into
a named helper so the forward-compatibility exception remains explicit.

Verification:

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

The full Java consumer suite passes 52 tests with no failures, errors, or
skips. The shared validator accepts 17,514 cells with zero failures and
`selected_scope_complete=true`; the generated ledger records 17,514 executed,
53,941 pending, and 71,455 planned. This completes Result Reason's local
generic TTLV scope; the overall Java matrix remains incomplete.

## Operation enumeration route completion

Sources: OASIS KMIP 2.0 §11.36, KMIP 2.1 §11.36, and §10.1 (TTLV
Enumeration representation). The aggregate local case covers all 60
catalogued Operation values through `TtlvValue.Enumeration`,
`TtlvItem.create`, `TtlvItem.view`, and `TtlvCodec.decode` for both protocol
versions, adding 480 value/route/version cells. Six Operations are assigned
in KMIP 2.1 only; the fixture also tests them as unknown Enumeration values
under KMIP 2.0 to verify forward-compatible preservation.

Red commit `c57db322f` requires all 60 catalogued targets and fails because
the Operation aggregate has no local fixtures. Green commit `39527b5aa`
adds the aggregate to the catalog-driven scope and round-trips the six
KMIP 2.1-only values as unknown in KMIP 2.0. Refactor commit `fe8490d7d`
declares the version-specific enum families together so their preservation
rule is explicit and shared with Result Reason.

Verification:

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

The full Java consumer suite passes 52 tests with no failures, errors, or
skips. The shared validator accepts 17,994 cells with zero failures and
`selected_scope_complete=true`; the generated ledger records 17,994 executed,
53,461 pending, and 71,455 planned. This completes Operation's local generic
TTLV scope; the overall Java matrix remains incomplete.

## Digital Signature Algorithm enumeration route completion

Sources: OASIS KMIP 2.0 and 2.1 §11.16, §4.16 (Cryptographic Parameters),
and §10.1 (TTLV Enumeration representation). The local aggregate case covers
all 20 catalogued values under both protocol versions through eight applicable
routes: four generic TTLV routes, `TypedAttribute.create`,
`TypedAttribute.fromItem`, `AttributeSet.tryNew`, and
`CryptographicParameters.Builder.digitalSignatureAlgorithm`. This adds 320
value/route/version cells.

Red commit `4ee2f0438` asserts all 20 values and fails because the aggregate
case has no local fixtures. Green commit `c3ebeffff` adds the case to the
catalog-driven local enumeration scope and invokes each installed generic,
typed attribute, and Cryptographic Parameters builder route. Refactor commit
`921b68352` obtains the builder route from the shared route map used by the
other Cryptographic Parameters enum recipes.

Verification:

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

The full Java consumer suite passes 52 tests with no failures, errors, or
skips. The shared validator accepts 18,314 cells with zero failures and
`selected_scope_complete=true`; the generated ledger records 18,314 executed,
53,141 pending, and 71,455 planned. This completes Digital Signature
Algorithm's local route scope; the full Java matrix remains incomplete.

## Key Format Type enumeration route completion

Sources: OASIS KMIP 2.0/2.1 §11.25 (Key Format Type Enumeration), §8.6
(KMIP 2.0 Batch Item response), and §9.7 (KMIP 2.1 Batch Item response).
The aggregate local case covers all 24 catalogued Key Format Type values
through the four generic TTLV routes, `TypedAttribute.create`,
`TypedAttribute.fromItem`, and `AttributeSet.tryNew` for both versions. The
controlled Get fixture also observes the `Raw` value in its actual response
and reads the response Batch Item through `ClientBatchItemResult.raw` under
both versions. This completes 338 cells: 336 local value/route/version cells
and two controlled-response accessor cells.

Red commit `510b0b804` requires all 24 values and fails because the aggregate
case has no local fixtures. Green commit `9d1468b63` enables the catalog-driven
typed/generic routes and emits the aggregate raw-response event only after
the controlled response contains the exact `Key Format Type=Raw` TTLV item.
Refactor commit `03356ee68` counts the additional response route evidence as
one invocation per protocol version.

Verification:

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

The full Java consumer suite passes 52 tests with no failures, errors, or
skips. The shared validator accepts 18,652 cells with zero failures and
`selected_scope_complete=true`; the generated ledger records 18,652 executed,
52,803 pending, and 71,455 planned. This completes Key Format Type's local
generic, typed, and response accessor scope; the full Java matrix remains
incomplete.

## Release pin and shared contract refresh

The Java branch merges shared contract commit `4f157e50e`, which includes the
release update to `489fafc79ef89db2524eafa7a6c6482f5cf75d97`. Its private
artifact pin, installer manifest, and checked-in route ledger now identify
that exact library commit. The installed Linux JNI JAR SHA-256 is
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`.
The shared comparison intentionally ignores the artifact digest when comparing
coverage baselines; the generated build ledger and provenance manifest retain
the current build's digest.

Red commit `b40856e5c` updates the pin and reproduces three Java failures caused
by test fixtures still requiring the previous SHA. Green commit `0575bdb50`
aligns those checks with the new pin. Refactor commit `e1cfd6914` reads the pin
from `library-pin.json` in one test helper so future pin updates do not require
duplicated literals.

Verification against this exact artifact:

```sh
TMPDIR=/home/ramp1953/.cache/kmipkit-interop-tmp \
  python3 tools/install_artifact.py --repository ../.. \
  --pin-file library-pin.json --maven-repository build/maven-repository \
  --manifest build/artifact-manifest.json
python3 ../../interop/shared/tools/probe_route_artifact.py --language java \
  --artifact build/maven-repository/org/kmipkit/kmipkit-java/1.0.0/kmipkit-java-1.0.0-linux-glibc-x86_64.jar \
  --provenance build/artifact-manifest.json
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

Installation and the public-route artifact probe pass for commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`. The full Java consumer suite
passes 52 tests with zero failures, errors, or skips. The shared validator
accepts 18,652 invoked cells with zero failures and
`selected_scope_complete=true`; 52,803 of the 71,455 planned cells remain
pending. This refresh does not establish interoperability with a deployed KMS
or complete the Java matrix.
