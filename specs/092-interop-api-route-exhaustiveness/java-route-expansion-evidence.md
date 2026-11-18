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

## NIST Key Type enumeration route completion

Sources: OASIS KMIP 2.0 and 2.1 §11.32 (NIST Key Type Enumeration) and
§4.34 (NIST Key Type Attribute); Tag 0x42013A is allocated in KMIP 2.0
§11.54 and KMIP 2.1 §11.56. The aggregate local case covers all 20
catalogued values through four generic TTLV routes and three typed attribute
routes in both versions, adding 280 cells. These local constructors verify
that every value remains readable through generic Enumeration, Item view,
decode, TypedAttribute creation/fromItem, and AttributeSet construction.

Red commit `d10d03473` requires 20 aggregate targets and fails because no
NIST Key Type fixtures were selected. Green commit `1ec3dbd15` adds the case to
the catalog-driven local enum fixtures and records all seven applicable
routes. Refactor commit `4e35ba1b0` shares the typed-attribute route list with
the other enum families that use the same installed API.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
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

The Java suite passes 52 tests with no failures, errors, or skips. The shared
validator accepts 18,932 invoked cells with zero failures and
`selected_scope_complete=true`; 52,523 of 71,455 planned cells remain pending.
NIST Key Type's local scope is complete; the Java matrix and vendor server
interoperability are not.

## Key Role Type enumeration route completion

Sources: OASIS KMIP 2.0 and 2.1 §11.26 (Key Role Type Enumeration) and §4.16
(Cryptographic Parameters). The aggregate local case covers all 25 catalogued
values through four generic TTLV routes and the
`CryptographicParameters.Builder.keyRoleType` route in both versions, adding
250 cells. Every assertion compares the preserved unsigned value across
Enumeration construction, Item creation/view, decode, and the builder setter.

Red commit `ba98894e1` requires all 25 aggregate targets and fails because the
case had no local fixtures. Green commit `6d874dc3f` adds all targets to the
catalog-driven fixtures and invokes the builder route for each value in both
versions. Refactor commit `e09fc63f0` centralizes the single-builder route
expectation shared by Key Role Type and Mask Generator.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
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

The Java suite passes 52 tests with no failures, errors, or skips. The shared
validator accepts 19,182 invoked cells with zero failures and
`selected_scope_complete=true`; 52,273 of 71,455 planned cells remain pending.
Key Role Type's local scope is complete; the Java matrix and vendor server
interoperability are not.

## Hashing Algorithm enumeration route completion

Sources: OASIS KMIP 2.0 and 2.1 §11.21 (Hashing Algorithm Enumeration) and
§4.16 (Cryptographic Parameters). The aggregate local case covers all 18
catalogued values through four generic TTLV routes and the
`CryptographicParameters.Builder.hashingAlgorithm` route in both versions,
adding 180 cells. Assertions check value preservation through generic
Enumeration construction, Item creation/view, decoding, and the builder setter.

Red commit `4fda41bbe` requires all 18 aggregate targets and fails because the
case had no local fixtures. Green commit `1c3793ade` adds every catalogued
value to the generic route fixtures and invokes the builder route for both
versions. Refactor commit `65d2ed657` groups builder-only enum route scopes so
their version and route expectations stay consistent.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
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

The Java suite passes 52 tests with no failures, errors, or skips. The shared
validator accepts 19,362 invoked cells with zero failures and
`selected_scope_complete=true`; 52,093 of 71,455 planned cells remain pending.
Hashing Algorithm's local scope is complete; the Java matrix and vendor server
interoperability are not.

## Link Type enumeration route completion

Source: OASIS KMIP 2.0 and 2.1 §11.28 (Link Type Enumeration). The aggregate
local case covers all 15 catalogued values through four generic TTLV routes in
both protocol versions, adding 120 cells. Each value is constructed as a
generic Enumeration, wrapped in an Item, read through its Item view, and
decoded from deterministic TTLV bytes.

Red commit `3166a664f` requires all 15 aggregate targets and fails because the
case has no local fixtures. Green commit `0275a6635` adds all catalogued
values to the local enum fixtures. Refactor commit `2dd170ef3` reuses a shared
generic enum-scope helper for Link Type and the other generic-only matrices.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
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

The Java suite passes 52 tests with no failures, errors, or skips. The shared
validator accepts 19,482 invoked cells with zero failures and
`selected_scope_complete=true`; 51,973 of 71,455 planned cells remain pending.
Link Type's local scope is complete; the Java matrix and vendor server
interoperability are not.

## Padding Method enumeration route completion

Sources: OASIS KMIP 2.0 and 2.1 §11.37 (Padding Method Enumeration) and §4.16
(Cryptographic Parameters). The aggregate local case covers all 11 catalogued
values through four generic TTLV routes and the
`CryptographicParameters.Builder.paddingMethod` route in both versions,
adding 110 cells. Each value is checked through generic Enumeration
construction, Item creation/view, TTLV decoding, and the builder setter.

Red commit `e268db773` requires all 11 aggregate targets and fails because the
case has no local fixtures. Green commit `d8d05e1b0` adds every catalogued
value to the local enum fixtures and invokes the builder route under both
protocol versions. Refactor commit `65521e330` names the shared KMIP 2.0/2.1
test-version set used by the local route matrix.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
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

The Java suite passes 52 tests with no failures, errors, or skips. The shared
validator accepts 19,592 invoked cells with zero failures and
`selected_scope_complete=true`; 51,863 of 71,455 planned cells remain pending.
Padding Method's local scope is complete; the Java matrix and vendor server
interoperability are not.

## Profile Name aggregate route completion

Source: OASIS KMIP 2.0 and 2.1 §11.41 (Profile Name Enumeration). The local
aggregate now checks all 43 catalogued targets (37 assigned, five reserved,
and one extension allocation) through generic Enumeration construction, Item
creation, Item view, and TTLV decoding in both protocol versions. The reserved
range target uses its catalogued lower boundary; the separate target-specific
reserved-range and Profile Information structure-member cases remain pending.

Red commit `0c0156ab` added an assertion for all 43 Profile Name targets; the
focused controlled-peer test failed as expected with zero aggregate fixtures.
Green commit `dd6994fd` added the Profile Name case to the catalog-driven local
enum scope. The installed-package test then passed and the shared validator
accepted 19,936 invoked cells, zero failed cells, and 51,519 pending cells out
of 71,455 planned cells.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` (artifact SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`):

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions \
  --rerun-tasks
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The complete Java route matrix and deployed-server interoperability remain
incomplete.

## Catalog-driven local enumeration aggregate coverage

The local controlled-peer suite now derives its eligible generic enum aggregate
case IDs from the pinned route catalog. It requires fixtures for every local
`test-enum-*` case whose enum value has a same-named standalone tag, instead of
maintaining a hand-curated list. The inventory assertion independently checks
that the fixture set matches the catalog-derived case set. Credential model
routes are recorded only by their per-value cases to avoid duplicate route
events when aggregate values are replayed.

Red commit `1b923e88` added the catalog-derived completeness assertion. The
focused controlled-peer test failed as expected: only 27 of 58 eligible case
IDs had aggregate fixtures. Green commit `71cc7083` removed the fixed allowlist,
generates those local aggregate fixtures from the catalog, and avoids duplicate
Credential model evidence.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` (artifact SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`):

```sh
bash ../../bindings/java/gradlew --no-daemon --max-workers=2 \
  --init-script build/java-test-tmp.init.gradle \
  -Dkmipkit.test.tmp="$PWD/build/java-tmp" \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The full Java suite passes 52 tests with no failures, errors, or skips. The
shared validator accepts all 21,840 invoked cells with zero failed cells and
`selected_scope_complete=true`; the checked-in ledger records 49,615 pending
cells out of 71,455 planned. Refactor/documentation updates the ledger,
traceability and validator README. The remaining matrix and deployed-server
interoperability are still incomplete.

## Profile Name reserved-range per-target route completion

Sources: OASIS KMIP 2.0 §11.40 and KMIP 2.1 §11.41 (Profile Name
Enumeration). The catalog defines the reserved allocation as
`0x00000001–0x00000103`; the local scalar representative is its lower boundary,
`0x00000001`. The per-value case now runs the generic Enumeration constructor,
Item constructor, Item view, and TTLV decoder under KMIP 2.0 and 2.1, completing
eight Java cells.

Red commit `ab1d2497` requires a per-target fixture and fails because the
reserved-range case was excluded from the generic per-value fixtures. Green
commit `be555145` routes reserved enumeration ranges through the existing
catalogued lower-bound parser and updates the fixture inventory assertion to
require that representative.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` (artifact SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`):

```sh
bash ../../bindings/java/gradlew --no-daemon --max-workers=2 \
  --init-script build/java-test-tmp.init.gradle \
  -Dkmipkit.test.tmp="$PWD/build/java-tmp" \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The Java suite passes 53 tests with no failures, errors, or skips. The shared
validator accepts 21,848 invoked cells with zero failed cells and
`selected_scope_complete=true`; 49,607 of 71,455 planned cells remain pending.
Refactor/documentation updates the checked-in ledger, traceability, README, and
this evidence. Profile Information structure-member coverage and the full Java
matrix remain pending.

## Per-target bitmask value route completion

Sources: OASIS KMIP 2.0 and 2.1 §§12.1–12.3 (Cryptographic Usage Mask,
Protection Storage Mask, and Storage Status Mask). The Java controlled-peer
fixture derives per-value cases directly from the catalog and requires one
fixture for each of the 44 local bitmask value cases. It invokes the applicable
generic Integer constructor, Item creation, Item view, and decoder routes; the
two bitmasks with typed attributes additionally run `TypedAttribute.create`,
`TypedAttribute.fromItem`, and `AttributeSet.tryNew`. Reserved and extension
bits retain the exact catalogued bit pattern. This closes 592 per-value route,
version cells: 350 Cryptographic Usage Mask, 210 Protection Storage Mask, and
32 Storage Status Mask.

Red commit `e0635863` added a catalog-derived completeness assertion and failed
because no per-target bitmask cases had Java fixtures. Green commit `17bf07eb`
adds target-specific fixtures alongside the existing six aggregate cases and
emits only the routes applicable to each mask's public attribute surface.

Verification against installed release commit
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` (artifact SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`):

