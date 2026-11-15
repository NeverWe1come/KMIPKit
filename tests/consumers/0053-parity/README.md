# KMIPKIT-0053 external consumers

These four small applications are independent package consumers. They use the
public Rust, C, Java, or Python packages respectively. The Rust consumer uses
the `kmipkit` facade and the `kmipkit-client` crate for its fluent policy type.
Rust, C, and Python send Ping; all four perform Create/Get/Encrypt/Decrypt/
Destroy through their public client API. The flow uses the server-returned
identifier, explicit AES-GCM parameters, nonce, and additional authenticated
data, then feeds the fixture's Encrypt response fields into Decrypt. Each
consumer checks the typed responses. Java also checks pending and failure
outcomes and an explicit asynchronous header. The runners start a loopback-only
TLS 1.3/mTLS fixture responder using the repository's disposable test
identities. It validates decoded requests and returns encoded, predetermined
KMIP responses. The fixture does not perform AES; this checks client
construction and response handling, not cryptography or external KMIP
interoperability. These consumers cover the symmetric-key, AES-GCM,
key-pair/signing, and expert Vendor Attribute journeys. The batch-placeholder
journey remains in the KMIPKIT-0093 closeout work.

Run all consumers on Windows with `./verify.ps1`, or on Linux/macOS with
`./verify.sh`. The scripts build the native library, compile each consumer
against its package surface, and run it. The Rust consumer is built with the
project MSRV toolchain, Rust 1.94. The Java consumer is resolved through
Maven's local package repository; the Python consumer uses a local editable
package source so its native extension is built from the same checkout.
