# Patched `aws-lc-sys` provenance

KMIPKit pins `aws-lc-rs` 1.18.1, which resolves `aws-lc-sys` 0.45.0. The
vendored copy at `vendor/aws-lc-sys-0.45.0` is the exact crates.io package
with one documented change to the embedded jitterentropy header. Its original
Cargo archive has SHA-256
`9bff6c3b54fad79a2e60b8102caf565819711497c1f5f092f49508e2f5c31b27`, matching
the pre-patch `Cargo.lock` checksum. The extracted crate identifies source
revision `7943223c99d909bc399bdf1b856821bb04f1f3c5`; its AWS-LC source metadata
identifies `02561621ffa4cf17c0c4f70bc11a82df36b42ae9`. The preserved upstream
`LICENSE` file has SHA-256
`728536b4160e051f86d7c9c388f704866b3d512cd7df97ac3516c65279523c4e`, and the
unmodified package `Cargo.toml` has SHA-256
`836bac6390070d88f3c804f4212e16ca6e142f6c111795094d925470d07d6866`. The
original crates.io archive includes its upstream `Cargo.lock`; the checked-in
vendor copy intentionally excludes that file and contains 2,028 files. The
repository's root and fuzz lockfiles remain the workspace resolution records
and are checked with `--locked`.

## Patch

In
`aws-lc/third_party/jitterentropy/jitterentropy-library/jitterentropy.h`,
`JENT_PRIVATE_STATIC` expands to an empty declaration attribute when
`BORINGSSL_IMPLEMENTATION` is set. The embedded AWS-LC CMake build sets that
definition privately on both the `jitterentropy` object target and the
`crypto_objects` target that calls jitterentropy. This keeps the functions
linkable between AWS-LC objects while avoiding explicit Windows
`__declspec(dllexport)` and preserving the hidden ELF/Mach-O visibility used by
the AWS-LC build. Standalone jitterentropy builds without this definition keep
the upstream behavior. The patch does not disable CPU jitter entropy or alter
its implementation.

The root Cargo workspace applies this exact package through
`[patch.crates-io]`; the dependency-policy validator permits only the
`aws-lc-sys@0.45.0` manifest at this exact repository path. The isolated fuzz
workspace does not resolve `aws-lc-sys`; its lockfile update records only the
current versions of its two local protocol crates. The package's
upstream `LICENSE` and third-party notices remain present in the vendor tree.
No Cargo registry files are modified.

## Verification

The native ABI audit uses the already-built library in the `language-c` CI
matrix on Windows, Linux, and macOS:

```text
python -B tools/api_manifest/audit_binding_exports.py --repo-root . --c-only --c-binary <built-library> --check
```

It compares named native exports with the C header and canonical manifest and
fails if an export lacks the `kmipkit_` prefix, including any jitterentropy
symbol. The full C/Java/Python audit remains available with
`--report ... --check`.
