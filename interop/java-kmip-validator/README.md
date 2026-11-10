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
validator also constructs and decodes generic TTLV values locally. It does not
send arbitrary generic TTLV requests, and the Java package has no distinct
high-level API route. The KMIP 2.0 live route remains disabled while
protocol-version preservation is outside the pinned release artifact.

CI builds the pinned JNI JAR and runs the validator against a local TLS 1.3
mTLS peer with a controlled KMIP response. This verifies the external package
and request/response handling; it is not evidence for a commercial or other
deployed KMS.
