# KMIPKIT-0053 Normative and Parity Checklist

- [ ] 62 applicable attributes map to KMIPKIT-0048 source tables and pass four-language tests.
- [ ] Nine managed objects and all in-scope key forms map to KMIPKIT-0049.
- [ ] All 18 Cryptographic Parameters follow §4.16 Table 59 in each binding.
- [ ] Authentication follows §8.2 Table 395 once per Request Message.
- [ ] Validation follows KMIPKIT-0051; invalid requests are NotSent with zero bytes written.
- [ ] All 56 operations and asynchronous outcomes have equivalent language access.
- [ ] Response header/payload and nested vendor TTLV follow KMIPKIT-0052.
- [ ] Unknown values, bits and generic TTLV remain available.
- [ ] Every candidate route is mapped or has a narrow justified exclusion.
- [ ] Generation and Rust/C/Java/Python export audits pass.
- [ ] Redaction, release, zeroization, ABI, and bilingual limitations are verified.
- [ ] Coverage and hosted CI state is accurate; no unsupported conformance claim is added.
