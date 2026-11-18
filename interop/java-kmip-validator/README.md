# KMIPKit Java interoperability validator

This project is an external Java 17 consumer of the installed KMIPKit JNI JAR.
It does not compile the library sources from this checkout. The artifact pin is
in `library-pin.json`; the installer extracts that exact `release/1.0.0`
commit, builds the platform JAR in an isolated source tree, installs it into a
local Maven repository, and writes a SHA-256 provenance manifest. The current
pin is `12496d646d11d4069b58bcc88fc76cc619211ec6`.

Run commands from this directory. `plan` is offline, `preflight` checks local
files and provenance without DNS or sockets, and `run` executes enabled test
routes against the endpoint in the connection file:

```sh
python3 tools/install_artifact.py \
  --repository ../.. \
  --pin-file library-pin.json \
  --maven-repository build/maven-repository \
  --manifest build/artifact-manifest.json

bash ../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test

bash ../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" run \
  --args='plan --connection ../shared/profiles/connection.example.json'
```

A live connection file follows
[`server-connection.schema.json`](../shared/schemas/server-connection.schema.json).
It must confirm a dedicated test environment and provide TLS trust, client
certificate, and private key files. Username and password values are read
only from the configured environment-variable names. KMIPKit zeroizes its
native copies; Java environment `String` values cannot be erased by the
library. Set `server_product`
and `server_version` only to operator-verified metadata; capability profiles
are hypotheses. Each live run must use an isolated KMS configuration.

State-changing catalog cases require both `execution_policy.allow_mutations`
and `--enable-mutations`. Requests are not retried. Reports contain result
codes and sanitized diagnostics, never request or response bodies, credentials,
or remote text. JSON, Markdown, and HTML reports distinguish KMIPKit
correctness, observed server support, policy rejection, parity, and coverage
that has not run.

The live executor routes every catalog operation that has a public
`ClientBatchItem` factory through the installed typed client for KMIP 2.0 and
2.1, subject to each case's version and mutation requirements. It pairs
successful responses with the public typed response reader and preserves
numeric KMIP failure codes. `Log`, object lifecycle, attribute, session,
cryptographic, and other catalog cases that require mutation or isolated setup
remain protected by both mutation opt-ins. The Create recipe generates a
unique named AES-256 Symmetric Key, verifies the typed response, and sends a
separate `Destroy` exchange using the returned Unique Identifier.

Controlled TLS tests invoke every installed public KMIP 2.x
`ClientBatchItem` factory at both protocol versions and check the observed
operation and rejection classification. They also run Query and Get through
the validator CLI. The all-operation sweep returns `Operation Not Supported`,
so it proves construction, TTLV framing, transport reach, and error-response
handling. Successful operation-specific workflows, including seeded object
preconditions, state verification and cleanup for every applicable catalog
case, still require additional controlled response recipes. Those cases must
not be reported as validated interoperability until those recipes and the
full shared route matrix have passed.

The validator constructs and decodes generic TTLV values locally. It does not
send arbitrary generic TTLV requests, and the Java package has no distinct
high-level API route. The package exposes dedicated Java client executors for
Cancel, Poll, Process, and Query Asynchronous Requests. Controlled-peer tests
exercise those installed routes with explicit correlation fixtures. Catalog
live runs keep them pending until a recipe can obtain a server-issued
correlation value; the Java package does not classify those routes as absent.

CI builds the pinned JNI JAR and runs the validator against a local TLS 1.3
mTLS peer with a controlled KMIP response. This verifies the external package
and request/response handling; it is not evidence for a commercial or other
deployed KMS. CI also probes the installed JAR against the shared public-route
manifest and uploads `route-capability-probe.json`. That report establishes
package symbol and named-value availability only; it does not claim that every
catalog route or interoperability case has executed successfully.

