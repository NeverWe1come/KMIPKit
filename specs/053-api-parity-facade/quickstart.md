# KMIPKIT-0053 Cross-Language Quickstart

Complete executable examples as each adapter is implemented, using published package APIs:
1. Build typed attributes (scalar, text, Unique Identifier and Structure) plus one unknown generic Item.
2. Construct a managed object and Key Block with explicit material; close/release the owner.
3. Build Cryptographic Parameters and attach caller-owned Authentication to one batch.
4. Read a validation report and demonstrate invalid pre-send NotSent with no fake-transport write.
5. Execute direct/batch and Pending operations, then read typed response and generic raw TTLV.

The Rust consumer depends only on kmipkit. The C example compiles against the shipped header/library; Java and Python use published package APIs and documented ownership. English and Spanish guides are updated in the same PR.
