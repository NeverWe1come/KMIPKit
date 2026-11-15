# Research: Rustls Advisory Remediation

## Findings

- The active release branch pins `rustls = "=0.23.45"` in the workspace manifest. The transport crate inherits that dependency; the fuzz workspace does not depend on Rustls.
- The RustSec advisory database published four advisories on 2026-10-14 for affected Rustls 0.23.x releases: RUSTSEC-2026-0346 through RUSTSEC-2026-0349. Each lists `>= 0.23.46` as patched.
- Advisory 0346 concerns `Debug` output for client session values exposing TLS secrets when custom session stores format those values. Advisory 0347 concerns application data buffered before a rejected ECH handshake. Advisory 0348 concerns a large allocation through the optional Brotli certificate decompressor. Advisory 0349 concerns an unauthenticated ServerHello affecting key-exchange prediction and possible loss of post-quantum confidentiality.
- `cargo info rustls@0.23.46` confirmed that stable Rustls 0.23.46 is published and declares Rust 1.71 as its minimum; this is compatible with the repository's Rust 1.94 MSRV.
- The approved dependency-policy spec requires online advisory refresh for both workspaces and forbids cached-only success. No exception is appropriate when a patched compatible release exists.

## Decisions

1. Pin the workspace to exact Rustls 0.23.46, the earliest stable patch release covering all four advisories at the time this spec was written.
2. Preserve existing `default-features = false` and `aws_lc_rs`/`std` feature configuration. Do not enable Brotli, ECH, or other features as part of this correction.
3. Change only the root Rustls dependency pin and its lockfile resolution, plus regression contract, traceability, and evidence documentation.
4. Do not edit the dependency-policy tool, exception register, runner, or CI policy; they correctly surfaced the new advisories.

## Sources

- [RUSTSEC-2026-0346](https://raw.githubusercontent.com/RustSec/advisory-db/main/crates/rustls/RUSTSEC-2026-0346.md)
- [RUSTSEC-2026-0347](https://raw.githubusercontent.com/RustSec/advisory-db/main/crates/rustls/RUSTSEC-2026-0347.md)
- [RUSTSEC-2026-0348](https://raw.githubusercontent.com/RustSec/advisory-db/main/crates/rustls/RUSTSEC-2026-0348.md)
- [RUSTSEC-2026-0349](https://raw.githubusercontent.com/RustSec/advisory-db/main/crates/rustls/RUSTSEC-2026-0349.md)