The installed-consumer suite records exact local TTLV construction, decoding,
and view calls for the Query operation under KMIP 2.0 and 2.1, then exchanges
typed Query requests and a checked Query response with the controlled TLS
peer under both versions. It directly invokes the public Query response
readers and verifies the returned field count, kind, and raw operation value.
The same test records the expected unused and reserved tag decoder rejections
for both versions, and checks that an extension-range tag can be constructed,
viewed, and decoded from a deterministic wire fixture through the installed
TTLV API. The pinned public `TtlvCodec` exposes no generic encode operation;
typed request bytes are instead checked as captured by the controlled peer.
CI validates these per-cell events against the shared matrix,
uploads `route-execution-evidence.json` with the generated ledger, and compares
stable coverage fields with the checked-in `route-evidence-ledger.json` using
`tools/compare_route_ledger.py`. The comparer reads the checked-in reference and
the generated ledger from fixed project paths, and accepts no path overrides.
After a separately validated run changes the route-manifest or coverage
baseline, regenerate the checked-in ledger with
`python3 tools/compare_route_ledger.py --refresh-reference`; the command
rejects generated ledgers that do not record successful invocations.
The generated ledger retains the artifact
SHA-256 for each build; that build-specific digest is not used as a coverage
baseline. These recipes execute 34,702 of 71,641 planned Java route cells, with
no failed cells; the ledger records the other 36,939 as pending and does not imply
vendor interoperability or full-matrix completion. Local TTLV recipes exercise 51
enumeration families, all six bitmask aggregate cases, and all 44 individual
bitmask value cases through the applicable
installed constructors, item views, decoders, and typed attribute wrappers.
The RSA public Key Material members also pass through the installed typed
`KeyMaterial.fromFields` and `KeyMaterial.fromItem` constructors in KMIP 2.0 and
2.1, using a deterministic non-production modulus and public exponent fixture
(§3.8). This adds eight local typed-model cells and does not claim cryptographic
key validity or server acceptance.
The same typed constructors now cover all 29 catalogued Big Integer members
across transparent DSA, RSA, and DH private/public plus EC private Key Material
formats (KMIP 2.0/2.1 §§3.5–3.11). Fixtures include each source-defined required field
and applicable optional fields; they are local model checks, not cryptographic
key validation. EC public Key Material also exercises the `Recommended Curve`
and `Q String` members from §§3.11–3.12 through all six installed typed
`KeyMaterial`, `KeyValue`, and `KeyBlock` routes for both protocol versions.
They also cover the optional and repeatable Object Group member in KMIP 2.1
§7.23 with all three applicable generic Item routes; §4.35 Tables 99–100
define its Text String attribute representation.
The outer KMIP 2.1 Object Groups and Object Types Structures exercise the six
generic Structure/Item routes each, with exact repeatable Text String and
Enumeration members (§§7.23–7.24, Tables 375–376). The optional, repeatable Object Type member also exercises its Enumeration constructor, Item creation/view, and decoder routes with an empty Object Types structure and two ordered values (Symmetric Key, Public Key; §7.24 Table 376 and §11.34). Constraint and Constraints each complete six generic Structure/Item routes; fixtures check optional Constraint fields, their nested Object Types/Object Groups values, empty forms, and repeated Constraint children (§§7.6–7.7, Tables 357–358).
The separate required Object Defaults member exercises Item creation, view, and decoding
with its version-specific shape: an Object Type Enumeration in KMIP 2.0 (§7.18
Table 338) and an ObjectTypes Structure in KMIP 2.1 (§7.22 Table 374).
The required-if-PBKDF2 Salt member covers its Byte String constructor, Item,
view, and decoder routes in both versions (KMIP 2.0 §7.8 Table 328 and KMIP 2.1
§7.12 Table 364; tag allocation §§11.54/11.56).
The required Extension Name and optional Extension Description members also
cover their Text String constructor, Item, view, and decoder routes in both
versions (KMIP 2.0 §7.9 Table 329 and KMIP 2.1 §7.13 Table 365; tag allocation
§§11.54/11.56).
The optional Extension Tag member covers its Integer constructor, Item creation,
view, and decoder routes in both versions. The fixture uses `0x540001`, an Item
Tag Value in the Extensions range, per the definition in KMIP 2.0 §7.9 Table
329 and KMIP 2.1 §7.13 Table 365; the member tag is allocated in §§11.54/11.56.
Initialization Vector, Derivation Data, and Iteration Count also cover their
typed constructor, Item creation, view, and decoder routes in both versions,
using a Byte String value for the two byte fields and an Integer for Iteration
Count (KMIP 2.0 §7.8 Table 328 and KMIP 2.1 §7.12 Table 364; tag allocations
§§11.54/11.56). The fixture checks field typing and route behavior; it does not
claim to execute a PBKDF2 derivation.
Data Length, Final Indicator, Interop Identifier, Log Message, and MAC Data
exercise their six applicable local generic routes in KMIP 2.0 and 2.1 with
their source-defined Integer, Boolean, Text String, or Byte String values
(KMIP 2.0 §§7.6, 7.10, 7.12, 7.15–7.16; KMIP 2.1 §§7.10, 7.14, 7.16,
7.19–7.20; tag allocations §§11.54/11.56).
Authenticated Encryption Additional Data and Authenticated Encryption Tag
also exercise the six applicable local routes in both versions, with exact
Byte String values and both optional-absent and present Structure shapes;
Init Indicator exercises the Boolean routes (KMIP 2.0 §§7.1, 7.2, 7.13;
KMIP 2.1 §§7.3, 7.4, 7.17; tag allocations §§11.54/11.56).
Correlation Value exercises six local generic routes in both versions and the
typed `ClientBatchItemResult.pendingCorrelationValue` accessor against a
controlled pending Query response (KMIP 2.0 §7.4 and KMIP 2.1 §7.8; tag
allocation §§11.54/11.56). Signature Data exercises its six local Byte String constructor, Structure, Item, view, and decoder routes in both versions (KMIP 2.0 §7.32; KMIP 2.1 §7.38; tag allocations §§11.54/11.56).
Extension Information exercises its six generic Structure/Item routes and the
typed `ExtensionInformation.toTtlv` encoder in both versions; the fixture checks
all seven source-ordered fields and exact values (KMIP 2.0 §7.9 Table 329 and
KMIP 2.1 §7.13 Table 365; tag allocations §§11.54/11.56). Profile Information
and Profile Version each exercise all six generic Structure/Item routes in KMIP
2.0/2.1, and the five Profile Information members exercise constructor, Item,
view, and decode routes in both versions (64 cells total; KMIP 2.0 §§7.25–7.26,
Tables 345–346; KMIP 2.1 §§7.31–7.32, Tables 383–384). The controlled mTLS
Query response also invokes `QueryResponse.profileInformation()` and checks the
returned Profile Name, version, URI, and port; the route manifest has no
separate route ID for this convenience getter. Server Information adds 90 local
cells across KMIP 2.0 §7.31 and KMIP 2.1 §7.37 Table 389: the outer Structure
uses all six generic routes; its nine Text String member cases use the TextString
constructor, Item creation/view, and decode routes; Vendor-Specific Any uses
its three applicable Item routes. Fixtures check the optional empty structure,
all eight singleton Text String values, two ordered Alternative Failover
Endpoints, and two heterogeneous Vendor-Specific values. The controlled mTLS
Query response invokes `QueryResponse.serverInformations()` and checks the
12-field tag order, types, and values; this convenience getter has no separate
route ID in the manifest.
Four optional/repeatable members in the KMIP 2.1 Constraint and Constraints
structures also exercise the installed `TtlvValue.structure`, Item creation,
Item view, and decoder routes (§§7.6–7.7, Tables 357–358).
The two KMIP 2.1 Asynchronous Correlation Value members exercise the installed
ByteString constructor, Item creation, Item view, and decoder routes; §7.1 also
invokes `QueryAsyncRequestsRequest.withCorrelationValues` and checks the
optional filter, repeated values, order, and exact tags (§§7.1–7.2,
Tables 352–353; §11.56 Table 487).
The three required KMIP 2.1 Asynchronous Request fields in §7.2 Table 353
exercise their installed Enumeration or DateTimeExtended constructors, Item
creation, Item view, and decoder routes. The outer Asynchronous Request
Structure also exercises `TtlvStructure.create`, `TtlvStructure.withItem`,
`TtlvValue.structure`, Item creation/view, and decoding with its exact three
required fields. A mutual-TLS Query Asynchronous
Requests exchange invokes `AsyncOperationOutcome.responsePayloadTtlv` and
checks the response's Asynchronous Request Structure, field tags, order, and
values (§11.56 Table 487).
The completed selected scope now includes the Ephemeral Boolean field in KMIP
2.0/2.1 (§8.3) and all six versioned Add, Modify, and Set Attribute New
Attribute cases: each New Attribute is checked as a §5.7 Structure containing
one direct Name Item, and the installed direct constructors, request-to-batch
conversions, TTLV payload conversion, controlled TLS exchange, and typed
response readers are invoked under the operation-specific §6.1 tables.
It also executes all 12 catalogued Boolean message-field and structure-member
cases in both protocol versions through their applicable generic TTLV routes.
Controlled request and response fixtures exercise Criticality Indicator through
`RequestMessage.fromTtlv` and `ResponseMessage.fromTtlv`; a two-item request
checks Attestation Capable Indicator and Batch Order Option. Random IV is
checked through `CryptographicParameters.Builder.randomIv`, `build`,
`CryptographicParameters.fromTtlv`, and its `TypedAttribute` and `AttributeSet`
routes (KMIP 2.0/2.1 §§4.16, 9.3, 9.8, 9.13; Tables 58–60).
The Item Type aggregate and its 11 individual value cases complete their four
generic TTLV Enumeration routes in both versions; each code is cross-checked through
`TtlvItemType.fromCode` before its generic Enumeration item is round-tripped
(KMIP 2.0/2.1 §11.23).