```sh
bash ../../bindings/java/gradlew --no-daemon --max-workers=2 \
  --init-script build/java-test-tmp.init.gradle \
  -Dkmipkit.test.tmp="$PWD/build/java-tmp" \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The Java suite passes 54 tests with no failures, errors, or skips. The shared
validator accepts 22,440 invoked cells with zero failed cells and
`selected_scope_complete=true`; 49,015 of 71,455 planned cells remain pending.
Refactor/documentation updates the ledger, traceability, README, and this
evidence. The complete Java matrix remains pending.

## Ephemeral and New Attribute Java route repair

Sources: OASIS KMIP 2.0/2.1 §8.3 (Ephemeral); §5.7 Tables 148/163 (New
Attribute Structure); Add Attribute §6.1.2 Tables 152/167; Modify Attribute
§6.1.33 Table 247 in 2.0 and §6.1.34 Table 265 in 2.1; Set Attribute §6.1.47
Table 296 in 2.0 and §6.1.51 Table 322 in 2.1.

The original Java CI run `43498198852`, job `132143734742`, failed in “Validate
invoked Java routes and write the pending coverage ledger.” Its uploaded
`java-route-capability-evidence` confirmed the pinned installed Java API
surface. The failure was route evidence: the merged shared contract added
eight planned cells that the checked-in ledger did not include. Inspection of
the pinned Java API confirmed that all six New Attribute request cases are
supported by installed typed `Structure` routes: each operation request takes
the direct attribute `TtlvItem`, wraps it as the §5.7 one-item Structure, and
provides `toTtlvPayload()`, `toBatchItem()`, network execution, and typed
response readers. They are not unavailable routes.

Red commit `b78b7a23d` adds assertions for the new Ephemeral and New Attribute
local route obligations. The focused controlled-peer test fails because the
Ephemeral Boolean constructor evidence is absent. Green commit `649e42da1`
constructs and inspects the Ephemeral Request Batch Item under both versions,
then adds fixtures for all six versioned Add/Modify/Set Attribute payload
cases. Each New Attribute fixture asserts the single direct Name Item shape,
invokes local Structure construction, Item creation/view and decoding, invokes
both the direct `ClientBatchItem` constructor and request-model
`toBatchItem()`/`toTtlvPayload()` routes, sends both items through a mutually
authenticated controlled TLS peer, and checks the successful typed response
identifier. No route is marked unavailable or skipped. Refactor commit
`020115628` centralizes the operation fixture metadata so case IDs, route IDs,
operation values and assertions use one table.

Verification on the installed Java artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
```

The focused test and full Java suite pass (54 tests, no failures, errors, or
skips). The shared validator accepts 22,570/22,570 cells in
the declared Java selected scope, with zero failed cells and
`selected_scope_complete=true`. That scope adds 10 Ephemeral cells and 120
New Attribute cells. The generated full ledger has 71,463 planned, 22,570
executed, and 48,893 pending cells across 1,324 case IDs. Every pending cell
is still present in the matrix and has only the generic reason
`route_evidence_not_emitted`; no source-backed non-applicability or API
unavailability reason is recorded. Applicability is determined by the
catalog, installed-route capability manifest, protocol versions, supported
encoding/transport axes, and source-backed exclusions before cells are
planned. A planned cell is applicable. Under KMIPKIT-0092-FR-002 and FR-009,
these 48,893 rows are unfinished required coverage, not an accepted exception.

The current Java CI step validates the report's declared `case_ids` selected
scope and compares the generated ledger with its checked-in reference. It does
not require `complete_matrix=true` or zero global pending cells. Therefore a
green exact-head CI result for this selected scope is not completion of the
approved KMIPKIT-0092 matrix; the PR must remain WIP until all applicable Java
routes are evidenced or an approved source-backed exclusion removes them
from the matrix. No vendor interoperability claim is made by these local
controlled TLS fixtures.

## KMIP 2.1 Object Group member routes

Sources: OASIS KMIP 2.1 Specification §4.35 Tables 99–100 defines Object
Group as a Text String attribute, and §7.23 Table 375 marks the Object Group
member in Object Groups as optional and repeatable. The catalog target is
`KMIPKIT-ELEM-STRUCTURE-MEMBER-7-23-OBJECT-GROUP`, with tag `0x420056` and
three applicable local Java routes: `TtlvItem.create`, `TtlvItem.view`, and
`TtlvCodec.decode`.

Red commit `f80c82c39` requires all three route events for the source-backed
case and the focused test fails because the Item creation event is missing.
Green commit `5aba6f55f` constructs and checks an Object Group Text String,
decodes it through the installed generic API, and verifies the containing
Object Groups structure with the member both absent and repeated twice.
Refactor commit `79f32d323` extracts the optional/repeated shape and Item
assertions while preserving the evidence rows.

Verification uses the installed Java artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
```

The focused Java test and full Java suite pass (54 tests, no failures, errors,
or skips). The shared validator accepts 22,573/22,573 cells in the declared
selected scope, with zero failed cells. This slice adds three cells; the
generated full ledger now has 71,463 planned, 22,573 executed, and 48,890
pending cells across 1,323 case IDs. All pending cells remain planned and
carry `route_evidence_not_emitted`; the matrix is not complete. The previously
exact-head run `43537492866` passed before this slice; CI for the new head must
be checked separately.

## KMIP 2.1 Constraint Structure member routes

Sources: OASIS KMIP 2.1 Specification §7.6 Table 357 defines optional
Attributes (`0x420125`), Object Groups (`0x420166`), and Object Types
(`0x420167`) Structure members of Constraint (`0x420169`). Section 7.7
Table 358 defines the optional, repeatable Constraint member (`0x420169`) in
Constraints (`0x420168`). The catalog provides four local Java cases, each
with four applicable routes: `TtlvValue.structure`, `TtlvItem.create`,
`TtlvItem.view`, and `TtlvCodec.decode`.

Red commit `dc77279a5` requires the four per-case route sets and fails because
the installed Structure constructor event is absent. Green commit `db6c9c5e5`
invokes each installed constructor, Item creation/view, and decoder; it checks
absence and presence for each §7.6 field and absence plus two repeated
Constraint members for §7.7. Refactor commit `9784fa06f` reuses one
source-backed fixture table for execution, assertions, and report selection.

The focused controlled-peer test and shared selected-scope validator pass on
the installed Java artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`. The validator accepts 22,589 of
22,589 selected cells with zero failures. The four cases add 16 cells; the
full ledger has 71,463 planned, 22,589 executed, and 48,874 pending cells
across 1,319 pending case IDs. Every pending cell remains planned and carries
`route_evidence_not_emitted`; `complete_matrix` remains false.

The full installed Java consumer suite also passes: 54 tests, no failures,
errors, or skips. The generated route ledger matches its checked-in reference;
the full-matrix gate remains incomplete.

## KMIP 2.1 Asynchronous Correlation Value member routes

Sources: OASIS KMIP 2.1 Specification §7.1 Table 352 defines the optional
Asynchronous Correlation Value Byte String member of the Asynchronous
Correlation Values list; §7.2 Table 353 defines the required Byte String
member of an Asynchronous Request. Section 11.56 Table 487 assigns the
Asynchronous Correlation Values container and Byte String member tags. The
catalog and capability matrix plan
five cells for §7.1: the generic `TtlvValue.ByteString`, `TtlvItem.create`,
`TtlvItem.view`, and `TtlvCodec.decode` routes plus the typed
`QueryAsyncRequestsRequest.withCorrelationValues` route. Section 7.2 plans
the four generic routes. Both cases apply to KMIP 2.1 only.

Red commit `d3222f268` requires all nine per-case route events; the focused
test fails because the §7.1 ByteString constructor invocation is missing.
Green commit `07f0a1099` invokes all four generic routes for both members and
checks the exact tag and binary value through the installed decoder. For
§7.1, it also invokes the typed filter with two ordered values, verifies their
exact ByteString members, and checks that an unfiltered request omits the
optional filter. The fixture distinguishes the `Asynchronous Correlation
Values` container tag `0x420176` from the singular ByteString member tag
`0x420006`, as specified by the pinned tag inventory. Refactor commit
`ccffad7b9` uses fixture metadata to select the typed route and calculate the
expected route-event count.

Verification uses the installed Java artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-tmp" \
  bash ../../bindings/java/gradlew --no-daemon --max-workers=2 \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The focused Java test and full installed Java consumer suite pass (54 tests,
no failures, errors, or skips). The shared validator accepts all 22,598 of
22,598 selected cells with zero failed cells. This slice adds nine cells; the
full ledger has 71,463 planned, 22,598 executed, and 48,865 pending cells
across 1,317 pending case IDs. Each pending cell remains planned with reason
`route_evidence_not_emitted`; this selected-scope result does not set
`complete_matrix=true` and does not make the pending work an exclusion.

## KMIP 2.1 Asynchronous Request required scalar member routes

Sources: OASIS KMIP 2.1 Specification §7.2 Table 353 requires Operation,
Processing Stage, and Submission Date within an Asynchronous Request Structure;
§11.56 Table 487 assigns their tags. The local matrix requires the four generic
TTLV routes for each member. The Operation case also requires the typed
`AsyncOperationOutcome.responsePayloadTtlv` accessor, because that installed
client route returns the checked Query Asynchronous Requests response payload.

Red commit `055caa4f9` requires the three case IDs and all 13 route events. The
controlled-query test fails on the missing Operation Enumeration constructor
event, with an expected event count of 22,611 against the previous 22,598.
Green commit `37032f9b4` invokes the installed Enumeration and DateTimeExtended
constructors, Item creation/view, and decoder for each member. It also sends a
typed Query Asynchronous Requests operation through mutual TLS, calls
`AsyncOperationOutcome.responsePayloadTtlv`, and verifies the exact response
Structure tags, order, values, status, and request operation at the peer.
Refactor commit `ec0d651cb` closes every borrowed Item view and selects response
field fixtures by tag. The shared route validator then identified a misspelled
DateTimeExtended route ID; commit `7db3f9524` corrected it to the canonical
`datetimeextended` ID from the generated matrix.

Verification against the installed Java artifact built from
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

- Focused `ControlledTlsPeerTest`: pass.
- Full installed Java consumer suite: 54 tests, zero failures, errors, or skips.
- Shared route validator: 22,611/22,611 selected cells invoked, zero failed;
  selected scope complete.
