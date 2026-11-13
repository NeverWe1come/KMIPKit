# Plan: KMIPKIT-0125 Pre-Release Hardening

## Architecture Decisions

1. **Opt-in `fuzzing` feature**: Codecs and framers are exposed to `fuzz/` through an opt-in `fuzzing` Cargo feature with `#[doc(hidden)]` visibility, preserving the public API surface.
2. **Tape Library §5.12.3 closure**: Closed via dedicated negative test showing non-injection of Alternative Name by the client, backed by decision `KMIPKIT-DEC-035`.
3. **Miri execution**: Safe crates (`kmipkit-ttlv` and `kmipkit-protocol`) are tested under Miri. Tests that allocate 16 MiB or 100,000 items are marked `#[cfg_attr(miri, ignore)]` with documented rationales since synthetic preflight checks prove bounds in Miri in milliseconds.
4. **Sanitizers**: Broaden existing CI sanitizer jobs to cover all C consumers and JNI C++ tests.