The Batch Error Continuation Option tag `0x42000E` now has per-case Java
evidence for both `RawTag` identity routes and the `ClientBatch` setter. The
setter's evidence comes from the serialized header of a controlled two-item
Query request in KMIP 2.0 and 2.1 (KMIP 2.0 §11.54; KMIP 2.1 §11.56).

The current Java ledger has 41,061 pending cells across 1,114 case IDs; every
one has the reason `route_evidence_not_emitted`. The applicability rule is the
shared catalog and capability matrix after protocol-version, public-route,
encoding/transport, and source-backed exclusion filters. A cell remaining in
that matrix is planned and applicable; the ledger gives none of these cells a
non-applicability or API-unavailable reason. KMIPKIT-0092 FR-002 and FR-009
therefore treat them as unfinished coverage, not accepted exceptions.

Reports may declare `scope_cells` when a run exercises only selected
case/route/action/version/encoding/transport tuples. The validator checks that
each declared cell is planned, appears exactly once, and has a matching passing
invocation; the evidence ledger still measures execution against the complete
matrix. If `scope_cells` is omitted, `case_ids` selects every planned cell for
each listed case. A green selected scope does not mean the full-matrix gate is
satisfied. The PR remains WIP while any required Java route cells are pending.
The `test-enum-tag` aggregate now covers all 3,674 applicable generic TTLV
cells for catalogued tags and tag-allocation ranges in KMIP 2.0 and 2.1. Fifty
individual tag cases still have 652 pending typed-route cells; those remain
required per-case obligations and are not counted as covered by the aggregate.
All 19 Block Cipher Mode values now complete their five applicable generic and
Cryptographic Parameters builder routes in both KMIP 2.0 and 2.1 (§§4.16,
11.6); other enum families remain pending where the ledger says so. The three
Rotate Name Type values complete their four applicable generic TTLV routes in
KMIP 2.1 (§11.51). The §4.53 structure-member cases separately cover
`Rotate Name Value` (Text String) and `Rotate Name Type` (Enumeration); each
completes four generic field routes and three typed routes through the enclosing
`Rotate Name` Attribute, and each fixture includes both required members from
Table 135. NIST Key
Type completes four generic and three typed routes for all 20 values in KMIP
2.0/2.1 (§§4.34, 11.32 in both; tag allocation
§11.54 in 2.0 and §11.56 in 2.1). Key Role Type completes four generic routes
and its Cryptographic Parameters builder for all 25 values in KMIP 2.0/2.1
(§§4.16, 11.26). Hashing Algorithm completes four generic routes and its
Cryptographic Parameters builder for all 18 values in KMIP 2.0/2.1 (§§4.16,
11.21). Padding Method completes four generic routes and its Cryptographic
Parameters builder for all 11 values (§§4.16, 11.37). Link Type completes all four generic routes for its 15 catalogued
values in KMIP 2.0/2.1 (§11.28). Opaque Data Type completes
its four generic and three typed
attribute routes (§11.35); Processing Stage, Ticket Type, and Mask Generator
complete all four generic routes, with the Mask Generator Cryptographic
Parameters builder route also covered (§§11.40, 11.55/11.57, 11.30).
Encoding Option, Endpoint Role, Key Value Location Type, Key Wrap Type, Name
Type, Object Group Member, Put Function, Secret Data Type, and Usage Limits
Unit each complete all four generic TTLV routes for their catalogued values
in KMIP 2.0 and 2.1 (§§11.18, 11.19, 11.27, 11.29, 11.31, 11.33, 11.42/11.43,
11.50/11.52, 11.58/11.60).
All 57 Cryptographic Algorithm values now complete four generic TTLV routes,
three typed attribute routes, and the Cryptographic Parameters builder route
in KMIP 2.0 and 2.1 (§§4.16, 11.12).
All 43 Profile Name targets complete four generic TTLV routes in KMIP 2.0 and
2.1 (§11.40 in 2.0; §11.41 in 2.1), including the catalogued reserved-range
boundary and extension allocation. The reserved-range target also has its own
per-value generic routes in both protocol versions. Profile Information
structure-member cases remain pending. Catalog-driven local enumeration
fixtures also cover all 58 eligible aggregate cases whose enum targets have a
same-named standalone tag, preventing newly catalogued eligible aggregates
from silently remaining without generic-route evidence.
Other enum families, attribute routes, and operations remain pending. The
Activation Date recipe exercises generic and typed TTLV routes, both public
Get Attributes constructors, the typed response reader, and both protocol
versions against the controlled peer. No vendor KMS is deployed.

