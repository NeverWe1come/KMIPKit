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
high-level API route. Asynchronous request types that only build payloads and
operations with no public client request route carry separate API limitation
reasons.

CI builds the pinned JNI JAR and runs the validator against a local TLS 1.3
mTLS peer with a controlled KMIP response. This verifies the external package
and request/response handling; it is not evidence for a commercial or other
deployed KMS.
