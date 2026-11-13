# KMIPKit Java interoperability validator

This project is an external Java 17 consumer of the installed KMIPKit JNI JAR.
It does not compile the library sources from this checkout. The artifact pin is
in `library-pin.json`; the installer extracts that exact `release/1.0.0`
commit, builds the platform JAR in an isolated source tree, installs it into a
local Maven repository, and writes a SHA-256 provenance manifest.

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
baseline. These recipes execute 13,604 of 71,455 planned Java route cells, with
no failed cells; the ledger keeps the other 57,851 pending and does not imply
vendor interoperability or full-matrix completion. Local TTLV recipes exercise 41
enumeration families and all six bitmask catalog cases through the applicable
installed constructors, item views, decoders, and typed attribute wrappers.
All 19 Block Cipher Mode values now complete their five applicable generic and
Cryptographic Parameters builder routes in both KMIP 2.0 and 2.1 (§§4.16,
11.6); other enum families remain pending where the ledger says so.
The finite Profile Name reserved range
`test-norm-enumeration-value-profile-name-reserved-00000001-00000103` remains
pending until the shared catalog expands it to each named Profile Name value.
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

The recorded [development evidence](../../specs/092-interop-api-route-exhaustiveness/java-route-expansion-evidence.md)
keeps the Red/Green/Refactor commands and the distinction between this completed
local scope and the pending complete matrix.
