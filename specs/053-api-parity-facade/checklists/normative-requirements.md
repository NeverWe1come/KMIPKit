# KMIPKIT-0053 Normative and Parity Checklist

- [x] 62 applicable attributes map to KMIPKIT-0048 source tables and pass four-language tests.
- [x] Nine managed objects and all in-scope key forms map to KMIPKIT-0049.
- [x] All 18 Cryptographic Parameters follow KMIP 2.1 §4.16 Table 59 in each binding.
- [x] Authentication follows §8.2 Table 395 once per Request Message.
- [x] Validation follows KMIPKIT-0051; invalid requests are `NotSent` with zero bytes written.
- [x] All 56 operations and asynchronous outcomes have equivalent language access.
- [x] Response header/payload and nested vendor TTLV follow KMIPKIT-0052.
- [x] Unknown values, bits, and generic TTLV remain available.
- [x] All 750 baseline candidates are resolved: 693 exact four-language mappings and 57 exact per-route exclusions (50 enum-field artifacts, four trait implementation members, two module artifacts, and one Rust helper); the additional C block-copy bridge has its own exact exclusion.
- [x] The 934 synced validation candidates are resolved as 12 exact alias mappings and 922 individual exclusions; no owner declaration remains blocked.
- [x] Rust `RequestPayloadSchema` and `NestedStructureSchema` identities exactly match the foreign string parser, including `Unsupported` and the accepted-but-unsupported `SetEndpointRole` descriptor.
- [x] Fresh compiled Rust/C/Java/Python export audits match all 6,922 Rust declarations plus 16,936 Rust exclusions / 1,866 C / 2,161 Java / 2,347 Python routes, with zero missing, unobserved, kind-mismatched, or audit-error entries.
- [x] Independent review checked normative references, security effects, evidence paths, and the KMIPKIT-0053 exclusion set; no normative mismatch remains.
- [x] Redaction, release, zeroization, ABI, and bilingual limitations are verified.
- [x] Coverage and hosted CI state is accurately recorded as deferred; no unsupported conformance claim is added.

## Verification record

Windows x86_64 local results, including full-workspace Rust line coverage that meets each local numeric threshold, are recorded in `specs/053-api-parity-facade/implementation-evidence.md`. Cross-platform/binding coverage aggregation and hosted CI are outstanding (hosted CI is being ignored under the maintainer's billing instruction); Linux/macOS runs, the independent two-server interoperability gate, and qualified human security review also remain outside this branch's completed evidence.