The local catalog recipes also complete 264 Tag cases whose approved Java
surface is entirely generic, including 20 reserved tags. Each assigned tag
checks its exact identity through raw and checked construction, Item creation,
the Item view, and decoding; reserved tags check the actual constructor and
decoder rejections. Another 110 Tag cases have additional typed model
obligations: Message Extension was already covered by its controlled fixture,
and the other 109 remain pending. These tag-identity fixtures do not
assert that arbitrary scalar values satisfy a tag's operation-specific schema.

All 189 referenced PKCS#11 values (92 Function and 97 Return Code values)
exercise the installed Enumeration constructor, Item constructor, Item view,
and decoder in both protocol versions. They preserve each exact unsigned
value and its Function or Return Code tag and compare normalized tree hashes
across those routes. This includes the normative vendor-defined allocation
boundary; it does not execute vendor commands or claim vendor interoperability.
The two aggregate catalog cases for PKCS#11 Function and Return Code now also
record all 1,512 value/route/version cells under their aggregate case IDs, so
the aggregate rows no longer remain pending while the per-value rows pass.
The Recommended Curve aggregate exercises all 71 catalogued values through
the same four generic routes in KMIP 2.0/2.1 (§§11.44/11.45), adding 568
value/route/version cells.
The Result Reason aggregate exercises all 78 values through those routes in
both versions (§11.45 in KMIP 2.0; §11.46 in KMIP 2.1), adding 624 cells.
Three values first assigned in KMIP 2.1 are also round-tripped as unknown
Enumeration values in KMIP 2.0.
The Operation aggregate exercises all 60 values through the same four routes
in KMIP 2.0 and 2.1 (§11.36 in both), adding 480 cells. Six Operation values
first assigned in KMIP 2.1 are also preserved as unknown in KMIP 2.0. The Key
Format Type aggregate covers all 24 values through the four generic and three
typed attribute routes; the controlled Get response also reads its Raw value
through `ClientBatchItemResult.raw` in both versions (§§8.6, 9.7, 11.25), adding
338 cells total. The Digital
Signature Algorithm aggregate covers all 20 values through generic TTLV,
typed attribute, AttributeSet, and Cryptographic Parameters builder routes in
both versions (§§4.16, 11.16), adding 320 cells.