- Generated ledger comparison: matches the checked-in reference.
- Full Java ledger: 71,463 planned, 22,611 executed, 48,852 pending across
  1,314 case IDs. Every pending cell remains `route_evidence_not_emitted`;
  `complete_matrix=false` and no pending cell is treated as an exemption.

## KMIP 2.1 Asynchronous Request Structure routes

Source: OASIS KMIP 2.1 Specification §7.2 Table 353 defines the Asynchronous
Request Structure with required Operation, Processing Stage, and Submission
Date members; §11.56 Table 487 assigns the Structure and member tags. The
catalog and capability matrix plan six local cells for this Structure:
`TtlvStructure.create`, `TtlvStructure.withItem`, `TtlvValue.structure`,
`TtlvItem.create`, `TtlvItem.view`, and `TtlvCodec.decode`.

Red commit `d720d6118` requires the six events and fails because the installed
Structure constructor route has no invocation evidence. Green commit
`eb55eedca` builds the three-member Structure through the installed Java API,
checks exact tags, order, TTLV types, and values, then validates the same
structure through the installed decoder. The event report is limited to the
six planned route cells and the case is included in the selected-scope case
IDs. Refactor consolidates the KMIP 2.1 local-route assertions and refreshes
the checked-in ledger and traceability counts. The runtime implementation
already exposed all six routes; this slice adds invocation and fixture
evidence.

Verification used the installed artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The full Java consumer suite passes 54 tests with no failures, errors, or
skips. The shared validator accepts 22,617/22,617 selected cells with zero
failed cells. This slice adds six cells; the full ledger has 71,463 planned,
22,617 executed, and 48,846 pending cells across 1,313 pending case IDs.
Every pending cell remains `route_evidence_not_emitted`; `complete_matrix`
remains false and the pending work remains required.

## KMIP 2.1 Object Groups and Object Types Structure routes

Sources: OASIS KMIP 2.1 Specification §7.23 Table 375 defines Object Groups
as a Structure with repeatable Object Group Text String members; §7.24 Table
376 defines Object Types as a Structure with repeatable Object Type
Enumeration members. The six generic Structure/Item routes apply to each
case.

Red commit `1cab944a6` adds assertions for both six-cell cases and fails on the
missing Object Groups `TtlvStructure.create` event. Green commit `1b2717dfa`
constructs and decodes a valid member for each Structure using the installed
Java artifact, then checks the exact outer/member tags, TTLV types, value,
and order. Refactor consolidates the shared six-route evidence writer and
assertion table, and refreshes the checked-in ledger, README, and FR-003
traceability row.

Verification used the installed artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The focused test and full Java consumer suite pass; the suite has 54 tests,
with no failures, errors, or skips. All 33 feature/compliance traceability tests
pass. The shared validator accepts 22,629/22,629 selected cells with zero
failures. This slice adds 12 cells; the full ledger has 71,463 planned,
22,629 executed, and 48,834 pending cells across 1,311 pending case IDs. Both
operation-structure cases are removed from the pending set because their
planned routes now have invocation evidence. All other pending cells remain
`route_evidence_not_emitted`; `complete_matrix=false`.

## KMIP 2.0/2.1 scalar operation-structure routes

Sources: Data Length is Integer in KMIP 2.0 §7.6 and KMIP 2.1 §7.10;
Final Indicator is Boolean in §§7.10/7.14; Interop Identifier is Text String
in §§7.12/7.16; Log Message is Text String in §§7.15/7.19; and MAC Data is
Byte String in §§7.16/7.20. The respective field tags are `0x4200C4`,
`0x4200D8`, `0x420161`, `0x420141`, and `0x4200C6`. Each case has six
applicable local generic routes for both protocol versions: structure create,
structure with-item, the matching primitive value constructor, Item create,
Item view, and decode. KMIP 2.0 §11.54 and KMIP 2.1 §11.56 define the
corresponding tag allocations.

Red commit `14cf4c020` adds assertions for all six routes and both protocol
versions; the focused test fails because Data Length lacks its
`TtlvStructure.create` invocation event in KMIP 2.0. Green commit `4569bcfd9`
constructs and inspects the scalar field in a generic Structure, separately
decodes the exact tagged scalar Item, and checks its tag, TTLV type, and exact
value. The five fixtures add 60 route cells. Refactor regenerates the shared
ledger and updates the installed-consumer README and FR-003 traceability with
the new counts and the still-pending matrix state.

Verification used the installed Java artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
python3 -m unittest tools.normative_catalog.tests.test_feature_traceability \
  scripts.tests.test_requirement_traceability
```

The installed Java suite passes 54 tests with zero failures, errors, or skips;
all 33 feature and requirement traceability tests pass. The shared validator
accepts 22,689/22,689 selected cells with zero failed cells. The full ledger
has 71,463 planned, 22,689 executed, and 48,774 pending cells across 1,306
case IDs. Each pending cell remains classified as `route_evidence_not_emitted`;
none was reclassified or removed. The applicability filter is the shared
catalog and installed-route capability matrix after version, route,
encoding/transport, and source-backed exclusion checks. Every cell still
planned is applicable under FR-002/FR-009, so this selected-scope pass does not
complete the Java matrix and the PR remains WIP.

## KMIP 2.0/2.1 authenticated-encryption and Init Indicator routes

Sources: Authenticated Encryption Additional Data is a Byte String in KMIP 2.0
§7.1 and KMIP 2.1 §7.3; Authenticated Encryption Tag is a Byte String in
KMIP 2.0 §7.2 and KMIP 2.1 §7.4. Both are marked optional in §7.3/§7.4 of
KMIP 2.1. Init Indicator is Boolean in KMIP 2.0 §7.13 and KMIP 2.1 §7.17.
Their field tags are `0x4200FE`, `0x4200FF`, and `0x4200D7`; the assigned tag
allocations are in KMIP 2.0 §11.54 and KMIP 2.1 §11.56. Each case has six
applicable local generic routes in both protocol versions.

Red commit `166210d3a` asserts all six routes for the three fields; the focused
test fails because Authenticated Encryption Additional Data lacks its
`TtlvStructure.create` invocation event in KMIP 2.0. Green commit `1ae449bfd`
invokes the generic primitive, structure, Item, view, and decoder routes and
checks exact tags, TTLV types, and values. It also verifies that each optional
Authenticated Encryption member can be absent from the Structure and that its
present form retains the exact Byte String value. The three fixtures add 36
route cells. Refactor consolidates the scalar fixture table and refreshes the
generated ledger and route-count documentation.

Verification used the installed Java artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
python3 -m unittest tools.normative_catalog.tests.test_feature_traceability \
  scripts.tests.test_requirement_traceability
```

The installed Java suite passes 54 tests with zero failures, errors, or skips;
the shared validator accepts 22,725/22,725 selected cells with zero failures.
The full ledger has 71,463 planned, 22,725 executed, and 48,738 pending cells
across 1,303 case IDs. All remaining cells retain `route_evidence_not_emitted`;
the shared catalog and installed-route capability matrix still determines
applicability, and the completeness requirement remains open under FR-002 and
FR-009.

## KMIP 2.0/2.1 Correlation Value routes

Correlation Value is a Byte String in KMIP 2.0 §7.4 and KMIP 2.1 §7.8; its
tag is `0x4200D6`, allocated in KMIP 2.0 §11.54 and KMIP 2.1 §11.56. The
fixture runs six generic Structure, primitive, Item, view, and decoder routes
for each version. The controlled pending Query response also exercises the
implemented typed `ClientBatchItemResult.pendingCorrelationValue` accessor for
both versions and checks the exact correlation bytes.

Red commit `a23be1946` requires the six generic routes and fails because the
Correlation Value fixture lacks its `TtlvStructure.create` event. Green commit
`0c8482daf` invokes and asserts the generic routes in both versions. A separate
typed-route Red commit `ee33bf953` then fails because the implemented pending
correlation accessor is not attributed to the Correlation Value case. Green
commit `847a7314b` invokes the accessor on the controlled decoded response,
checks its returned bytes, and emits the typed route event for this case. The
fixture consolidation and evidence updates are recorded in the Refactor commit.

Verification used installed Java artifact pin
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` with SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
python3 -m unittest tools.normative_catalog.tests.test_feature_traceability \
  scripts.tests.test_requirement_traceability
```

The Java consumer suite passes 54 tests with zero failures, errors, or skips.
The shared validator accepts 22,739/22,739 selected cells with zero failed
cells. The full ledger has 71,463 planned, 22,739 executed, and 48,724 pending
cells across 1,302 case IDs. Every remaining cell retains
`route_evidence_not_emitted`: under the shared catalog/capability rule, planned
cells are applicable unless protocol version, route, encoding/transport, or a
source-backed exclusion removes them. No pending cell has been proven excluded
or unavailable. The Java selected-scope CI gate and ledger equality check do
not require zero pending cells, so passing that gate is not full-matrix
completion; FR-002/FR-009 remain open.

## KMIP 2.0/2.1 Extension Information routes

KMIP 2.0 §7.9 Table 329 and KMIP 2.1 §7.13 Table 365 define Extension
Information as a Structure; its tag allocation is in §§11.54/11.56. The
fixture checks the required Extension Name and all six optional fields in
source order: Extension Tag, Extension Type, Extension Enumeration, Extension
Attribute, Extension Parent Structure Tag, and Extension Description. It
invokes the six generic Structure/Item routes and the installed typed
`ExtensionInformation.toTtlv` route in both versions, for 14 cells total, and
asserts the encoded tags, types, values, decoded structure, and typed output.

Red commit `f08676be4` adds route requirements and fails because the fixture
does not yet emit the Extension Information `TtlvStructure.create` invocation.
Green commit `f19c98464` constructs the generic fixture, checks the exact
seven-field TTLV shape, decodes it, then invokes the public typed builder and
`toTtlv` encoder and checks the same shape. Green follow-up `56bb1fe4` adds the
new case ID to the report and updates its expected case count after the first
full-suite run exposed that metadata mismatch. No route requirement was
removed or weakened.

Verification used the installed Java artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` with SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
python3 -m unittest tools.normative_catalog.tests.test_feature_traceability \
  scripts.tests.test_requirement_traceability
```

