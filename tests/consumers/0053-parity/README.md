# KMIPKIT-0053 external consumers

These four small applications are independent package consumers. They use the
public Rust, C, Java, or Python packages respectively. The Rust consumer uses
the `kmipkit` facade and the `kmipkit-client` crate for its fluent policy type.
Each sends Ping and performs a Create/Get/Destroy lifecycle through its public
client API. The lifecycle uses a server-returned identifier and checks typed
responses. Java also checks pending and failure outcomes and an explicit
asynchronous header. The runners start a loopback-only TLS 1.3/mTLS fixture
responder using the repository's disposable test identities. It validates the
decoded requests and returns encoded KMIP responses; it is a client-behavior
fixture, not a KMIP server implementation or external interoperability
evidence.

Run all consumers on Windows with `./verify.ps1`, or on Linux/macOS with
`./verify.sh`. The scripts build the native library, compile each consumer
against its package surface, and run it. The Rust consumer is built with the
project MSRV toolchain, Rust 1.94. The Java consumer is resolved through
Maven's local package repository; the Python consumer uses a local editable
package source so its native extension is built from the same checkout.