The recorded [development evidence](../../specs/092-interop-api-route-exhaustiveness/java-route-expansion-evidence.md)
keeps the Red/Green/Refactor commands and the distinction between this completed
local scope and the pending complete matrix.

The KMIP 2.1 Ping case now records all seven selected Java route cells. The
controlled mTLS fixture sends a real `ClientBatchItem.ping()` request, checks
the peer-observed Operation, and reads the successful response through
`ClientBatchItemResult.pingResponse()`. The same case round-trips the Operation
Enumeration through the installed generic TTLV item constructor, view, and
decoder, as required by KMIP 2.1 §6.1.36. The selected Java report now validates
28,752/28,752 cells; 42,889 of the full 71,641-cell Java matrix remain pending,
so this does not complete the Java matrix. The Set Endpoint Role request field
and Operation now have generic TTLV item creation, view, and decode evidence for
the KMIP 2.1 request and both KMIP 2.0 request and response cases (KMIP 2.0
§6.1.48 Tables 299–300; KMIP 2.1 §6.1.54 Table 331; §11.19; §11.36).
The Batch Order Option message-field case completes three generic TTLV routes
and `RequestMessage.fromTtlv` in both KMIP 2.0 and 2.1 (§8.2; §9.8 Tables 369
and 407).
The Attestation Capable Indicator case completes the same four routes in both
versions (§8.2; §9.3 Tables 364 and 402).

