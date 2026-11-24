# KMIPKit Java interoperability validator

This project is an external Java 17 consumer of the installed KMIPKit JNI JAR.
The installer archives the checked-out Git commit, builds the platform JAR in
an isolated source tree, and installs it into a local Maven repository. It
records the source commit and the artifact's SHA-256 alongside the generated
`library-pin.json`. Configure OpenSSL 3.5 before building as described in the
[build guide](../../../docs/user-guide/en/language-packages.md#building-kmipkit-from-source).

Run commands from this directory. `plan` is offline, `preflight` checks local
files and provenance without DNS or sockets, and `run` executes enabled test
routes against the endpoint in the connection file:

```sh
python3 tools/install_artifact.py \
  --repository ../../.. \
  --maven-repository build/maven-repository \
  --manifest build/artifact-manifest.json

bash ../../../bindings/java/gradlew \
  -PkmipkitMavenRepo="$PWD/build/maven-repository" test

bash ../../../bindings/java/gradlew \
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

The current live executor sends typed requests for `Discover Versions`,
`Ping`, `Query`, and `Create`. The Create recipe generates a unique named
AES-256 Symmetric Key, verifies the typed response, and sends a separate
`Destroy` exchange using the returned Unique Identifier. The CLI only sends
this recipe when both mutation opt-ins are set. The controlled TLS test covers
both exchanges. The installed Java API can also build and decode `Log`, and a
controlled TLS test covers its request and response. The catalog currently
classifies `Log` as read-only even though KMIP 2.1 §6.1.29 writes the message
to the server log, so the live CLI keeps it pending until the shared contract
requires mutation opt-in. Although the installed Java API also provides
`Get Constraints` and `Get Attribute List` request constructors, their catalog
cases require complete object setup, response assertions, and cleanup; they
remain pending until those recipes are implemented. The route inventory checks
every catalog operation against the installed Java API.
Other public typed request constructors are reported as pending until their
case has a complete fixture, request recipe, response assertion, and cleanup.
Asynchronous request types that only build payloads and operations with no
public client request route carry separate API limitation reasons. The
validator also constructs and decodes generic TTLV values. Use
`api_level: generic-ttlv` with `raw_tls` and `TTLV` to run Locate, Query,
Discover Versions, RNG Retrieve, and single-part SHA-256 Hash in KMIP 2.0
and 2.1, plus Ping in 2.1. These probes use a CA, a client certificate chain,
and an unencrypted PKCS#8 PEM client key. RNG Retrieve follows the catalog's
mutation opt-in.

With mutation opt-in, the direct tag route also runs Create, Get, Get Attributes, Destroy,
Archive and Recover. Each case creates an isolated AES-256 key and uses the
server-returned Text String identifier for all follow-up exchanges. The
validator checks the response identifier and performs cleanup with Destroy.

Archive and Recover use the isolated key lifecycle through the typed and
Fluent routes.

CI builds the JNI JAR from the revision under test and runs the validator against a local TLS 1.3
mTLS peer with a controlled KMIP response. This verifies the external package
and request/response handling; it is not evidence for a commercial or other
deployed KMS.

## Local KMIP 3.0 draft campaign

After installing the artifact, build the standalone distribution.

```sh
bash bindings/java/gradlew --project-dir verification/interop/java-kmip-validator installDist
python3 verification/interop/shared/tools/campaign.py --version 3.0 \
  --runner java verification/interop/java-kmip-validator/tools/run_draft_campaign.sh \
  --output-dir /tmp/kmip-java-draft
```

The runner verifies that the loaded JAR matches the installer manifest, calls
all seven public generic-payload constructors through JNI, and checks their
operation codes. It generates local evidence. The existing live runner
handles KMIP 2.0 and 2.1.

The same local runner accepts `--version 2.0 --version 2.1` to check the
numeric values and allocation of concrete generic TTLV tags from the
shared matrix.

For one live typed KMIP 2.x Query Operations exchange, reuse the Java
runner with `--scope live --connection PATH`. The report retains the numeric
KMIP result and the outcome of the installed typed response checks.

To select the common source revision for a cross-language run, pass a
checkout of the exact commit to `tools/install_artifact.py --repository`.
Use the JAR produced by that installer to build the standalone distribution.