The focused `ControlledTlsPeerTest` and full Java consumer suite pass; the full
suite reports 54 tests with no failures, errors, or skips. The shared validator
accepts 22,753/22,753 selected cells with zero failed cells, and both ledger
comparison commands pass. The full ledger has 71,463 planned, 22,753 executed,
and 48,710 pending cells across 1,301 case IDs. Every pending cell remains
`route_evidence_not_emitted`. Applicability is determined by the shared
normative catalog and installed-route capability matrix after protocol-version,
route, encoding/transport, and source-backed exclusion rules; all cells still
planned are applicable and remain required coverage under FR-002/FR-009. The
completeness gate requires zero pending cells, while this Java CI step checks
only its declared selected scope and ledger equality; it does not yet enforce
global completeness. The PR remains WIP.

## KMIP 2.0/2.1 Profile Information and Profile Version routes

KMIP 2.0 §§7.25–7.26, Tables 345–346, and KMIP 2.1 §§7.31–7.32,
Tables 383–384 define Profile Information and Profile Version. The fixture
checks required Profile Name, optional Profile Version, Server URI, and Server
Port in table order. When Profile Version is present, it contains both
required Integer members, Profile Version Major and Minor. It also constructs
a Profile Information structure with only the required Profile Name to verify
that its three optional members may be absent.

The local matrix adds 64 invocations in each selected protocol scope combined:
six generic Structure/Item routes for each outer structure, plus constructor,
Item creation, Item view, and decode routes for each of the five member cases,
in KMIP 2.0 and 2.1. The installed Query consumer fixture now includes a real
Profile Information item in its controlled mutual-TLS response. It invokes
`QueryResponse.profileInformation()` and checks the returned Structure, all
four fields, their order and types, and the nested Profile Version values. The
capability manifest declares no separate route ID for this convenience getter;
its invocation and value assertions are therefore functional consumer evidence,
without adding an invented route cell to the ledger.

Red commit `a3154579` adds route requirements and fails on the missing Profile
Information `TtlvStructure.create` event. A second Red commit `46b409f8` adds
the installed `QueryResponse.profileInformation()` assertion and fails because
the controlled response has no Profile Information field. Green commit
`6ea7a42f` invokes and checks both structures and all five member fixtures,
then updates the controlled TLS Query response and the raw batch-item shape
assertion. The Refactor commit records the evidence and refreshes the ledger;
no route requirement or applicability rule was relaxed.

Local verification used the installed Java artifact pinned to
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` with SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
python3 -m unittest tools.normative_catalog.tests.test_feature_traceability \
  scripts.tests.test_requirement_traceability
```

The focused controlled-peer test and full Java consumer suite pass; the suite
reports 54 tests with zero failures, errors, or skips. The route validator
accepts 22,817/22,817 selected cells with zero failed cells. The full ledger
records 71,463 planned, 22,817 executed, and 48,646 pending cells across 1,294
case IDs. Every pending cell has reason `route_evidence_not_emitted`: no
validated per-cell invocation evidence has been recorded for it. Those cells
remain applicable because the shared catalog and route-capability matrix keep
them planned after protocol-version, route, encoding/transport, and
source-backed exclusion rules. No pending cell is recategorized as unsupported
or non-applicable. Under FR-002/FR-009, completeness still requires zero
pending cells. The Java workflow's selected-scope validation and ledger
comparison do not enforce that global condition, so this passing local slice
does not complete the full matrix; the PR remains WIP.

## KMIP 2.1 Object Type structure-member routes

KMIP 2.1 §7.24 Table 376 defines Object Type as an Enumeration member of the
Object Types Structure; the member is optional and may be repeated. The
fixture uses assigned Symmetric Key (`0x00000002`) and Public Key
(`0x00000003`) values from §11.34. It decodes an empty Object Types Structure
to verify absence, then decodes two ordered Object Type members and checks both
tags, TTLV types, and values. The standalone member also exercises the
installed Enumeration constructor, Item creation, Item view, and decoder
routes under its structure-member case ID.

Red commit `4c90412d` adds the case ID and requires all four routes; the
focused controlled-peer test fails on the missing Enumeration-constructor
event. Green commit `d02218b8` constructs, views, and decodes the member,
checks optional and repeated parent shapes, and emits the four route events.
The Refactor commit updates the generated ledger and traceability evidence;
no applicability rule or route requirement changes.

Local verification used installed Java artifact
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` with SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
python3 -m unittest tools.normative_catalog.tests.test_feature_traceability \
  scripts.tests.test_requirement_traceability
```

The focused test and full Java consumer suite pass; the suite reports 54 tests
with zero failures, errors, or skips. The route validator accepts 22,821/22,821
selected cells with zero failed cells. The full ledger records 71,463 planned,
22,821 executed, and 48,642 pending cells across 1,293 case IDs. Each pending
cell remains `route_evidence_not_emitted`, meaning no validated route execution
evidence was emitted for that planned cell. Applicability continues to follow
the shared catalog and capability matrix after protocol-version, route,
encoding/transport, and source-backed exclusion filters; every still-planned
cell remains required. FR-002/FR-009 completeness still requires zero pending
cells, while the Java selected-scope job and ledger comparison do not enforce
global completion. This slice does not complete the matrix; the PR remains WIP.

## KMIP 2.1 Constraint and Constraints operation structures

KMIP 2.1 §7.6 Table 357 defines Constraint as a Structure with optional Object
Types, Object Groups, and Attributes fields. KMIP 2.1 §7.7 Table 358 defines
Constraints as a Structure with an optional, repeatable Constraint member. The
fixtures invoke all six generic Structure/Item routes for each outer structure.
The Constraint fixture includes Object Types (one assigned Symmetric Key) and
Object Groups (one Text String), while omitting optional Attributes; an empty
Constraint is also decoded. The Constraints fixture has two ordered Constraint
children; an empty Constraints form verifies the repeated child may be absent.
All nested tags, types, values, and field order are asserted after construction
and decoding.

Red commit `3e0aca24` requires both structures' six generic routes and fails on
the missing Constraint `TtlvStructure.create` event. Green commit `c827cab6`
constructs and decodes both structures, checks empty and populated branches,
and emits all twelve route events. The Refactor commit updates the generated
ledger and traceability documentation without changing planned applicability.

Local verification used installed Java artifact
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` with SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
python3 -m unittest tools.normative_catalog.tests.test_feature_traceability \
  scripts.tests.test_requirement_traceability
```

The focused test and full Java consumer suite pass; the suite reports 54 tests
with zero failures, errors, or skips. The route validator accepts 22,833/22,833
selected cells with zero failed cells. The full ledger records 71,463 planned,
22,833 executed, and 48,630 pending cells across 1,291 case IDs. All pending
cells remain `route_evidence_not_emitted` and therefore remain planned,
applicable requirements with missing per-cell execution evidence. This reason
means no validated invocation event exists for a cell; it is not an unsupported
API or a non-applicability decision. The shared catalog and route-capability
matrix apply protocol-version, route, encoding/transport, and source-backed
exclusion rules before retaining these planned cells. FR-002/FR-009 require
zero pending cells for matrix completeness; the Java selected-scope workflow
and ledger comparison do not enforce the global requirement. This slice does
not complete the Java matrix, and the PR remains WIP.

## KMIP 2.0/2.1 Signature Data routes

KMIP 2.0 §7.32 and KMIP 2.1 §7.38 define Signature Data as a Byte String
used to transfer signature data in cryptographic operations; its tag is
allocated in §§11.54/11.56. The selected Java fixture exercises six applicable
local generic routes in both protocol versions: `TtlvStructure.create`,
`TtlvStructure.withItem`, `TtlvValue.ByteString`, `TtlvItem.create`,
`TtlvItem.view`, and `TtlvCodec.decode`. It asserts the exact tag, Byte String
type, and fixture bytes through construction, view, and decode.

Red commit `44642769` requires these routes and fails on the missing
`TtlvStructure.create` event for KMIP 2.0. Green commit `db754a62` adds the
catalogued Signature Data fixture to the existing scalar operation-structure
runner, which invokes and verifies all six routes in both versions. The
Refactor commit refreshes the ledger and route documentation; no route cell is
removed or treated as inapplicable.

Local verification used installed Java artifact
`489fafc79ef89db2524eafa7a6c6482f5cf75d97` with SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`:

```sh
TMPDIR="$PWD/build/native-test-tmp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/native-test-tmp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
python3 -m unittest tools.normative_catalog.tests.test_feature_traceability \
  scripts.tests.test_requirement_traceability
```

The focused test and full Java consumer suite pass; the suite reports 54 tests
with zero failures, errors, or skips. The route validator accepts 22,845/22,845
selected cells with zero failed cells. The full ledger records 71,463 planned,
22,845 executed, and 48,618 pending cells across 1,290 case IDs. Every pending
cell remains `route_evidence_not_emitted`, meaning a validated invocation
record has not been emitted for that planned cell. The shared catalog and
capability matrix first apply protocol-version, public-route,
encoding/transport, and source-backed exclusion rules; any cell still planned
is applicable. No pending cells are API unavailability exemptions. FR-002 and
FR-009 require zero pending cells for matrix completion. The Java selected
scope and ledger comparison do not enforce global completion, so this slice
does not complete the matrix and the PR remains WIP.

## KMIP 2.0/2.1 Server Information routes

KMIP 2.0 §7.31 and KMIP 2.1 §7.37 Table 389 define Server Information as a
Structure with optional members. The Java consumer now exercises the outer
Structure's six generic routes and 39 member routes per version: nine
Text String structure-member cases each use the `TtlvValue.TextString`
constructor, Item creation, Item view, and decoder, while the repeatable
Vendor-Specific Any member uses its three applicable Item routes. The complete
fixture checks all eight singleton values, two ordered Alternative Failover
Endpoints, and two heterogeneous vendor values; an empty Server Information
structure verifies that all members are optional. The controlled mutual-TLS
Query response invokes `QueryResponse.serverInformations()` and checks its
12 child tags, types, order, and values. That convenience getter has no
separate route ID in the matrix.

Red commit `3c2693c9c452ee3b6e6e607696c675e056d9df74` adds the case IDs,
per-route assertions, and 90-cell event expectation; the focused test fails on
the missing `TtlvStructure.create` event for KMIP 2.0. Green commit
`1ab98ffd88a366878c208c60be92602ef9ab434a` adds the installed constructor,
Item/view/decoder invocations, typed Query accessor assertion, and controlled
response fixture. Refactor updates the evidence ledger, traceability, and
consumer documentation; no route cell is removed or deemed inapplicable.

Local verification used installed Java artifact
`489fafc79ef89db2524eafa7a6c6482f5cf75d97`, SHA-256
`830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e`:

```sh
env TMPDIR="$PWD/build/tmp" TMP="$PWD/build/tmp" TEMP="$PWD/build/tmp" \
  JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/tmp" \
  bash ../../bindings/java/gradlew -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 ../../interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report build/route-execution-evidence.json \
  --artifact-sha256 830db52367b5887fbd6e1e2ad6a927a1bb52aab318b04d22cef6e8ba9041813e \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The focused test and all 54 installed-consumer tests pass with zero failures,
