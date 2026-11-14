# KMIPKit Java interoperability validator

This project is an external Java 17 consumer of the installed KMIPKit JNI JAR.
It does not compile the library sources from this checkout. The artifact pin is
in `library-pin.json`; the installer extracts that exact `release/1.0.0`
commit, builds the platform JAR in an isolated source tree, installs it into a
local Maven repository, and writes a SHA-256 provenance manifest. The current
pin is `489fafc79ef89db2524eafa7a6c6482f5cf75d97`.

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
baseline. These recipes execute 22,689 of 71,463 planned Java route cells, with
no failed cells; the ledger records the other 48,774 as pending and does not imply
vendor interoperability or full-matrix completion. Local TTLV recipes exercise 51
enumeration families, all six bitmask aggregate cases, and all 44 individual
bitmask value cases through the applicable
installed constructors, item views, decoders, and typed attribute wrappers.
They also cover the optional and repeatable Object Group member in KMIP 2.1
§7.23 with all three applicable generic Item routes; §4.35 Tables 99–100
define its Text String attribute representation.
The outer KMIP 2.1 Object Groups and Object Types Structures exercise the six
generic Structure/Item routes each, with exact repeatable Text String and
Enumeration members (§§7.23–7.24, Tables 375–376).
Data Length, Final Indicator, Interop Identifier, Log Message, and MAC Data
exercise their six applicable local generic routes in KMIP 2.0 and 2.1 with
their source-defined Integer, Boolean, Text String, or Byte String values
(KMIP 2.0 §§7.6, 7.10, 7.12, 7.15–7.16; KMIP 2.1 §§7.10, 7.14, 7.16,
7.19–7.20; tag allocations §§11.54/11.56).
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

The 48,774 pending cells belong to 1,306 case IDs listed in the generated
ledger; every one has the reason `route_evidence_not_emitted`. The applicability
rule is the shared catalog and capability matrix after protocol-version,
public-route, encoding/transport, and source-backed exclusion filters. A cell
remaining in that matrix is planned and applicable; the current ledger gives
none of these cells a non-applicability or API-unavailable reason. KMIPKIT-0092
FR-002 and FR-009 therefore treat them as unfinished coverage, not accepted
exceptions. This CI recipe validates only its declared `case_ids` selected
scope and the ledger comparer checks stable ledger equality; a green selected
scope does not mean the full-matrix completeness gate is satisfied. The PR
remains WIP while any required Java route cells are pending.
All 19 Block Cipher Mode values now complete their five applicable generic and
Cryptographic Parameters builder routes in both KMIP 2.0 and 2.1 (§§4.16,
11.6); other enum families remain pending where the ledger says so. The three
Rotate Name Type values complete their four applicable generic TTLV routes in
KMIP 2.1 (§11.51). NIST Key Type completes four generic and three typed routes
for all 20 values in KMIP 2.0/2.1 (§§4.34, 11.32 in both; tag allocation
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
