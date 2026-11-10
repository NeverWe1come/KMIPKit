# KMIPKit Interoperability Validators

This directory contains independent Rust, C, Java, and Python consumers for
observing KMIPKit behavior against KMIP 2.0 and 2.1 servers. A profile is a
planning hint only; it is never evidence that a server supports a capability.
Runs probe every version-applicable case and record the exact server and
configuration. These suites do not make certification or blanket profile claims.

## Projects

- `shared/`: schemas, versioned test catalog, vectors, profiles, and tools.
- `rust-kmip-validator/`: independent Rust consumer.
- `c-kmip-validator/`: independent C ABI consumer.
- `java-kmip-validator/`: independent Java consumer.
- `python-kmip-validator/`: independent Python consumer.

## Offline contract checks

```bash
python3 interop/shared/catalog/generate_test_catalog.py --check
python3 interop/shared/vectors/generate_vectors.py --check
python3 -B -m unittest discover -s interop/shared/tests -p 'test_*.py' -v
```

The catalog targets KMIP 2.0 and 2.1 and records each case as planned,
implemented, or not applicable. A planned case is not a test result. The
contract also records that the current public API has no generic-TTLV outbound
request path; a test of that API level must not be reported as server
incompatibility.

## Preparing a live run

Copy `shared/profiles/connection.example.json` and set the protocol version,
endpoint, transport, encoding, and API level. TLS certificate and key paths
must point to files readable by the test process. KMIP username and password
are read from the named environment variables, never from the JSON file.
Keep `environment.kind` set to `dedicated_test` and retain its explicit
confirmation. Mutating cases remain disabled until
`execution_policy.allow_mutations` is set to `true`; the validators must reject
live mutation runs without both safety conditions. Do not target a shared or
production key manager.

Build artifacts from one exact `release/1.0.0` commit and install each
language's artifact into an isolated consumer environment. Keep the commit and
artifact SHA-256 with every result. Compare language results only when server,
protocol version, configuration fingerprint, catalog, and library artifact
match. Compare normalized KMIP semantics across API levels, transports, and
encodings; byte equality applies only to deterministic requests using the same
encoding.

Server results remain evidence for the exact product version, server
configuration, protocol version, transport, encoding, and executed case. A
failure does not by itself establish whether the problem is KMIPKit, the
server, or a local policy; reports retain those dimensions separately.
