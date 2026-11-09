# KMIPKIT-0051 synchronization evidence (2026-11-09)

## Stacked predecessor update

KMIPKIT-0047's exact KMIP 3.0 tag correction was committed as Red
`5f5bab807`, Green `752c51aa6`, and Refactor `f3b2199f8`. The Red test
demonstrated that the previous `0x420153..=0x420172` gate incorrectly treated
KMIP 2.1 tags as KMIP 3.0. The correction uses the exact assignments from
KMIP 3.0 §11.58, Table 605 and retains KMIP 2.1 tags such as `Object Defaults`
and `Protection Storage Masks`.

That update was propagated with ordinary merges through KMIPKIT-0048, -0049,
-0050, and -0051. The KMIPKIT-0051 synchronization merge is
`bfbc54bb7`. The final protocol test fixture for the KMIP 3.0-only
`NIST Security Category` tag is `0x4201C2`; `0x420153` is the KMIP 2.1
`Object Defaults` tag. The provenance-order regression now checks the extracted
request-build-and-version-gate helper directly.

## Local verification on the synchronized 0051 tree

- `cargo test -p kmipkit-client --lib`: 747 passed.
- `cargo test -p kmipkit-protocol --lib`: 1,430 passed.
- `cargo clippy -p kmipkit-protocol -p kmipkit-client --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --all --check`: passed after formatting the updated source-order test.
- `git diff --check`: passed.
- `python -B tools/api_manifest/generate.py --check`: all 12 outputs verified.
- `python -B tools/normative_catalog/generate_ttlv_tags.py --check`: generated tag allocations verified.

The Rust export audit and shared Rust inventory still have a repository-wide
gap of 4,199 routes missing from the manifest. Those routes are not attributed
to 0051; their owner-by-owner reconciliation remains part of the broader
parity work. The binding export report records exact matches for C (1,733),
Java (1,906), and Python (2,025); its `--check` passed. The Rust audit
`--check` and shared inventory `--check` remain blocked by the 4,199 global
manifest gaps.

Coverage was not run under the maintainer's instruction to suspend coverage
thresholds while the in-progress branches are being completed. This evidence
does not claim hosted CI success. PR #143 remains Draft until all applicable
repository gates and required CI checks pass.
