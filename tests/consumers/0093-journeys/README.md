# KMIPKIT-0093 fluent lifecycle journey consumers

These four small applications are independent package consumers. Their only
KMIPKit dependency is the public Rust, C, Java, or Python package respectively.
Each completes Create → Get → Destroy against one loopback-only TLS 1.3/mTLS
fixture responder using the repository's test-only identities. Rust, Java, and
Python use their fluent request APIs; C uses the direct semantic functions for
these common tasks. The fixture decodes each received request in order and
sends encoded KMIP responses. It is a client-behavior fixture, not a KMIP
server implementation or interoperability evidence.

Run all consumers on Windows with `./verify.ps1`, or on Linux/macOS with
`./verify.sh`. The scripts build the native library, compile each consumer
against its package surface, and run it. The Rust consumer is built with the
project MSRV toolchain, Rust 1.94. The Java consumer is resolved through
Maven's local package repository; the Python consumer uses a local editable
package source so its native extension is built from the same checkout.