errors, or skips. The selected-scope validator accepts 22,935/22,935 invoked
cells with zero failures. The full ledger records 71,463 planned, 22,935
executed, and 48,528 pending cells across 1,279 case IDs. Every remaining cell
is still marked `route_evidence_not_emitted`. The shared catalog and capability
matrix first apply protocol-version, public-route, encoding/transport, and
source-backed exclusion rules; a cell remaining in the matrix is applicable.
No pending cell is classified as an API unavailability exemption. FR-002 and
FR-009 require zero pending cells for matrix completion. The selected-scope
workflow and ledger comparison do not enforce global completion, so the PR
remains WIP.

## KMIP Tag inventory aggregate routes

Red commit `b9094b27c` added an assertion that every generic TTLV Tag and
allocation-range invocation also satisfies the `test-enum-tag` aggregate. The
focused controlled-peer test failed because the aggregate report omitted those
cells. The failure exposed 50 assigned Tag identities that had no generic
route invocation: their per-case typed route set was only partially available,
so neither the generic-only nor the all-typed selection included them.

Green commit `5ab78dd52` executes those 50 identities through the installed generic
TTLV API under the aggregate case, duplicates existing per-identity and
allocation-range evidence under the aggregate case, and checks that duplicate
matrix cells have identical outcomes. This produces all 3,674 planned Tag
aggregate cells across KMIP 2.0/2.1 with no duplicate or unplanned route event.
The Refactor updates this evidence, traceability, README, and generated ledger;
it does not remove or reclassify any planned route cell.

Local verification used installed artifact SHA-256
`504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545`:

```sh
bash ../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 ../../interop/shared/tools/validate_route_evidence.py \
  --report build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --selected-scope --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The focused controlled-peer test passes. The selected-scope validator accepts
26,627/26,627 invoked cells with zero failures. The full ledger now records
71,649 planned, 26,627 executed, and 45,022 pending cells across 1,278 case
IDs. All pending cells remain `route_evidence_not_emitted`; the full matrix
remains incomplete and T006/T007/T009 remain open. The PR must remain WIP.

## Java Boolean field route execution

Red commit `4651a7c4c` required generic TTLV evidence for catalogued Boolean
message fields. Red commit `a62eda031` added a focused controlled-peer assertion
and reproduced the missing-route failure. Green commit `98a4dabc9` executes all
12 selected Boolean message-field and structure-member cases through their
applicable generic TTLV routes. The valid two-item request exercises
`RequestMessage.fromTtlv` for Attestation Capable Indicator and Batch Order
Option; request and response fixtures exercise Criticality Indicator through
both typed message parsers. Random IV is exercised via
`CryptographicParameters.Builder.randomIv`, `build`, `CryptographicParameters.fromTtlv`,
`TypedAttribute.create`, `TypedAttribute.fromItem`, and `AttributeSet.tryNew`
(KMIP 2.0/2.1 §§4.16, 9.3, 9.8, 9.13; Tables 58–60).

The generated ledger was refreshed only after the selected report passed the
repository validator. The full Java consumer test suite passes 56/56 tests;
the selected-scope report validates 26,745/26,745 cells with zero failed
invocations, and the normal ledger comparison passes. The ledger now records
71,649 planned, 26,745 executed, and 44,904 pending cells across 1,266 case
IDs. Pending cells remain `route_evidence_not_emitted`; T006, T007, and T009
remain open and the PR must remain WIP.

Verification commands:

```sh
TMPDIR="$PWD/build/java-temp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
bash ../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 interop/shared/tools/validate_route_evidence.py \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --selected-scope --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
```

## Java KMIP 2.0/2.1 Response Header Server Correlation Value — 2026-11-17

The previously committed Red assertions require the four generic TTLV and
typed `ResponseMessage.fromTtlv` routes for
`test-norm-message-field-message-field-8-5-server-correlation-value`. They
failed while the controlled Query response omitted the optional field. The
Green fixture now includes the catalogued Text String at tag `0x420106`, checks
the parsed `ResponseMessage.serverCorrelationValue()`, and emits route evidence
for KMIP 2.0 and 2.1. The event-count assertion accounts for all four routes in
both versions. The normative basis is OASIS KMIP 2.0/2.1 §8.5 and the versioned
Response Header field tables in §9.3.

The focused controlled-peer test passes, and the complete Java consumer suite
passes with no failures. The selected-scope validator accepts 27,360/27,360
cells with zero failed cells. The refreshed ledger records 71,641 planned,
27,360 executed, and 44,281 pending cells. This closes eight cells; the full
Java matrix remains incomplete and the PR remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" test
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```
## Java repeatable Any object-attribute members — 2026-11-17

Red commit `d9285a466` requires the generic Item creation, Item view, and
decoder routes for the six repeatable `Any attribute in §4 - Object
Attributes` members in KMIP 2.0/2.1 §§5.1–5.7. The focused test failed because
the catalog-driven route executor was absent. Green commit `957355db1` uses a
well-formed Name attribute (KMIP 2.0/2.1 §4.32; tags §§11.54/11.56), exercises
absent, single, and repeated member counts, and emits each required route for
both protocol versions. Existing route evidence is checked first to prevent
duplicate cells.

The focused local test and full Java consumer suite pass (58 tests, no skips or
failures). The evidence validator accepts 27,396/27,396 selected cells with
zero failed cells. The ledger records 71,641 planned, 27,396 executed, and
44,245 pending cells. This closes 36 cells; the full matrix remains incomplete
and the PR remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --tests org.kmipkit.validator.LocalCatalogRouteChecksTest.executesRepeatableObjectAttributeMemberThroughGenericTtlvRoutes
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" test
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java KMIP 2.0/2.1 repeatable Response Header Attestation Type — 2026-11-17

OASIS KMIP 2.0/2.1 §8.5 specifies an optional, repeatable Attestation Type in
the Response Header. Red commit `855778c41` adds exact per-version assertions
for generic Item create/view/decode and typed `ResponseMessage.fromTtlv`; the
controlled-peer test fails because the target has no events. Green commit
`d0471de8c` creates and decodes the generic Enumeration and parses a response
with two Attestation Type values. The typed `ResponseHeaderView` returns both
values in wire order. Refactor commit `31f5fb590` shares the values used by the
request and response fixtures. The first selected-scope validation found that
the evidence report's explicit `case_ids` omitted this target; commit
`4e1a6866a` includes it and adjusts the report-size assertion.

The focal route test passes. The complete Java consumer suite passes 57/57
tests with no failures, errors, or skips. The shared route-capability suite
passes 68/68 tests, and `route_capabilities.py --check` reports 1,507 routes.
The selected-scope validator accepts 27,344/27,344 cells with zero failed
cells; the generated ledger contains 71,641 planned, 27,344 executed, and
44,297 pending cells across 1,209 pending case IDs. The ledger comparer passes.
This is local-model evidence and does not establish server interoperability or
complete the Java route matrix; the PR remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-temp" \
  bash bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  -p interop/java-kmip-validator test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-temp" \
  bash bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  -p interop/java-kmip-validator test --rerun-tasks
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java KMIP 2.0/2.1 Response Header Client Correlation Value — 2026-11-17

OASIS KMIP 2.0/2.1 §8.5 defines the optional Client Correlation Value in the
Response Header. Red commit `6e85b8b63` requires generic Item create/view/decode
and typed `ResponseMessage.fromTtlv` evidence under KMIP 2.0 and 2.1. The
initial run exposed an artifact manifest that pointed to the #217 worktree;
after reinstalling the pinned artifact locally, the focused test fails because
the target has zero route events. Green commit `220eeec3f` constructs and
decodes a Text String with the catalogued tag and parses it through
`ResponseMessage.fromTtlv`; `ResponseHeaderView.clientCorrelationValue()` must
return the fixture value. Refactor commit `06d74f08b` shares that fixture value
between construction and assertion.

The focal test passes after Green and Refactor. The full Java suite passes
57/57 tests with no failures, errors, or skips; shared route-capability tests
pass 68/68; the route generator reports 1,507 routes. Selected-scope validation
accepts 27,352/27,352 cells with zero failures. The generated ledger records
71,641 planned, 27,352 executed, and 44,289 pending cells across 1,208 pending
case IDs. The ledger comparer passes. This closes eight local-model cells; it
does not establish external server interoperability or complete the Java
matrix, so the Java PR remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-temp" \
  bash bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  -p interop/java-kmip-validator test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-temp" \
  bash bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  -p interop/java-kmip-validator test --rerun-tasks
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java KMIP 2.0/2.1 repeatable Request Header Attestation Type — 2026-11-17

Source: OASIS KMIP 2.0 Specification §8.2 and KMIP 2.1 Specification §8.2.
The catalog marks Attestation Type optional and explicitly repeatable. Red
commit `2c869bcab` adds exact per-version assertions for generic Item create,
Item view, TTLV decode, and typed `RequestMessage.fromTtlv`; the focused test
fails because all eight expected route/version cells are absent.

Green commit `5b985bfd0` constructs and decodes the Attestation Type
Enumeration, then parses two values in the Request Header and asserts that the
typed `RequestHeaderView.attestationTypes()` preserves `[1, 2]` in wire order.
The fixture places the repeated field after Attestation Capable Indicator, as
required by the §8.2 Request Header ordering. Refactor commit `d0766d376`
centralizes the values and four route IDs. The focused controlled-peer class
passes 19/19 tests; the complete Java suite passes 57/57 tests with no skips,
failures, or errors.

Selected-scope validation accepts 27,336/27,336 cells with zero failed cells.
The regenerated ledger records 71,641 planned, 27,336 executed, and 44,305
pending cells across 1,210 pending case IDs. This slice closes eight cells,
one per route and protocol version pair for the four applicable Java routes.
The ledger comparer passes. Evidence uses pinned artifact commit
`12496d646d11d4069b58bcc88fc76cc619211ec6` with SHA-256
`504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545`.
These are local-model routes, not vendor interoperability or full matrix
completion; the Java PR remains WIP.

