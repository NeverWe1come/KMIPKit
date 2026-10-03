# KMIPKit project definition

Status: approved design baseline
Date: 2026-10-03
Owner: NeverWe1come

## 1. Purpose

KMIPKit will be an open source client library for OASIS Key Management
Interoperability Protocol 2.1. The project has two goals:

1. Build a rigorous, traceable understanding of KMIP 2.1.
2. Provide a production quality, multi-language client that makes KMIP usable
   without requiring every application developer to understand the wire
   protocol.

The differentiator is the combination of complete KMIP 2.1 client coverage,
a Rust core, a stable C ABI, idiomatic language APIs, high level operations,
and evidence based conformance.

## 2. Release scope

### 2.1 Version 1.0.0

Version 1.0.0 includes:

- KMIP 2.1 only.
- TTLV encoding only.
- Every operation initiated by a KMIP client.
- KMIP protocol asynchronous semantics, including pending results and their
  follow-up operations.
- Batch messages and a high level batch API.
- Raw binary TTLV over TLS.
- Binary TTLV over HTTPS with HTTP/1.1.
- TLS 1.3 and mutual TLS.
- Synchronous/blocking client APIs.
- Rust, C, Java, and Python with functional parity.
- High level, typed protocol, and generic TTLV APIs.
- Every KMIP message credential type in scope.
- Generic vendor extension representation and discovery.

Version 1.0.0 excludes:

- Server initiated operations.
- JSON and XML encodings.
- Rust async/await APIs.
- Automatic retries, failover, connection pools, proxy support, and
  capability caching.
- Local cryptographic algorithm implementations.
- Dynamic executable plugins.
- PKCS#11, HSM, operating system, and language runtime TLS key providers.

### 2.2 Later releases

- 1.1.0: server initiated operations.
- Additional idiomatic adapters, prioritized as Go, C++, C#, and JavaScript.
- Later compatible 1.x releases: JSON, XML, optional async APIs, additional
  transports and operational facilities.
- A new major version is reserved for public API or ABI incompatibility.

## 3. Completeness

Completeness means every applicable KMIP 2.1 client operation, object,
attribute, message structure, option, and value can be represented and used.
Server policy determines whether an individual request is accepted.

The library does not preemptively reject legitimate standard options because
a particular server might not support them. It validates protocol invariants
locally and returns detailed KMIP failures from the server.

The project distinguishes protocol coverage from profile conformance. A
profile is claimed only after its applicable normative clauses and tests pass.
No certification claim is made without a formal external process.

## 4. Architecture summary

- Rust core.
- Stable C ABI as the common foreign-language boundary.
- Idiomatic Java JNI and Python CFFI adapters in 1.0.
- Cargo workspace split into TTLV, protocol, transport, client, facade, FFI,
  and test support crates.
- All unsafe code restricted to the FFI crate.
- Declarative normative catalog and public API manifest drive generated code.
- Generated output is reviewed and committed.

See [the architecture overview](../architecture/overview.md).

## 5. API levels

1. High level operation builders covering the entire 1.0 operation scope.
2. Complete typed KMIP request and response models.
3. Structurally valid generic TTLV for extensions and advanced use.

High level APIs fill mechanical protocol fields. Security decisions such as
algorithm, key length, usage, mode, padding, and export policy remain explicit.

## 6. Synchronous behavior and concurrency

- The Rust client is synchronous.
- KMIP protocol asynchronous results are fully supported.
- A client is thread safe and owns one reusable connection.
- Concurrent calls on one client are serialized.
- Applications create multiple clients for parallel network operations.
- No request is retried automatically.
- A failed connection is discarded; a later request may open a new one.

## 7. Security posture

- TLS 1.3 only through rustls and aws-lc-rs.
- Mutual TLS and mandatory server certificate validation.
- 0-RTT, redirects, HTTP compression, proxies, TLS key logging, and automatic
  network revocation lookup are disabled.
- Explicit CRL input is supported.
- Secret types minimize copies and zeroize owned memory.
- Logs and errors redact credentials, keys, raw bodies, and sensitive data.
- Decoder resource limits are enforced before allocation.
- No cryptographic primitives are implemented by KMIPKit apart from using
  established dependencies for TLS.

KMIPKit may transport attestation, CSR, ticket, OTP, PKCS#11, and similar
artifacts. Their production belongs to the appropriate external subsystem.

## 8. Standards and AI context

Exact upstream HTML copies of the OASIS KMIP 2.1 Specification, Profiles,
Usage Guide, and Test Cases are pinned under `specification/oasis/` with URLs
and SHA-256 hashes. They are immutable inputs.

The project will create a reviewed machine-readable normative catalog. Builds
will never scrape OASIS pages. Every normative implementation requirement is
linked to source, code, and tests.

## 9. Development methodology

- Spec Driven Development using Spec Kit.
- Strict Red, Green, Refactor TDD.
- One approved specification, feature branch, worktree, and responsible agent.
- One foundation agent until shared types, TTLV, error model, transport
  boundaries, client skeleton, minimal C ABI, CI, and conformance harness are
  stable.
- Independent agents may work in parallel only after the foundation is stable
  and their specifications do not share mutable boundaries.
- Agents open draft PRs. Humans review and merge.

Spec Kit is installed in this repository; the maintainer owns its framework
files.

## 10. Quality targets

- Rust 2024, MSRV 1.94.
- Rustfmt and Clippy with warnings denied.
- No panic based production control flow.
- Public APIs documented.
- Normative traceability: 100 percent.
- TTLV and protocol line coverage: at least 95 percent.
- Transport, FFI, and bindings: at least 85 percent.
- Workspace overall: at least 90 percent.
- Changed code: at least 95 percent.
- Two independent real KMIP implementations tested before 1.0.
- Independent security review before 1.0.

## 11. Supported systems

- Linux glibc `x86_64` and `aarch64`, compatible with manylinux 2.28.
- Windows x86_64 MSVC, Windows 10 or Server 2019 and newer.
- macOS x86_64 and aarch64, macOS 12 and newer.
- Java 17 and newer.
- Python 3.12 and newer.

Musl, Alpine, Windows ARM64, and additional platforms are later work.

## 12. Distribution

- Synchronized semantic versions across official packages.
- Rust crates on crates.io.
- Java artifacts and platform native JARs on Maven Central.
- Python binary wheels and source distribution on PyPI.
- C dynamic and static libraries with CMake and pkg-config metadata.
- Signed GitHub Releases with hashes, SBOM, provenance, and release notes.
- No runtime binary downloads.

## 13. Licensing and governance

- Apache-2.0.
- DCO sign-off; no CLA initially.
- Private repository until 1.0.0.
- Human maintainer owns approval and merging.
- External OASIS material retains its notices and is not relicensed.
- KMIPKit is independent and does not claim OASIS endorsement.

## 14. Documentation

- Technical sources, code, specs, ADRs, rustdoc, and API naming are English.
- Development discussion may occur in Spanish.
- The user guide begins in Spanish and is complete in both English and Spanish
  before 1.0; English is canonical from 1.0.
- Examples are compiled or executed in CI.