The KMIP 2.0/2.1 §5.1 `Attributes` structure now exercises the installed
`TtlvStructure.create`, `TtlvStructure.withItem`, `TtlvItem.create`, item view,
and `TtlvCodec.decode` routes using one source-shaped Attribute with a Comment
name and text value. The selected report validates 28,752/28,752 cells; 42,889
of 71,641 remain pending, so the matrix and PR remain WIP.

The scalar Attribute cases for Integer, Interval, and Byte String now exercise
the installed value constructors, `TtlvItem.create`, item view, and decoder.
The fixtures cover Certificate Length, Cryptographic Length and Usage Mask,
Protection Storage Mask, Lease Time, Protection Period, Short Unique Identifier,
and the KMIP 2.1 Rotate Generation, Rotate Interval, and Rotate Offset
attributes. The selected report validates 28,820/28,820 cells with zero
failures; 42,821 of 71,641 remain pending, so this local generic-TTLV evidence
does not complete the Java matrix or demonstrate server policy behavior.

The remaining catalogued Text String targets now use generic value construction,
item creation, view, and decode routes. This completes all 117 planned Java
`TtlvValue.TextString` cells; the selected report validates 28,876/28,876 cells
with zero failures, while 42,765 of the complete 71,641-cell matrix remain
pending. The examples are local TTLV fixtures and do not assert server-side
identifier or profile policy.

The remaining source-catalogued Byte String and Date Time Extended values now
use generic construction, item creation, view, and decode routes. All 74
planned Java ByteString cells and all five DateTimeExtended cells are covered.
The selected report validates 28,924/28,924 cells; 42,717 of 71,641 remain
pending. These local TTLV fixtures do not claim credential, PKCS#11, or
certificate semantic validation.

Catalogued Big Integer key-material fields now use generic construction, item
creation, view, and decode routes with an eight-byte synthetic value. All 62
planned Java BigInteger cells are covered. The selected report validates
29,164/29,164 cells; 42,477 of 71,641 remain pending. No production key
material is used or exposed by these local fixtures.

The last three source-catalogued Enumeration fields now use generic value
construction, item creation, view, and decode routes. These cover Interop
Function, Attribute Reference encoded as a Tag, and Extension Type encoded as
an Item Type. The Java Enumeration constructor route is now complete across all
3,767 planned cells. The selected report validates 29,188/29,188 cells; 42,453
of 71,641 remained pending at that point.

Generic Structure routes now cover all 138 planned `TtlvStructure.create` and
`TtlvStructure.withItem` cells and all 227 planned `TtlvValue.structure` cells.
The fixtures use catalogued Structure tags where available, parent tags for
catalogued members, and a generic extension-range tag for untagged model types.
They verify only local generic TTLV construction, view, and decode behavior;
they do not assert KMIP field semantics or conformance. The selected report
validated 30,110/30,110 cells at that point; 41,531 of 71,641 were then pending.

