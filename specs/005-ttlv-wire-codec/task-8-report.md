# T008 Test Report (provisional)

## Scope and status

This report records only KMIPKIT-0005 T008. The test change is in `crates/kmipkit-client/src/wire_encoder.rs` under its private unit-test module. It adds no production code, API, manifest or lockfile changes, client integration, configurable decoder limits, or new dependency. T008 remains unchecked in `tasks.md` pending independent review.

DCO-signed test commit: `06fcdf8f193e3608f51bfa99541ff316d2cefa10` (`test(client): add deterministic TTLV round trips`).

## Generated round-trip coverage

A local xorshift generator uses fixed seed `0x6d69_706b_6974_0005` and creates 88 roots, eight for each of the eleven represented Item Types. It generates nested Structures to at most four Structure levels, with two to four children per Structure. Sibling leaf types differ so child reordering is observable even though the bounded fixtures reuse an assigned Tag. Big Integer and Byte String payloads are at most 12 octets; Text String payloads are bounded to 12 UTF-8 octets. No large fixture or property-testing dependency is used.

Each source model is encoded by the existing private writer and decoded through `kmipkit_ttlv::codec::decode`. The expected model is built recursively from the source model. Unaligned Big Integer octets receive the minimum leading `0x00` or `0xff` sign extension needed to reach a multiple of eight; already aligned octets are copied exactly. The recursive comparison checks every Tag, Item Type, scalar/payload value, Structure child count, and child order. Assertions do not format payload bytes.

A dedicated Big Integer test round-trips an unaligned positive value, an unaligned negative value, and an already aligned value whose octets must remain unchanged. Inline test comments identify OASIS KMIP Specification v2.1 §§10.1.1–10.1.5 and §11.23, stable NR/catalog requirement IDs, and KMIPKit FR-002/FR-003 where project policy or generic child-order behavior is tested. The comments do not claim schema-defined Structure ordering or an OASIS empty Big Integer prohibition.

## Verification evidence

The following commands completed with exit code 0 after the final source changes:

- `cargo fmt --all --check`
- `cargo clippy -p kmipkit-client --all-targets --all-features -- -D warnings`
- `cargo check -p kmipkit-client --all-features`
- `cargo test -p kmipkit-client`
- `cargo doc -p kmipkit-client --no-deps`
- `git diff --check`

The test run reported 34 unit tests and 6 integration tests passed, 0 failed or ignored; there were 0 doctests. This is 40 passed tests total. The generated round-trip property is one of the 34 unit tests and executes all 88 generated roots; the dedicated Big Integer vector test is a second new unit test.

No production behavior was changed. Independent T008 review is pending; this report does not close the task.
