# Security and robustness checks

`fuzz/` contains bounded libFuzzer targets and seed inputs for TTLV, extension schemas, JSON, XML framing, and Server-to-Client/version decoding. Run a short fuzzing campaign from the repository root with `python verification/run.py security-smoke`; the runner also includes the configured Miri, native sanitizer, and extended negative checks.

Install the Rust nightly toolchain and the pinned `cargo-fuzz` version listed in CI before running the campaign. Inputs are bounded to limit the work performed per iteration. A clean campaign is evidence for the exact targets, iterations, toolchain, and inputs recorded; it is not a proof that all malformed inputs are safe.

`openssl/` records the native OpenSSL LTS series, source archive checksum,
reusable build helper, and scheduled update tracker. See
[`docs/security/openssl.md`](../../docs/security/openssl.md) for consumer
build instructions and the maintainer update procedure.