Assigned Tag cases that also have typed API routes now retain their own generic
`RawTag`, Item creation, Item view, and decode events. This includes tags such
as Authentication whose typed route is not a `StandardAttribute` route. The
generic fixture checks the assigned 24-bit tag and an Integer value; it does
not claim the value is semantically valid for the tag. The selected report now
validates 30,580/30,580 cells; 41,061 of 71,641 remain pending across 1,114
case IDs.

Sixty catalogued operation-target cases now exercise the `Operation` Enumeration
as a generic TTLV Item through `TtlvItem.create`, `TtlvItem.view`, and
`TtlvCodec.decode` for each applicable KMIP version (345 version-specific
cells; §§8.2 and 11.36). This proves local generic TTLV behavior only; it does
not prove operation payload validity or server behavior. The selected report
validates 30,925/30,925 cells; 40,716 of 71,641 remain pending across 1,108
case IDs.

The Java route matrix now adds generic `Operation` Item construction, view, and
decode evidence for every Operation obligation in the selected catalog cases.
This includes operation-bearing object and payload cases and ordered batch
steps with their workflow identity; local enum fixtures keep their primary
operation as context unless the catalog separately plans it as an Operation
target. The new events add 2,883 cells to the parent ledger. The full Java
matrix remains incomplete (33,978/71,641 executed), so the PR stack remains
WIP. These are local TTLV checks, not evidence of operation payload validity or
server behavior.

Verification for this increment:

