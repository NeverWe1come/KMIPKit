# Fuzz targets

KMIPKit keeps bounded coverage-guided fuzz targets in this workspace.

## Extension schema validation

`extension_schema` decodes at most 4 KiB of untrusted TTLV using limits of 64
nested Structures and 512 Items. If the root is a Structure, the target runs
it through a fixed nested data-only schema with discriminator, scalar range,
enumeration, repeated-field, and order constraints. Schema mismatches are
expected; panics and limit bypasses are not.

Run with the installed nightly toolchain and `cargo-fuzz`:

```text
cargo +nightly fuzz run extension_schema -- -max_len=4096 -timeout=5
```

Use `-runs=1000` for a short smoke campaign. Minimized crashes are written to
`fuzz/artifacts/extension_schema/`. Promote each reproducible crash to a
deterministic protocol regression test before making a fix.

The executable OASIS-derived extension vectors cite KMIP Specification v2.1
§§7.13/Table 365, 8.3/Table 396, 9.13/Table 418, 11.44/Table 476, and §11.56
tag allocations in `tests/fixtures/extensions/oasis/` and
`crates/kmipkit-protocol/tests/extension_oasis_vectors.rs`.
