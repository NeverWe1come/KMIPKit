# Java route PR: native consumer linkage evidence

This records the native test-harness repair for PR #187 under
KMIPKIT-0092-FR-002 and FR-006. It does not alter the approved route matrix,
fixtures, public library API, or coverage thresholds.

## Red

Baseline: `9bb6ceb49bf6bc16325a38feb05b1d377234c901`.

```sh
cargo +1.100.0 test --locked -p kmipkit-ffi --all-features \
  --test operation_parity_0053 --test c_api_coverage
```

The existing executable C consumer regression targets fail to link with
`unable to find library -lkmipkit_ffi` (exit 101). Rust 1.99.1 passes the
same command; Rust 1.100.0 matches the failing current stable CI toolchain.
No additional test is needed to reproduce this build-boundary regression.

Exact-head CI run `43300133687` also records:

- Core Ubuntu stable and Coverage Ubuntu: missing `-lkmipkit_ffi`.
- Core Windows 1.94: LNK2005/LNK1169 because `kmipkit_ffi.lib` and
  `libkmipkit_ffi.rlib` both define the exported ABI.
- Coverage Windows: LNK1181, missing `kmipkit_ffi.lib`.

The explicit native `#[link(name = "kmipkit_ffi")]` introduces a second ABI
library alongside Cargo's Rust dependency. Its build-script search path also
assumes the pre-1.100 output directory layout. The regression targets must
exercise their compiled C consumers against exactly one in-process ABI copy.

## Green

The explicit native ABI library links are removed. Both integration targets
now depend on Cargo's `kmipkit_ffi` rlib; the operation target uses the public
opaque batch type in its foreign declarations, matching the C header.

The same Rust 1.100.0 command passes 50 C ABI tests and one operation-parity
test (exit 0), including compiled C calls and mutual-TLS fixture exchanges.