```sh
bash bindings/java/gradlew -p interop/java-kmip-validator test --rerun-tasks
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

The transparent key-material cases now exercise the installed typed
`KeyValue.fromFields`, `KeyValue.fromItem`, `KeyBlock.fromFields`, and
`KeyBlock.fromItem` routes in addition to the two `KeyMaterial` constructors.
The 29 catalogued Big Integer members run in both KMIP versions, with the
nested member checked after each typed conversion. The selected Java report
validates 34,246/34,246 cells with zero failures; 37,395 of the full 71,641
cells remain pending, so the Java matrix and stack remain WIP.

The Transparent Symmetric Key `Key` Byte String also exercises all six typed
`KeyMaterial`, `KeyValue`, and `KeyBlock` constructors in KMIP 2.0/2.1 (§3.4).
The fixture bytes are deterministic synthetic test data. The selected report
now validates 34,258/34,258 cells with zero failures and leaves 37,383 pending.

The complete `Key Material` Structures for all nine transparent key formats
also pass all six typed `KeyMaterial`, `KeyValue`, and `KeyBlock` constructors
in both KMIP versions. The selected report now validates 34,366/34,366 cells
with zero failures and leaves 37,275 pending.

Each transparent Key Material case now also passes through the installed
`ManagedObject.fromFields` and `ManagedObject.fromItem` routes, wrapped in the
matching Symmetric Key, Private Key, or Public Key object structure according
to its format (§§2.5, 2.6, 2.9, 3.4–3.12). Both conversions read back the
nested Key Block and verify its format and catalogued material fields. Fixture
values are deterministic synthetic data; they do not test cryptographic
validity. The selected report validates 34,534/34,534 cells with zero failures
and leaves 37,107 of 71,641 planned Java cells pending, so the matrix and PR
stack remain WIP.

Each transparent Key Material case now also calls the matching typed object
factory (`ManagedObject.symmetricKey`, `privateKey`, or `publicKey`) and reads
the nested Key Block back. This closes 84 factory-route cells across the nine
formats and KMIP 2.0/2.1. The selected report validates 34,618/34,618 cells
with zero failures and leaves 37,023 of 71,641 pending; the matrix remains WIP.

The Split Key managed-object root now exercises the installed
`ManagedObject.splitKey`, `ManagedObject.fromFields`, and `ManagedObject.fromItem`
routes in KMIP 2.0 and 2.1. Its deterministic fixture uses the source-defined
Table 9 field order, XOR, two parts, threshold two, and a Raw Key Block; it
checks the typed accessors and object type. This adds six local model cells and
does not test cryptographic splitting or server behavior. The selected report
validates 34,702/34,702 cells with zero failures; 36,939 of 71,641 planned Java
cells remain pending, so the full matrix remains WIP.

The Split Key Table 9 member cases now exercise `ManagedObject.splitKey`,
`ManagedObject.fromFields`, and `ManagedObject.fromItem` for all six structure
members and the Split Key tag in KMIP 2.0 and 2.1. The Prime Field Size fixture
uses Polynomial Sharing Prime Field and Big Integer 257, as required by the
conditional field rule; other fixtures use XOR and omit that field. The
deterministic Raw Key Block is model data, not a test of key splitting. This
adds 38 local route cells. The selected report validates 34,740/34,740 cells
with zero failures; 36,901 of 71,641 planned Java cells remain pending, so the
full matrix and stack remain WIP.

The PGP Key and Secret Data structures now run through their named
`ManagedObject` factories and the applicable `fromFields`/`fromItem` routes in
KMIP 2.0 and 2.1 (§§2.4, 2.7). Fixtures preserve each structure's required
version/type field followed by a Raw Key Block. This adds 40 local model route
cells and does not test cryptographic key validity. The selected report
validates 34,780/34,780 cells with zero failures; 36,861 of 71,641 planned Java
cells remain pending, so the full matrix and stack remain WIP.

The Java generic TTLV runner also covers the Key Block structure-member targets
for PGP Key, Private Key, Public Key, Secret Data, and Split Key. These cases
are catalogued as `Object Data Structure`; the runner records Item construction,
view, and decode, and uses the route manifest to decide whether a target also
has a `TtlvValue.structure` obligation. This evidence is local generic TTLV
execution, not field-level conformance or server interoperability. The selected
report validates 34,816/34,816 cells with zero failures; 36,825 of 71,641 full
Java cells remain pending.

The six KMIP 2.0 version-boundary cases for Get Constraints, Ping, Process,
Query Asynchronous Requests, Set Constraints, and Set Defaults now execute the
assigned KMIP 2.1 Operation values through local generic Item construction,
view, and decode. No unsupported 2.0 request is sent. The selected report
validates 34,834/34,834 cells; 36,807 of 71,641 Java cells remain pending.

Batch workflow cases now exercise the generic TTLV construction, view, and
decode routes for each catalogued Request Batch Item and Response Batch Item
target in KMIP 2.0/2.1 (§§8.3, 8.6). These local fixtures cover only generic
representation routes, not workflow behavior. The full Java suite passes
92/92; the selected report validates 34,984/34,984 cells with zero failures,
and 36,657 of 71,641 full Java route cells remain pending.

Get Attributes now exercises both installed typed builder routes for every
catalogued Get Attributes case in the supported KMIP 2.0/2.1 test contexts.
Each request runs through the controlled mutual-TLS TTLV peer and validates
the returned attribute response. This adds 889 selected route cells: the
report validates 35,873/35,873 selected cells with zero failures, while
35,768 of the full 71,641-cell matrix remain pending. These operation-level
fixtures do not claim that every attribute has independently completed its
semantic construction policy.

## Java Get Attributes encoding and transport coverage — 2026-11-18

Both installed typed Get Attributes construction routes now execute each
catalogued case in KMIP 2.0 and 2.1 over TTLV, JSON, and XML with raw TLS and
HTTPS. Controlled mutual-TLS peers parse each request and return an encoding-
matched response; the validator checks the requested attribute references,
protocol version, batch identifiers, and typed result. The TLS connection
builder applies a DNS-name override only when configured and retains the HTTPS
request target path and query. These routes follow OASIS KMIP 2.0/2.1 §6.1.20
and §§8.3, 8.6, within the approved KMIPKit encoding and transport contracts.
This establishes route execution for the operation, not semantic coverage of
every possible attribute value or vendor interoperability.

The evidence generator now declares the exact encoding/transport combinations
observed in its report, and the shared validator rejects omitted combinations.
The selected report validates 39,743/39,743 cells with zero failures; the full
Java ledger records 71,641 planned, 39,743 executed, and 31,898 pending cells.
The Java suite passes 92/92 tests, the complete shared unit suite passes
196/196, and the ledger comparison and 1,507-route capability audit pass. The
full matrix remains open under KMIPKIT-0092-FR-002, FR-003, and FR-009, so #187
and its dependent stack remain WIP.
