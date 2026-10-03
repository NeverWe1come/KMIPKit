# Testing strategy

## Goals

Tests demonstrate protocol correctness, parser robustness, cross-language
parity, ABI stability, interoperability, and safe failure. Coverage is a gate,
not a substitute for behavior and requirement traceability.

## Test layers

### Unit tests

Small tests cover individual types, validation rules, encodings, state
transitions, and errors. They are independent and descriptive.

### TTLV conformance

- Official OASIS vectors are immutable fixtures.
- Project vectors fill identified gaps.
- Every primitive type includes boundaries and invalid forms.
- Canonical encode and strict decode are tested separately.
- Generic valid decode/encode roundtrips produce identical bytes.

### Negative parser tests

Include truncated headers and values, integer overflow, invalid lengths,
invalid type lengths, invalid padding, excessive depth, excessive count,
unknown values, malformed structures, trailing/incomplete frames, and limits.

### Property tests

Generate bounded valid TTLV values and assert encode/decode properties. Use
shrinking to retain minimal failures. Property tests begin in the foundation;
coverage-guided fuzzing follows later.

### Client tests

A deterministic fake transport verifies exact sent bytes, response handling,
connection invalidation, delivery state, timeouts, batch outcomes, pending
operations, and redaction without network I/O.

### Transport tests

An ephemeral PKI creates CA, server, and client material. Test valid mTLS,
unknown CA, hostname mismatch, expiry, CRL, incomplete I/O, timeout, HTTP
framing, forbidden redirect/compression behavior, and connection reuse.

### FFI and adapters

- Compile and execute a C consumer.
- Run panic-containment and memory ownership tests.
- Run sanitizer jobs for the FFI surface.
- Test Java from the packaged native JAR.
- Test Python from the built wheel in a clean environment.
- Run shared behavioral vectors through Rust, C, Java, and Python.

### Interoperability

Before 1.0, test against at least two independent server implementations. One
open source implementation runs in CI. A second may use a licensed or manual
environment. Record server product, version, transport, profile, operation,
result, and known workaround.

## Coverage

- TTLV/protocol: 95 percent line minimum.
- Transport/FFI/bindings: 85 percent line minimum.
- Workspace: 90 percent line minimum.
- Changed code: 95 percent line minimum.
- Normative requirement traceability: 100 percent.

Use `cargo llvm-cov`. Report branch coverage initially and promote it to a gate
after tool reliability is proven. Exclusions require a documented reason.

## CI levels

### Every PR

- Formatting and linting.
- MSRV and stable Rust tests.
- Linux, Windows, and macOS tests.
- Coverage and traceability.
- Documentation with warnings denied.
- Affected ABI and adapter tests.
- Dependency, advisory, source, and license policy.

### Scheduled

- Wide feature combinations.
- Miri on compatible safe crates.
- FFI sanitizers.
- Bounded fuzzing.
- External server integration.
- Updated vulnerability review.

### Release

- Complete target matrix.
- Build, install, and smoke test final packages.
- ABI and public API comparisons.
- Signatures, hashes, SBOM, and provenance.
- Independent security review findings resolved or explicitly accepted.

## Fuzzing

Initial targets include TTLV frame parsing, tree decoding, typed conversion,
unknown extensions, and response correlation. Seed corpora use public OASIS and
project fixtures with secrets removed.

## Performance

Benchmarks are added before 1.0 after correctness stabilizes. Measure codec
throughput and latency, allocations, batch behavior, FFI overhead, and small,
medium, large, and configured maximum messages. Establish targets from real
measurements rather than unsupported claims.