Verification commands:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
```

## Java Item Type enum routes

Red commit `8534357d6` adds a focused assertion for the 11 catalogued Item
Type enum values and their four generic TTLV routes. It fails because the
general enum recipe requires a same-named standalone Tag, which the §11.23
Item Type enum does not have. Green commit `86887bae4` adds a dedicated local
fixture that reads the enum values from the catalog, validates each through
`TtlvItemType.fromCode`, then constructs and decodes a generic TTLV Enumeration
Item for each code in KMIP 2.0 and 2.1.

Verification:

```sh
TMPDIR="$PWD/build/java-temp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
bash ../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.LocalCatalogRouteChecksTest \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 interop/shared/tools/validate_route_evidence.py \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --selected-scope --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
```

The selected report validates 26,833/26,833 cells with zero failures; the
ledger records 71,649 planned, 26,833 executed, and 44,816 pending cells across
1,265 case IDs. The full matrix remains incomplete and the PR remains WIP.
The complete Java validator Gradle suite passes 57/57 tests with zero failures,
errors, or skips. The selected-scope report, generated ledger refresh, normal
ledger comparison, and `git diff --check` all pass.

## Java Item Type individual value cases

Red commit `875ccfa80` changes the focused fixture expectation so each of the
11 catalogued Item Type values must emit routes under its individual normative
case ID as well as under the aggregate. It fails while the fixture emits only
the aggregate. Green commit `57c6b2160` maps each value target to its unique
catalog case, emits the four generic TTLV Enumeration routes under both case
IDs, and adds the new individual IDs to the controlled report scope.

Verification:

```sh
TMPDIR="$PWD/build/java-temp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
bash ../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.LocalCatalogRouteChecksTest \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
TMPDIR="$PWD/build/java-temp" \
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
bash ../../bindings/java/gradlew -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 interop/shared/tools/validate_route_evidence.py \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --selected-scope --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
```

The selected report validates 26,921/26,921 invoked cells with zero failures;
the ledger contains 71,649 planned, 26,921 executed, and 44,728 pending cells
across 1,254 case IDs. The complete Java validator suite passes 57/57 tests
with zero failures, errors, or skips. The full matrix remains incomplete and
the PR stays WIP.

## Java scalar fields, batch option, and asynchronous payload slice

Red: the new `test_async_response_payload_accessor_claims_only_response_payload_fields`
failed because name matching assigned the generic asynchronous response-payload
accessor 12 targets, including Message Field 9.15 Operation. The controlled
fixture returns the KMIP 2.1 Asynchronous Request operation structure (§7.2
Table 353); its Operation member is a structure member, not Message Field 9.15.
The source-backed route override now names only the Asynchronous Request
structure. The same controlled TLS fixture invokes `AsyncOperationOutcome.responsePayloadTtlv`
for Query Asynchronous Requests in KMIP 2.1 and Cancel in KMIP 2.0 and 2.1,
checking the payload values and order.

The controlled pending Query fixture now exercises `ClientBatch.addErrorContinuationValue`
with two Query items and verifies the option on the serialized request header
for KMIP 2.0 and 2.1. The first experiment used a single-item batch and failed
at the existing `SingleItemBatchErrorContinuation` validation; the corrected
fixture follows the rule that Batch Error Continuation Option requires
Batch Count greater than one. The response contains both correlated results
under Continue. The real `ClientBatchItemResult.uniqueBatchItemId()` call is
also recorded against Unique Batch Item ID (§9.21). These fixtures emit no
raw protocol body to evidence.

The scalar-field slice adds generic constructor, Item, view, and decoder routes
for the applicable Boolean, Integer, Enumeration, DateTime, Text String, and
Byte String header fields, and exercises `RequestMessage.fromTtlv`,
`ResponseMessage.fromTtlv`, and `Nonce.fromTtlv` on valid KMIP 2.0/2.1
messages. The vendor-identification `Text String` target is now normalized
from its restricted source-encoding description, so all four languages retain
the appropriate generic constructor route.

Verification on 2026-11-17:

```sh
python3 -m unittest \
  interop.shared.tests.test_route_capabilities.RouteCapabilitiesTests.test_async_response_payload_accessor_claims_only_response_payload_fields \
  interop.shared.tests.test_route_capabilities.RouteCapabilitiesTests.test_text_string_constructor_covers_normatively_restricted_text_encoding
python3 interop/shared/tools/route_capabilities.py --check
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
python3 ../../interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 tools/compare_route_ledger.py --refresh-reference
python3 tools/compare_route_ledger.py
```

The focused TLS test and both focused shared tests pass. Route validation
reports 27,135/27,135 selected Java cells invoked, zero failed cells, and a
complete selected scope. The generated ledger records 71,641 planned cells,
27,135 executed cells, and 44,506 still pending across the global matrix.
The normal ledger comparison matches the checked-in reference. This is not
full KMIPKIT-0092 closure; the remaining global cells are still required, and
the PR remains WIP.

## Batch Error Continuation Option Tag route — 2026-11-17

Red: a new assertion requiring `ClientBatch.addErrorContinuationValue` evidence
under the individual Tag case `test-norm-tag-tag-42000e` failed with two expected
version events and zero observed. Green: the Java controlled-peer test now
records `RawTag.fromRaw`, `RawTag.checked`, and the typed batch setter for
`KMIPKIT-ELEM-TAG-42000E`. The setter event is emitted only after the real
two-item Query request is captured and its header is asserted to contain the
Batch Error Continuation Option tag and value in KMIP 2.0 and 2.1 (§11.54 and
§11.56 respectively). This adds 12 validated cells to the selected Java report.

Verification on 2026-11-17:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
```

The focused TLS test and full Java suite pass (57/57 tests, no failures,
errors, or skips); all 68 shared route-capability tests pass. Route validation
reports 27,147/27,147 selected cells invoked with zero failures. The full Java
matrix remains incomplete and the PR stays WIP.

## Java Ping route — 2026-11-17

The catalogued KMIP 2.1 Ping case (`test-op-ping`, OASIS KMIP 2.1 §6.1.36)
now invokes the Java installed-package factory and response reader through a
controlled TLS 1.3 mutual-TLS exchange. The fixture confirms the peer received
Operation Ping and that the returned `ClientBatchItemResult.pingResponse()` is
present and true. Its Operation Enumeration is separately constructed, viewed,
and decoded with the installed generic TTLV APIs. This completes all seven
Java cells selected for this case: three local TTLV routes, typed request
construction/send/response decode over TTLV raw TLS, and the typed Ping
response reader.

Red, commit `812c615f6`:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
```

The test failed because `test-op-ping` had no observed
`client_batch_item_response_ping` route event. Green, commit `4f2e96761`, adds
the controlled Ping exchange and its generic TTLV events. The focused test and
the complete Java Gradle suite pass; the latter completed all 57 tests with no
failures, errors, or skips. All 68 shared route-capability tests pass, the
route capability generator check passes, and the evidence validator accepts
27,154/27,154 selected cells with zero failures. Its generated ledger contains
71,641 planned, 27,154 executed, and 44,487 pending cells. The checked-in
reference was refreshed with `compare_route_ledger.py --refresh-reference`
from the validated build ledger and then matched by the comparer. The full
matrix remains incomplete, so the PR stays WIP.

## Set Endpoint Role generic TTLV routes — 2026-11-17

The Java local recipe now covers the required Endpoint Role request field and
Set Endpoint Role Operation Enumeration from the KMIP 2.1 catalog case
`test-norm-payload-client-set-endpoint-role-request-21-001`. It constructs,
views, and decodes each Enumeration through the installed generic TTLV API,
preserving the field tag `0x420151` and Operation value `0x32`. The normative
basis is OASIS KMIP 2.1 §6.1.54 Table 331, §11.19 (Endpoint Role Enumeration),
and §11.36 (Operation Enumeration).

Red, commit `87f60844c`:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
```

The test failed because the six expected generic route events for the payload
field and Operation Enumeration were absent. Green, commit `56a2401ae`, adds
both real constructor/view/decode sequences and includes their catalog case in
the generated report. The focused controlled-peer test passes; the complete
Java suite passes 57/57 tests with no failures, errors, or skips. All 68 shared
route-capability tests pass, the generator check passes, and selected-scope
validation accepts 27,160/27,160 cells with zero failures. The generated
ledger records 71,641 planned, 27,160 executed, and 44,481 pending cells. The
ledger comparer passes after regenerating from the validated build report.
Full-matrix closure remains required, so the PR stays WIP.

Verification commands:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java KMIP 2.1 Set Endpoint Role response routes — 2026-11-17

Red, commit `c0d99e17a`, adds the response case ID and per-route assertions; the
focused controlled-peer test fails because the response's `TtlvItem.create`
route has no event. Green, commit `9eecdae9e`, round-trips the required
Endpoint Role response field and Set Endpoint Role Operation Enumeration via
the installed generic TTLV constructor, Item view, and decoder. The fixture
uses Client value 1 as the role accepted by the server (OASIS KMIP 2.1
§6.1.54 Table 332; §11.19); Operation is `0x32` (§11.36). This adds six
selected cells. The full Java suite passes 57/57 tests with `--rerun-tasks`.
Refactor, commit `4382e0ea0`, extracts a shared helper for request and response
Endpoint Role fields; the focused and full tests pass on the refactored code.
All 68 route-capability tests pass, the generator check reports 1,507 routes,
and selected-scope validation accepts 27,240/27,240 cells with zero failures.
The ledger records 71,641 planned, 27,240 executed, and 44,401 pending cells
across 1,222 pending case IDs. The ledger comparer passes after refresh. The
global matrix remains incomplete and the PR stays WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java Extension Tag Item routes — 2026-11-17

Red, commit `db63682e0`, adds per-version assertions for Extension Tag's
Integer constructor, Item creation, Item view, and decoder routes. The focused
controlled-peer test fails as expected because the Integer constructor event
is absent for
`test-norm-structure-member-structure-member-7-13-extension-tag`.

