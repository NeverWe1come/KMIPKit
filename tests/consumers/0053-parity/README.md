# KMIPKIT-0053 external consumers

These four small applications are independent package consumers. Their only
KMIPKit dependency is the public Rust, C, Java, or Python package respectively.
Each one creates a Ping request, checks its KMIP operation value, and navigates
an owned generic TTLV structure. They make no network request.

Run all consumers on Windows with `./verify.ps1`, or on Linux/macOS with
`./verify.sh`. The scripts build the native library, compile each consumer
against its package surface, and run it. The Rust consumer is built with the
project MSRV toolchain, Rust 1.94. The Java consumer is resolved through
Maven's local package repository; the Python consumer uses a local editable
package source so its native extension is built from the same checkout.
