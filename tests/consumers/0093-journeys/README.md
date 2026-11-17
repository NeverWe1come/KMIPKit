# KMIPKIT-0093 fluent API journey consumers

These four small applications are independent package consumers. Their only
KMIPKit dependency is the public Rust, C, Java, or Python package respectively.
Each completes five progressive journeys against one loopback-only TLS
1.3/mTLS fixture responder using the repository's test-only identities:
Create → Get → Destroy, explicit AES-GCM, ECDSA key-pair signing, a catalogued
expert attribute, and a two-item Create → Get batch using ID Placeholder.
Rust, Java, and Python use their fluent request APIs; C uses the semantic
builder and typed batch-item APIs. The fixture decodes every request in order,
validates the batch and placeholder fields, and sends encoded KMIP responses.
It is a client-behavior fixture, not a KMIP server implementation or
interoperability evidence.

Run all consumers on Windows with `./verify.ps1`, or on Linux/macOS with
`./verify.sh`. The scripts build the native library, compile each consumer
against its package surface, and run it. The Rust consumer is built with the
project MSRV toolchain, Rust 1.94. The Java consumer is resolved through
Maven's local package repository; the Python consumer uses a local editable
package source so its native extension is built from the same checkout.

The fixture expects 52 TLS connections: 12 single-item exchanges and one
two-item batch for each consumer. In that batch, Create omits Unique Identifier
and Get sends the KMIP 2.1 ID Placeholder Enumeration (1), with Batch Order
Option true. The fixture assigns the identifier in its Create response and
returns the same identifier from Get. Each batch item carries a distinct Unique
Batch Item ID, and the response echoes each one in order. No consumer guesses
or inserts the server-assigned identifier into the Get request.

See the [English guide](../../../docs/user-guide/en/fluent-api-journeys.md) or
the [Spanish guide](../../../docs/user-guide/es/recorridos-api-fluida.md) for
the request semantics and source paths for each language.