Green, commit `89b26b958`, executes those four routes for KMIP 2.0 and 2.1 and
checks the exact member tag, Integer type, and value `0x540001` (an Item Tag
Value in the Extensions range). The normative references are KMIP 2.0 §7.9
Table 329 and KMIP 2.1 §7.13 Table 365; the member tag is allocated by
§§11.54/11.56. The fixture asserts local generic TTLV behavior and does not
claim server interoperability.

The focused controlled-peer test and full Java suite pass (57 tests, no
failures or errors). All 68 shared route-capability tests pass, the generator
check reports 1,507 routes, and selected-scope validation accepts 27,234 of
71,641 Java route cells with zero failures; 44,407 cells remain pending. The
generated ledger comparer passes after refresh. The full matrix remains open,
so #187 and its dependent stacks remain WIP.

Verification commands:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java KMIP 2.0 Set Endpoint Role request and response routes — 2026-11-17

Red, commit `9c01f2e22`, adds per-case assertions for the required Endpoint
Role Enumeration and Set Endpoint Role Operation cells in the KMIP 2.0 request
and response cases. The focused controlled-peer test fails at the missing
`TtlvItem.create` event for the request field. Green, commit `6132e4fbb`,
constructs, views, and decodes each field and Operation Enumeration through the
installed generic TTLV API. It uses the Client value (1) as the request and
response fixture value for the role the server is asked to apply, as defined by
OASIS KMIP 2.0 §6.1.48 Tables 299–300 and §11.19; Operation value `0x32` is
defined by §11.36. This adds 12 selected cells across two cases.

The focused test and full Java suite pass; the complete suite reports 57/57
tests without failures, errors, or skips. All 68 shared route-capability tests
pass, the generator check passes for 1,507 routes, and selected-scope validation
accepts 27,172/27,172 cells with zero failures. The generated ledger records
71,641 planned, 27,172 executed, and 44,469 pending cells across 1,231 pending
case IDs. The ledger comparer passes after regeneration. Full-matrix coverage
remains incomplete and the PR stays WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java Attestation Capable Indicator request-header routes — 2026-11-17

Red, commit `c40efe3d8`, asserts the three generic TTLV routes and typed
`RequestMessage.fromTtlv` route for
`test-norm-message-field-message-field-8-2-attestation-capable-indicator` in
KMIP 2.0 and 2.1. The focused test fails as expected (`expected 2 route events,
got 0`). The target's normative basis is OASIS KMIP 2.0 §8.2 and §9.3 Table
364 and KMIP 2.1 §8.2 and §9.3 Table 402. The controlled request fixture
contains a true Attestation Capable Indicator, and the typed parser asserts
that value.

Green, commit `3271bd8f1`, builds the Boolean Item from the catalogued target
tag, validates the generic item view and decoder, emits all three generic
events per protocol version, and includes the 8.2 target in typed request
parsing. Refactor, commit `60f4eb8e1`, shares the Boolean message-field
fixture with the existing Batch Order Option case. The focused controlled-peer
test passes after both changes.

The full Java suite passes 57 tests with no failures, errors, or skips. The
selected scope validates 27,270/27,270 cells, with zero failed cells; the
71,641-cell global matrix retains 44,371 pending cells across 1,218 case IDs.
The ledger comparison passes after regeneration. These results use pinned Java
artifact commit `12496d646d11d4069b58bcc88fc76cc619211ec6` with SHA-256
`504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545` and
provide local-model evidence, not vendor interoperability or complete matrix
coverage.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions --rerun-tasks
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java KMIP 2.0/2.1 shared Request Header scalar routes — 2026-11-17

Red commit `661d07efa` requires the three generic TTLV Item routes and typed
`RequestMessage.fromTtlv` for the seven scalar fields catalogued under the
shared §8.2 Request Header: Asynchronous Indicator, Batch Count, Batch Error
Continuation Option, Client Correlation Value, Maximum Response Size, Server
Correlation Value, and Time Stamp. The focused test failed because those exact
§8.2 cells had no emitted routes. Green commit `d8a64d9ed` resolves the shared
field encodings from the matching version-specific KMIP 2.0/2.1 definitions
(§§9.2, 9.5, 9.6, 9.9, 9.10, 9.12, and 9.20), constructs and decodes each
field through the generic Item API, and parses a valid two-item RequestMessage
fixture. The parser asserts the header values; Server Correlation Value is
checked through the public generic header TTLV view because the typed header has
no direct accessor for it. The Batch Error Continuation Stop value also reaches
the installed `ClientBatch.addErrorContinuationValue` API. This adds 58
selected cells: eight per field and one additional BEC setter cell per version.

Refactor commit `50d4b494d` centralizes the shared §8.2 case-ID prefix. The
focused controlled-peer test passes after Green and Refactor. The full Java
suite passes 57/57 tests with no skips or failures; all 68 shared route
capability tests pass; the generated route check confirms 1,507 routes. The
selected-scope validator accepts 27,328/27,328 cells with zero failed cells.
The refreshed Java ledger records 71,641 planned, 27,328 executed, and 44,313
pending cells across 1,211 pending case IDs. The ledger comparer passes. This
is local-model route evidence, not vendor interoperability or a complete Java
matrix; the Java PR remains WIP.

Verification commands:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test \
  --tests org.kmipkit.validator.ControlledTlsPeerTest.controlledQueryProducesPerRouteEvidenceForBothProtocolVersions \
  --rerun-tasks
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/build/java-temp" \
  bash ../../bindings/java/gradlew --no-daemon \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test --rerun-tasks
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py \
  --selected-scope --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
```

## Java scalar structure-member routes — 2026-11-17

Red commit `d54861542` adds a focused requirement for generic TTLV construction
and decoding of a scalar structure member, using the exact “Attribute Name”
target in OASIS KMIP 2.0/2.1 §5.5. It failed before the route executor existed.
Green commit `77a8577a2` builds source-identified scalar members from the
catalogued name, encoding, assigned tag, and version, then exercises
`TtlvValue`, `TtlvItem.create`, `TtlvItem.view`, and `TtlvCodec.decode`. The
selector consults the generated Java capability manifest and only adds cases
whose applicable route set is generic TTLV; cases that also require typed
managed-object routes remain pending until those routes have their own
complete fixture.

This adds 31 scalar structure-member cases and 248 complete cells for KMIP
2.0/2.1. The cases cover Attribute Name and Vendor Identification, Extension
Information scalars, Key Wrapping Data scalars, PKCS#11 Function and Return
Code, Profile Name, RNG Parameters scalars, Ticket scalars, Usage Limits Unit,
Validation Information scalars, and Server Information scalar fields. The
selected scope validates 27,644/27,644 cells with zero failed cells; the
71,641-cell Java matrix now has 27,644 executed and 43,997 pending across
1,170 pending case IDs. The installed artifact remains pinned at
`12496d646d11d4069b58bcc88fc76cc619211ec6` (SHA-256
`504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545`). The
full Java validator suite passes 59 tests with no failures or errors. This
evidence is local TTLV route coverage, not vendor interoperability or a
complete Java matrix; PR #187 remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" test
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java server Query scalar payload routes — 2026-11-17

Red commit `a1ea1b088` requires construction and decoding evidence for the
server-initiated Query Function field and its dual catalog identity. It failed
before a fixture existed. Green commit `87adc236c` selects source-backed
values from the normative catalog and exercises the installed generic TTLV
Item constructor, view, and decoder for both the payload-field target and its
operation target. The fixtures cover Query Function (Query Operations), Object
Type (Symmetric Key), Vendor Identification, Attestation Type (TPM Quote), and
Client Registration Method (Client Generated), for KMIP 2.0/2.1. The values
follow OASIS KMIP 2.1 §6.2.4 and §§11.4, 11.10, 11.34, and 11.44; tags use
§11.56. No server listener or sending route is claimed.

The selected scope validates 27,704/27,704 cells with zero failed cells. The
Java matrix records 71,641 planned, 27,704 executed, and 43,937 pending cells
across 1,160 pending case IDs. The full Java validator suite passes 60/60 tests
with no skips or failures. The 68 route-capability tests pass, the generated
route check confirms 1,507 routes, and the refreshed ledger compares cleanly.
The installed artifact remains the pinned release artifact recorded above;
these are local TTLV results, not vendor interoperability or a complete Java
matrix. PR #187 remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" test
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java server Notify scalar payload route — 2026-11-17

Red commit `fff7fcf83` adds a focused check for the server-initiated Notify
`Unique Identifier` payload field from OASIS KMIP 2.0/2.1 §6.2.2. It failed at
test compilation because the shared server-payload executor did not exist.
Green commit `b0fb13c32` extends the generic TTLV fixture executor to support
that catalogued text field, constructs it with the published `Unique
Identifier` tag, and checks the value through Item construction, Item view,
and decoding. The validator emits the three generic routes for both the
Notify field target and its operation target; this records local TTLV route
evidence and does not claim an inbound server listener or vendor exchange.

The two version-specific Notify cases add 12 complete cells. The selected
scope validates 27,716/27,716 cells with zero failures. Of the 71,641 Java
matrix cells, 27,716 are executed and 43,925 remain pending across 1,158 case
IDs. The full Java validator suite passes 61 tests with no skips, failures,
or errors; the 68 route-capability tests pass and the generated route check
confirms 1,507 routes. The refreshed evidence ledger compares with the
checked-in reference.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" test
python3 -m unittest interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
git diff --check
```

## Java scalar structure routes and explicit cell scopes — 2026-11-18

Red commits `513ce6d75` and `bfb348a47` require a source-assigned value for the
`Certificate Type` enumeration and generic TTLV evidence even when a scalar
structure member also has typed routes. The focused tests failed first because
the fixture emitted enum code zero and the controlled report omitted the
generic route cells for typed targets.

Green commit `5c9c355a7` extends generic scalar structure execution to those
typed targets and obtains enumeration fixtures from the catalog's assigned
values or published extension ranges. It handles the KMIP-referenced PKCS#11
enum values and the shared Hashing Algorithm enum for Mask Generator Hashing
Algorithm. This contributes 672 generic TTLV cells. Those cases still remain
pending in the full matrix wherever their other typed routes lack evidence.

The first scoped Java report then exposed that the CLI accepted case scopes but
did not consume the validator's existing exact-cell scope support. Red commit
`8cba59f06` reproduces that mismatch. Green commit `507337f96` adds optional
`scope_cells` to the report schema, validates it against planned cells, and
preserves the full-matrix pending count when writing the ledger. The Java
controlled-peer report now lists its exact executed cell identities. This
allows the validator to prove each selected invocation without treating
unexecuted routes for the same case as completed.

The Java suite passes 62/62 tests with no failures, errors, or skips. The Python
route-evidence, schema, and capability suites pass 108 tests. The generated
route check confirms 1,507 routes; the ledger comparison passes. The selected
scope validates 28,388/28,388 cells with zero failures. The global Java ledger
records 28,388 executed and 43,253 pending of 71,641 cells across 1,158 pending
case IDs. These are local installed-artifact and controlled-peer results, not
vendor interoperability; #187 remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" test --rerun-tasks
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## Client asynchronous correlation payload TTLV routes — 2026-11-18

Red commit `b99d1b3a1` requires generic TTLV item creation, item view, and
codec-decode evidence for client payload correlation fields. Green commit
`67ceb732a` runs those routes against a catalogued Byte String Tag for Cancel
and Poll in KMIP 2.0/2.1 and Process in KMIP 2.1. The controlled peer suite
still verifies the installed async operation routes separately; this increment
only records the local generic TTLV obligations and does not infer the other
payload fields from these correlation fixtures.

The full Java suite passes 62 tests with no failures, errors, or skips. The
selected scope validates 28,418/28,418 cells with zero failures. The Java
ledger records 28,418 executed and 43,223 pending of 71,641 cells across 1,158
pending case IDs. Shared route-evidence, schema, and capability tests pass 108
tests, and the capability check confirms 1,507 routes. The full route matrix
remains incomplete and the PR remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks --refresh-dependencies
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## Client asynchronous correlation response TTLV routes — 2026-11-18

Red commit `bc8c4abef` requires the Cancel response correlation field to run the
same generic Item construction, view, and decoder routes as the request-side
field. Green commit `81f1348de` exercises the catalogued Byte String through
those routes for KMIP 2.0 and 2.1, recording the field and enclosing Cancel
operation identities. The evidence remains a local model roundtrip and does
not claim that a server response was received for those cells.

The full Java suite passes 62 tests with no failures, errors, or skips. The
selected scope validates 28,430/28,430 cells with zero failures. The ledger
records 28,430 executed and 43,211 pending of 71,641 cells across 1,158 pending
case IDs. The Python route-evidence, schema, and capability suites pass 108
tests; the route check confirms 1,507 routes. Full Java matrix completion is
still pending, and #187 remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks --refresh-dependencies
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## KMIP 2.0/2.1 Attributes generic Structure routes — 2026-11-18

Red commit `48bff5e27` adds a focused Java validator test requiring all five
catalogued generic TTLV routes for the KMIP 2.0/2.1 §5.1 `Attributes`
structure; it fails to compile because the route executor does not yet exist.
Green commit `285e22fd8` builds a source-shaped Attribute containing the
`Comment` name and text value, invokes the installed `TtlvStructure.create`,
`TtlvStructure.withItem`, `TtlvItem.create`, item view, and `TtlvCodec.decode`
APIs, then verifies the outer tag and structure shape. The refactor integration
adds those per-version route invocations to the canonical report and ledger.
These are local TTLV model cells; the enclosing test also runs its controlled
TLS query, but these calls are not recorded as network exchanges.

The full Gradle suite passes 65 tests with zero failures, errors, or skips. The
selected report validates 28,440/28,440 cells with zero failures. The ledger
records 28,440 executed and 43,201 pending of 71,641 cells across 1,157 case
IDs. Shared route evidence, schema, and capability suites pass 108 tests, and
the route capability audit confirms 1,507 routes. Full Java matrix completion
is still pending and the PR remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks --refresh-dependencies
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## Structure-valued Attribute generic TTLV routes — 2026-11-18

Red commit `5fb9d2bff` adds a focused test requiring the four generic TTLV
routes for seven source-defined Structure-valued Attributes: Alternative Name,
Application Specific Information, Key Value Location, Link, Name, Revocation
Reason, and KMIP 2.1 Rotate Name. Green commit `58de68136` builds these
structures using their catalogued member tags and source encodings, then
checks member order, types, values, item views, and decoding. The controlled
TLS report records the four generic routes per applicable test case and
protocol version; the structures themselves are local model exercises, not
network exchanges.

The full Gradle suite passes 66 tests with zero failures, errors, or skips. The
selected report validates 28,492/28,492 cells with zero failures. The generated
ledger records 28,492 executed and 43,149 pending of 71,641 cells across 1,156
pending case IDs. The shared route-evidence, schema, and capability suites pass
108 tests; the route-capability audit confirms 1,507 routes. Full Java matrix
completion is still pending and the stack remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## Text String Attribute generic TTLV routes — 2026-11-18

Red commit `660c9deb9` adds a focused test requiring the installed generic TTLV
routes for all five catalogued Text String Attributes: Comment, Contact
Information, Description, Object Group, and PKCS#12 Friendly Name. Green commit
`3a8a315e5` constructs source-shaped text values using each catalogued tag,
checks the item view and decoded bytes, and records TextString construction,
item creation, view, and decode for KMIP 2.0 and 2.1. These are local TTLV
model cells and are not network exchanges.

The full Gradle suite passes 67 tests with zero failures, errors, or skips. The
selected report validates 28,532/28,532 cells with zero failures; the generated
ledger records 28,532 executed and 43,109 pending of 71,641 cells across 1,157
pending case IDs. Shared route-evidence, schema, and capability suites pass 108
tests; the route audit confirms 1,507 routes. Full Java matrix completion is
still pending and the stack remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## Boolean Attribute generic TTLV routes — 2026-11-18

Red commit `983d54687` requires generic TTLV evidence for the nine Boolean
Attribute cases. Green commit `5dc359c20` constructs a source-tagged Boolean
item, verifies its item view and decoded value, and records Boolean construction,
item creation, view, and decode for each applicable version. The report treats
these as local TTLV model routes, not server exchanges. It also updates the
controlled report's expected case count for the new Boolean cases.

The full Gradle suite passes 68 tests with zero failures, errors, or skips. The
selected report validates 28,596/28,596 cells with zero failures; the ledger
records 28,596 executed and 43,045 pending of 71,641 cells across 1,157 pending
case IDs. Shared route-evidence, schema, and capability suites pass 108 tests;
the route audit confirms 1,507 routes. Full Java matrix completion is still
pending and the stack remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## Date-Time Attribute generic TTLV routes — 2026-11-18

Red commit `93e197eda` adds a focused test for the twelve catalogued Date-Time
Attribute cases. Green commit `a8941cc8e` constructs the TTLV DateTime value
using a deterministic timestamp, then checks the tag, item type, decoded value,
and four generic routes. Existing complete Activation Date evidence is reused
by the controlled report; other applicable Date-Time cases are emitted without
duplicating the completed case. These are local TTLV model routes.

The full Gradle suite passes 69 tests with zero failures, errors, or skips. The
selected report validates 28,680/28,680 cells with zero failures; the generated
ledger records 28,680 executed and 42,961 pending of 71,641 cells across 1,157
pending case IDs. Shared route-evidence, schema, and capability suites pass 108
tests, and the route audit confirms 1,507 routes. The full Java matrix remains
incomplete and the stack stays WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## Enumeration Attribute generic TTLV routes — 2026-11-18

Red commit `ae3d0c3ff` adds a focused test requiring generic TTLV routes for nine
Enumeration Attributes. Green commit `87fbf9550` selects the lowest source
assigned enumeration for each Attribute from the normative catalog, constructs
the item through the installed Java API, and verifies tag, type, and decoded
value across applicable KMIP versions. This records local TTLV model behavior,
not a network exchange.

The full Gradle suite passes 70 tests with zero failures, errors, or skips. The
selected report validates 28,752/28,752 cells with zero failures; the generated
ledger records 28,752 executed and 42,889 pending of 71,641 cells across 1,157
pending case IDs. Shared route-evidence, schema, and capability suites pass 108
tests; the route audit confirms 1,507 routes. Full Java matrix completion is
still pending and the stack remains WIP.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```

## Scalar Attribute generic TTLV routes — 2026-11-18

Red commit `5d2381c20` adds the focused test requiring four generic TTLV routes
for ten catalogued Integer, Interval, and Byte String Attributes. The test
failed because the scalar Attribute route executor did not exist. Green commit
`60068f4ee` adds source-tag lookup, typed value construction, item creation,
view, and decode assertions for all applicable KMIP 2.0/2.1 cases. Integer
fixtures use 256 for the two length Attributes and bit value 1 for the usage and
storage masks; Interval uses 60 seconds; the Short Unique Identifier uses a
fixed four-byte fixture. KMIP 2.1-only Rotate Attributes are exercised only
for that version. Follow-up fix commit `da2f4aaf3` aligns the Byte String route
identifier with the public capability manifest (`ttlvvalue.bytestring`).

The full Gradle suite passes. The selected Java report validates 28,820/28,820
cells with zero failures. The generated ledger records 28,820 executed and
42,821 pending cells of 71,641 across 1,157 pending case IDs; full-matrix
completion remains outstanding. The shared route-evidence, schema, and
capability suites pass 108 tests, and the route audit confirms 1,507 routes.
These cases exercise local generic TTLV construction and decoding; they do not
claim Attribute policy enforcement by a server.

Verification:

```sh
JAVA_TOOL_OPTIONS="-Djava.io.tmpdir=$PWD/interop/java-kmip-validator/build/java-tmp" \
  bash bindings/java/gradlew -p interop/java-kmip-validator \
  -PkmipkitMavenRepo="$PWD/interop/java-kmip-validator/build/maven-repository" \
  test --rerun-tasks
python3 interop/shared/tools/validate_route_evidence.py --selected-scope \
  --report interop/java-kmip-validator/build/route-execution-evidence.json \
  --artifact-sha256 504516f1785ec7ad8bead0fb66301708af781e215ac31dd1b4cb1970df257545 \
  --write-ledger
python3 interop/java-kmip-validator/tools/compare_route_ledger.py --refresh-reference
python3 interop/java-kmip-validator/tools/compare_route_ledger.py
python3 -m unittest interop.shared.tests.test_route_execution_evidence \
  interop.shared.tests.test_schemas_and_profiles interop.shared.tests.test_route_capabilities
python3 interop/shared/tools/route_capabilities.py --check
git diff --check
```
